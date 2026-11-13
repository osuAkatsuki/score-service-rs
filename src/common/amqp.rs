use anyhow::Context;
use lapin::uri::{AMQPAuthority, AMQPUri, AMQPUserInfo};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
