use crate::common::error::{ServiceResult, unexpected};
use crate::settings::AppSettings;
use discord_webhook2::error::DiscordWebhookError;
use discord_webhook2::message::Message;
use discord_webhook2::webhook::DiscordWebhook;
use iso8601_timestamp::Timestamp;
use std::time::Duration;

/// How often sending a message is attempted before giving up on it
const MAX_ATTEMPTS: u32 = 10;

const PURPLE_EMBED_COLOR: u32 = 0x6611FF;
const RED_EMBED_COLOR: u32 = 0xFF5555;
const BLUE_EMBED_COLOR: u32 = 0x00a2ff;

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

pub async fn send_embed(
    webhook_url: &str,
    title: &str,
    description: &str,
    url: Option<&str>,
    color: u32,
) -> ServiceResult<()> {
    let webhook = DiscordWebhook::new(webhook_url)?;
    let message = Message::new(|message| {
        message.embed(|embed| {
            embed
                .description(description)
                .author(|author| {
                    let author = author.name(title);
                    match url {
                        Some(url) => author.url(url),
                        None => author,
                    }
                })
                .color(color)
                .footer(|footer| footer.text("score-service 🎯"))
                .timestamp(Timestamp::now_utc())
        })
    });

    // Waits 1, 2, 4, ... seconds between the attempts
    let mut attempt = 1;
    loop {
        match webhook.send(&message).await {
            Ok(_) => return Ok(()),
            Err(e) if attempt < MAX_ATTEMPTS && is_worth_retrying(&e) => {
                tokio::time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
                attempt += 1;
            }
            Err(e) => return unexpected(e),
        }
    }
}

/// Only failures that are likely to go away on their own are worth another attempt
fn is_worth_retrying(error: &DiscordWebhookError) -> bool {
    match error {
        DiscordWebhookError::ReqwestError(e) => e.is_connect() || e.is_timeout() || e.is_request(),
        // All that is known about a rejected message is the body of the response.
        // Discord tells clients that are sending too many messages when to try again.
        DiscordWebhookError::FormatError(response) => response.contains("retry_after"),
        DiscordWebhookError::UrlParseError(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retries_rate_limited_messages_only() {
        let rate_limited = DiscordWebhookError::FormatError(
            r#"{"message": "You are being rate limited.", "retry_after": 0.5, "global": false}"#
                .to_owned(),
        );
        assert!(is_worth_retrying(&rate_limited));

        let invalid = DiscordWebhookError::FormatError(
            r#"{"message": "Cannot send an empty message", "code": 50006}"#.to_owned(),
        );
        assert!(!is_worth_retrying(&invalid));
    }
}
