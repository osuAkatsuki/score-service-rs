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

/// Calculates the performance of multiple scores at once.
/// The results are in the order of the requested scores.
pub async fn calculate_many(
    args: &[PerformanceRequest<'_>],
) -> ServiceResult<Vec<PerformanceResult>> {
    let url = make_url("/api/v1/calculate");
    let response = CLIENT.post(url).json(args).send().await?;
    if response.status() != StatusCode::OK {
        return Err(AppError::PerformanceFailedCalculating);
    }

    let results: Vec<PerformanceResult> = response.json().await?;
    match results.len() == args.len() {
        true => Ok(results),
        false => Err(AppError::PerformanceFailedCalculating),
    }
}

pub async fn calculate(args: PerformanceRequest<'_>) -> ServiceResult<PerformanceResult> {
    let results = calculate_many(&[args]).await?;
    Ok(results[0])
}
