use std::sync::LazyLock;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::common::error::ServiceResult;
use crate::settings::AppSettings;

fn make_url(endpoint: &str) -> String {
    let settings = AppSettings::get();
    format!("{}{endpoint}", settings.bancho_service_base_url)
}

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::new());

#[derive(Deserialize)]
pub struct BaseSuccessData {
    pub message: String,
    pub status: u16,
}

#[derive(Serialize)]
pub struct FetchPlayerMatchDetailsArgs {
    #[serde(rename = "id")]
    pub user_id: i64,
}

#[derive(Deserialize)]
pub struct PlayerMatchDetails {
    pub match_name: String,
    pub match_id: i64,
    pub slot_id: u8,
    pub game_id: i64,
    pub team: u8,
}

#[derive(Deserialize)]
pub struct PlayerMatchDetailsResponse {
    #[serde(flatten)]
    pub base: BaseSuccessData,
    pub result: Option<PlayerMatchDetails>,
}

#[derive(Serialize)]
struct SendMessageArgs<'a> {
    #[serde(rename = "k")]
    key: &'a str,
    #[serde(rename = "to")]
    target: &'a str,
    #[serde(rename = "msg")]
    message: &'a str,
}

/// Sends a chat message to a channel or user on behalf of the bot
pub async fn send_message(target: &str, message: &str) -> ServiceResult<()> {
    let settings = AppSettings::get();
    let url = make_url("/api/v1/fokabotMessage");
    let args = SendMessageArgs {
        key: &settings.app_ci_key,
        target,
        message,
    };
    CLIENT
        .get(url)
        .query(&args)
        .timeout(Duration::from_secs(15))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn fetch_player_match_details(user_id: i64) -> ServiceResult<Option<PlayerMatchDetails>> {
    let url = make_url("/api/v1/playerMatchDetails");
    let response = CLIENT
        .get(url)
        .query(&FetchPlayerMatchDetailsArgs { user_id })
        .send()
        .await?;
    let response: PlayerMatchDetailsResponse = response.json().await?;
    Ok(response.result)
}