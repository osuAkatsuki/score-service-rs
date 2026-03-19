use crate::common::error::{AppError, ServiceResult};
use crate::settings::AppSettings;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

fn make_url(endpoint: &str) -> String {
    let settings = AppSettings::get();
    format!("{}{endpoint}", settings.performance_service_base_url)
}

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::new());

#[derive(Serialize)]
pub struct PerformanceRequest<'a> {
    pub beatmap_id: i32,
    pub beatmap_md5: &'a str,
    pub mode: i32,
    pub mods: u32,
    pub max_combo: i32,
    pub accuracy: f32,
    pub miss_count: i32,
}

#[derive(Deserialize, Copy, Clone)]
pub struct PerformanceResult {
    pub max_combo: i32,
    pub stars: f32,
    pub pp: f32,
    pub ar: f32,
    pub od: f32,
}

pub async fn calculate(args: PerformanceRequest<'_>) -> ServiceResult<PerformanceResult> {
    /*let url = make_url("/api/v1/calculate");
    let response = CLIENT.post(url).json(&[args]).send().await?;
    if response.status() != StatusCode::OK {
        Err(AppError::PerformanceFailedCalculating)
    } else {
        let results: [PerformanceResult; 1] = response.json().await?;
        Ok(results[0])
    }*/
    Ok(PerformanceResult {
        max_combo: 314,
        stars: 2.63,
        pp: 50.0,
        ar: 6.0,
        od: 6.0,
    })
}
