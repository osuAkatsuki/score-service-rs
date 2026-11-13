use anyhow::{Context, bail};
use lapin::options::BasicPublishOptions;
use lapin::types::ShortString;
use lapin::uri::{AMQPAuthority, AMQPUri, AMQPUserInfo};
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// How long connecting to the broker may take
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Describes where the broker is and how to log into it.
/// The parts are not joined into a URL, in which a password could be mistaken for a part of the address.
pub fn broker_uri(user: &str, password: &str, host: &str, port: &str) -> anyhow::Result<AMQPUri> {
    let port = port.parse().context("The port of the AMQP broker is invalid")?;
    Ok(AMQPUri {
        authority: AMQPAuthority {
            userinfo: AMQPUserInfo {
                username: user.to_owned(),
                password: password.to_owned(),
            },
            host: host.to_owned(),
            port,
        },
        // The virtual host every broker comes with
        vhost: "/".to_owned(),
        ..AMQPUri::default()
    })
}

/// Connects to the broker and opens the channel that messages are published on
async fn open_channel(uri: &AMQPUri) -> anyhow::Result<Channel> {
    let connecting = async {
        let runtime = async_rs::Runtime::tokio_current();
        let properties = ConnectionProperties::default();
        let connection =
            Connection::connect_uri_with_runtime(uri.clone(), properties, runtime).await?;
        // The channel keeps its connection open
        connection.create_channel().await
    };
    let channel = tokio::time::timeout(CONNECT_TIMEOUT, connecting)
        .await
        .context("Timed out connecting to the AMQP broker")??;
    Ok(channel)
}

/// Publishes messages to the default exchange of the broker.
/// A connection that was lost, as it is when the broker restarts, is established again.
pub struct AmqpPublisher {
    uri: AMQPUri,
    channel: RwLock<Channel>,
}

impl AmqpPublisher {
    pub async fn connect(uri: AMQPUri) -> anyhow::Result<Self> {
        let channel = open_channel(&uri).await?;
        Ok(Self {
            uri,
            channel: RwLock::new(channel),
        })
    }

