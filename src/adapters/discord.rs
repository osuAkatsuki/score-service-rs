use crate::common::error::ServiceResult;
use crate::settings::AppSettings;
use discord_webhook2::message::Message;
use discord_webhook2::webhook::DiscordWebhook;
use iso8601_timestamp::Timestamp;

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
    webhook
        .send(&Message::new(|message| {
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
        }))
        .await?;

    Ok(())
}
