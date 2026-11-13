use crate::common::error::{AppError, ServiceResult};
use crate::settings::AppSettings;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::Duration;

/// Searching the mirror can take a while when it has to ask its own upstreams
const MIRROR_TIMEOUT: Duration = Duration::from_secs(15);

fn make_url(endpoint: &str) -> String {
    let settings = AppSettings::get();
    format!("{}{endpoint}", settings.beatmaps_service_base_url)
}

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(super::http_client);

#[derive(Deserialize)]
pub struct AkatsukiBeatmap {
    pub beatmap_id: i32,
    pub beatmapset_id: i32,
    pub beatmap_md5: String,
    pub song_name: String,
    pub file_name: String,
    pub ar: f32,
    pub od: f32,
    pub mode: u8,
    pub max_combo: i32,
    pub hit_length: i32,
    pub bpm: i32,
    pub ranked: i8,
    pub latest_update: i64,
    pub ranked_status_freezed: bool,
    pub playcount: i64,
    pub passcount: i64,
    pub rankedby: Option<i64>,
    pub rating: f32,
    pub bancho_ranked_status: Option<i8>,
    pub count_circles: Option<i32>,
    pub count_spinners: Option<i32>,
    pub count_sliders: Option<i32>,
    pub bancho_creator_id: Option<i32>,
    pub bancho_creator_name: Option<String>,
}

/// A beatmap in the format of the cheesegull beatmap mirror
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheesegullBeatmap {
    pub diff_name: String,
    pub mode: i32,
    #[serde(rename = "AR")]
    pub ar: f32,
    #[serde(rename = "OD")]
    pub od: f32,
    #[serde(rename = "CS")]
    pub cs: f32,
    #[serde(rename = "HP")]
    pub hp: f32,
    pub difficulty_rating: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheesegullBeatmapset {
    #[serde(rename = "SetID")]
    pub set_id: i32,
    pub children_beatmaps: Vec<CheesegullBeatmap>,
    pub ranked_status: i32,
    pub last_update: String,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub has_video: bool,
}

#[derive(Serialize)]
pub struct BeatmapsetSearchArgs<'a> {
    pub query: Option<&'a str>,
    pub mode: Option<i32>,
    pub status: Option<i32>,
    pub offset: i32,
    pub amount: i32,
}

pub async fn search_beatmapsets(
    args: BeatmapsetSearchArgs<'_>,
) -> ServiceResult<Vec<CheesegullBeatmapset>> {
    let url = make_url("/public/api/search");
    let response = CLIENT
        .get(url)
        .query(&args)
        .timeout(MIRROR_TIMEOUT)
        .send()
        .await?;
    Ok(response.error_for_status()?.json().await?)
}

pub async fn fetch_beatmapset(beatmapset_id: i32) -> ServiceResult<CheesegullBeatmapset> {
    let url = make_url(&format!("/public/api/s/{beatmapset_id}"));
    let response = CLIENT.get(url).timeout(MIRROR_TIMEOUT).send().await?;
    match response.status() {
        StatusCode::NOT_FOUND => Err(AppError::BeatmapsNotFound),
        _ => Ok(response.error_for_status()?.json().await?),
    }
}

/// A beatmap is looked up by exactly one of these
#[derive(Serialize, Default)]
struct BeatmapLookupQuery<'a> {
    beatmap_md5: Option<&'a str>,
    beatmap_id: Option<i32>,
}

async fn lookup(query: BeatmapLookupQuery<'_>) -> ServiceResult<AkatsukiBeatmap> {
    let url = make_url("/api/akatsuki/v1/beatmaps/lookup");
    let response = CLIENT.get(url).query(&query).send().await?;
    match response.status() {
        StatusCode::NOT_FOUND => Err(AppError::BeatmapsNotFound),
        _ => Ok(response.error_for_status()?.json().await?),
    }
}

pub async fn fetch_by_md5(beatmap_md5: &str) -> ServiceResult<AkatsukiBeatmap> {
    let query = BeatmapLookupQuery {
        beatmap_md5: Some(beatmap_md5),
        ..Default::default()
    };
    lookup(query).await
}

pub async fn fetch_by_id(beatmap_id: i32) -> ServiceResult<AkatsukiBeatmap> {
    let query = BeatmapLookupQuery {
        beatmap_id: Some(beatmap_id),
        ..Default::default()
    };
    lookup(query).await
}