    pub async fn publish(&self, routing_key: &str, payload: &[u8]) -> anyhow::Result<()> {
        let channel = self.connected_channel().await?;
        channel
            .basic_publish(
                ShortString::default(),
                ShortString::from(routing_key),
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await?;
        Ok(())
    }

    async fn connected_channel(&self) -> anyhow::Result<Channel> {
        {
            let channel = self.channel.read().await;
            if channel.status().connected() {
                return Ok(channel.clone());
            }
        }

        // Only one message at a time gets to reconnect. The others give up instead of
        // piling up behind it for as long as the broker is gone.
        let Ok(mut channel) = self.channel.try_write() else {
            bail!("The connection to the AMQP broker is being established again");
        };
        if !channel.status().connected() {
            warn!("Lost the connection to the AMQP broker, connecting again");
            *channel = open_channel(&self.uri).await?;
            info!("Connected to the AMQP broker again");
        }
        Ok(channel.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    const METHOD_FRAME: u8 = 1;
    const FRAME_END: u8 = 0xCE;

    /// Speaks just enough AMQP for a client to connect, open a channel and publish to it
    struct FakeBroker {
        listener: TcpListener,
    }

    impl FakeBroker {
        async fn start() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            Self { listener }
        }

        fn port(&self) -> String {
            self.listener.local_addr().unwrap().port().to_string()
        }

        /// Lets a client connect and returns its connection along with the virtual host it asked for
        async fn accept(&self) -> (TcpStream, String) {
            let (mut client, _) = self.listener.accept().await.unwrap();
            let mut protocol_header = [0; 8];
            client.read_exact(&mut protocol_header).await.unwrap();
            assert_eq!(&protocol_header, b"AMQP\x00\x00\x09\x01");

            // connection.start: version 0.9, no server properties, the mechanism and locale on offer
            let start = b"\x00\x09\x00\x00\x00\x00\x00\x00\x00\x05PLAIN\x00\x00\x00\x05en_US";
            write_method(&mut client, 0, (10, 10), start).await;
            read_method(&mut client, (10, 11)).await;
            // connection.tune: 2047 channels, frames of up to 128 KiB, no heartbeats
            write_method(&mut client, 0, (10, 30), b"\x07\xFF\x00\x02\x00\x00\x00\x00").await;
            read_method(&mut client, (10, 31)).await;
            let open = read_method(&mut client, (10, 40)).await;
            let vhost = read_short_string(&open);
            write_method(&mut client, 0, (10, 41), b"\x00").await;

            read_method(&mut client, (20, 10)).await;
            write_method(&mut client, 1, (20, 11), b"\x00\x00\x00\x00").await;
            (client, vhost)
        }

        /// Returns the routing key and the body of the next message the client publishes
        async fn receive(client: &mut TcpStream) -> (String, Vec<u8>) {
            let publish = read_method(client, (60, 40)).await;
            // Skips the reserved field and the empty name of the default exchange
            let routing_key = read_short_string(&publish[3..]);
            let (_, header) = read_frame(client).await;
            let body_size = u64::from_be_bytes(header[4..12].try_into().unwrap());
            let (_, body) = read_frame(client).await;
            assert_eq!(body.len() as u64, body_size);
            (routing_key, body)
        }
    }

    async fn read_frame(client: &mut TcpStream) -> (u8, Vec<u8>) {
        let mut header = [0; 7];
        client.read_exact(&mut header).await.unwrap();
        let size = u32::from_be_bytes(header[3..7].try_into().unwrap());
        let mut payload = vec![0; size as usize + 1];
        client.read_exact(&mut payload).await.unwrap();
        assert_eq!(payload.pop(), Some(FRAME_END));
        (header[0], payload)
    }

    /// Returns the arguments of the method, which has to be the expected one
    async fn read_method(client: &mut TcpStream, (class, method): (u16, u16)) -> Vec<u8> {
        let (frame_type, payload) = read_frame(client).await;
        assert_eq!(frame_type, METHOD_FRAME);
        assert_eq!(payload[..2], class.to_be_bytes());
        assert_eq!(payload[2..4], method.to_be_bytes());
        payload[4..].to_vec()
    }

    async fn write_method(
        client: &mut TcpStream,
        channel: u16,
        (class, method): (u16, u16),
        arguments: &[u8],
    ) {
        let mut frame = vec![METHOD_FRAME];
        frame.extend(channel.to_be_bytes());
        frame.extend((arguments.len() as u32 + 4).to_be_bytes());
        frame.extend(class.to_be_bytes());
        frame.extend(method.to_be_bytes());
        frame.extend(arguments);
        frame.push(FRAME_END);
        client.write_all(&frame).await.unwrap();
    }

    fn read_short_string(arguments: &[u8]) -> String {
        let length = arguments[0] as usize;
        String::from_utf8(arguments[1..=length].to_vec()).unwrap()
    }

    #[test]
    fn connects_to_the_default_virtual_host() {
        let uri = broker_uri("akatsuki", "p@ss/w:rd", "rabbitmq", "5672").unwrap();
        assert_eq!(uri.vhost, "/");
        assert_eq!(uri.authority.userinfo.username, "akatsuki");
        assert_eq!(uri.authority.userinfo.password, "p@ss/w:rd");
        assert_eq!(uri.authority.host, "rabbitmq");
        assert_eq!(uri.authority.port, 5672);
    }

    #[test]
    fn rejects_ports_that_are_not_one() {
        assert!(broker_uri("guest", "guest", "localhost", "").is_err());
        assert!(broker_uri("guest", "guest", "localhost", "amqp").is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn publishes_again_after_the_broker_was_gone() {
        let outage = async {
            let broker = FakeBroker::start().await;
            let uri = broker_uri("guest", "guest", "127.0.0.1", &broker.port()).unwrap();

            let (publisher, (mut client, vhost)) =
                tokio::join!(AmqpPublisher::connect(uri), broker.accept());
            let publisher = publisher.unwrap();
            assert_eq!(vhost, "/");

            publisher.publish("score_submission", b"first").await.unwrap();
            let (routing_key, body) = FakeBroker::receive(&mut client).await;
            assert_eq!(routing_key, "score_submission");
            assert_eq!(body, b"first");

            // The broker goes away, which the publisher takes a moment to find out about
            drop(client);
            while publisher.channel.read().await.status().connected() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            let (published, (mut client, _)) = tokio::join!(
                publisher.publish("score_submission", b"second"),
                broker.accept()
            );
            published.unwrap();
            let (routing_key, body) = FakeBroker::receive(&mut client).await;
            assert_eq!(routing_key, "score_submission");
            assert_eq!(body, b"second");
        };

        // Fails the test instead of hanging in it, should the two wait for each other
        let outage = tokio::time::timeout(Duration::from_secs(10), outage).await;
        assert!(outage.is_ok(), "The publisher and the broker stopped talking");
    }
}
