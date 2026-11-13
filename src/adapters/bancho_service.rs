use std::sync::LazyLock;
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