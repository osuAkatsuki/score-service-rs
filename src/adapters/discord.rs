use crate::common::error::{ServiceResult, unexpected};
use crate::settings::AppSettings;
use chrono::{SecondsFormat, Utc};
use reqwest::StatusCode;
use serde_json::{Value, json};
use std::fmt;
use std::sync::LazyLock;
use std::time::Duration;

/// How often sending a message is attempted before giving up on it
const MAX_ATTEMPTS: u32 = 10;

const PURPLE_EMBED_COLOR: u32 = 0x6611FF;
const RED_EMBED_COLOR: u32 = 0xFF5555;
const BLUE_EMBED_COLOR: u32 = 0x00a2ff;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(super::http_client);

pub async fn send_ac_purple_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
) -> ServiceResult<()> {
    send_ac_embed(title, description, url, PURPLE_EMBED_COLOR).await
}

pub async fn send_ac_blue_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
) -> ServiceResult<()> {
    send_ac_embed(title, description, url, BLUE_EMBED_COLOR).await
}

pub async fn send_ac_red_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
) -> ServiceResult<()> {
    send_ac_embed(title, description, url, RED_EMBED_COLOR).await
}

pub async fn send_ac_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
    color: u32,
) -> ServiceResult<()> {
    let settings = AppSettings::get();
    match &settings.discord_ac_logs_webhook_url {
        None => {
            tracing::warn!(title, description, url, "Discord Logs Webhook url not set");
            Ok(())
        }
        Some(webhook_url) => send_embed(webhook_url, title, description, url, color).await,
    }
}

pub async fn send_hw_purple_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
) -> ServiceResult<()> {
    send_hw_embed(title, description, url, PURPLE_EMBED_COLOR).await
}

pub async fn send_hw_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
    color: u32,
) -> ServiceResult<()> {
    let settings = AppSettings::get();
    match &settings.discord_hw_logs_webhook_url {
        None => {
            tracing::warn!(
                title,
                description,
                url,
                "Discord Hardware Logs Webhook url not set"
            );
            Ok(())
        }
        Some(webhook_url) => send_embed(webhook_url, title, description, url, color).await,
    }
}

pub async fn send_embed(
    webhook_url: &str,
    title: &str,
    description: &str,
    url: Option<&str>,
    color: u32,
) -> ServiceResult<()> {
    let message = embed_message(title, description, url, color);

    // Waits 1, 2, 4, ... seconds between the attempts
    let mut attempt = 1;
    loop {
        match send_message(webhook_url, &message).await {
            Ok(()) => return Ok(()),
            Err(e) if attempt < MAX_ATTEMPTS && e.is_worth_retrying() => {
                tokio::time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
                attempt += 1;
            }
            Err(e) => return unexpected(e),
        }
    }
}

/// The body of a webhook message that consists of a single embed
fn embed_message(title: &str, description: &str, url: Option<&str>, color: u32) -> Value {
    let mut author = json!({ "name": title });
    if let Some(url) = url {
        author["url"] = json!(url);
    }
    json!({
        "embeds": [{
            "description": description,
            "author": author,
            "color": color,
            "footer": { "text": "score-service 🎯" },
            "timestamp": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        }]
    })
}

async fn send_message(webhook_url: &str, message: &Value) -> Result<(), SendError> {
    let response = CLIENT
        .post(webhook_url)
        // Without it, Discord does not report messages that it fails to save
        .query(&[("wait", "true")])
        .json(message)
        .send()
        .await
        .map_err(SendError::Request)?;

    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    let body = response.text().await.unwrap_or_default();
    Err(SendError::Rejected { status, body })
}

#[derive(Debug)]
enum SendError {
    /// The message did not make it to Discord, or its answer did not make it back
    Request(reqwest::Error),
    /// Discord answered, but did not accept the message
    Rejected { status: StatusCode, body: String },
}

impl SendError {
    /// Only failures that are likely to go away on their own are worth another attempt
    fn is_worth_retrying(&self) -> bool {
        match self {
            SendError::Request(e) => e.is_connect() || e.is_timeout() || e.is_request(),
            // Discord tells clients that are sending too many messages to slow down
            SendError::Rejected { status, .. } => *status == StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendError::Request(e) => write!(f, "Failed to send the Discord message: {e}"),
            SendError::Rejected { status, body } => {
                write!(f, "Discord rejected the message with {status}: {body}")
            }
        }
    }
}

impl std::error::Error for SendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SendError::Request(e) => Some(e),
            SendError::Rejected { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejected(status: StatusCode) -> SendError {
        SendError::Rejected {
            status,
            body: String::new(),
        }
    }

    #[test]
    fn retries_rate_limited_messages_only() {
        assert!(rejected(StatusCode::TOO_MANY_REQUESTS).is_worth_retrying());
        assert!(!rejected(StatusCode::BAD_REQUEST).is_worth_retrying());
        assert!(!rejected(StatusCode::NOT_FOUND).is_worth_retrying());
    }

    #[test]
    fn links_the_author_only_when_there_is_a_url() {
        let link = "https://akatsuki.gg/u/1000";
        let message = embed_message("User restricted", "reason", Some(link), RED_EMBED_COLOR);
        assert_eq!(
            message["embeds"][0]["author"],
            json!({ "name": "User restricted", "url": link })
        );

        let message = embed_message("User restricted", "reason", None, RED_EMBED_COLOR);
        assert_eq!(
            message["embeds"][0]["author"],
            json!({ "name": "User restricted" })
        );
    }

    #[test]
    fn timestamps_have_millisecond_precision() {
        let message = embed_message("User restricted", "reason", None, RED_EMBED_COLOR);
        let timestamp = message["embeds"][0]["timestamp"].as_str().unwrap();
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp).is_ok());
        assert_eq!(timestamp.len(), "2026-11-16T00:56:44.085Z".len());
    }
}
