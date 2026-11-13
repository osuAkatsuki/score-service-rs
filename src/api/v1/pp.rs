use crate::common::error::AppError;
use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;
use crate::models::performance::{BeatmapPerformance, PerformancePoints};
use crate::usecases::performance;
use axum::Json;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CalculatePerformanceArgs {
    #[serde(rename = "b")]
    pub beatmap_id: i32,
    #[serde(rename = "m", default)]
    pub mods: Mods,
    #[serde(rename = "g", default)]
    pub mode: Gamemode,
    #[serde(rename = "a")]
    pub accuracy: Option<f32>,
    pub max_combo: Option<i32>,
}

/// This endpoint keeps the response format its consumers know from the services before this one
#[derive(Serialize)]
pub struct PerformanceResponse {
    pub status: u16,
    pub message: &'static str,
    pub song_name: String,
    pub pp: PerformancePoints,
    pub length: i32,
    pub stars: f32,
    pub ar: f32,
    pub bpm: i32,
}

impl From<BeatmapPerformance> for PerformanceResponse {
    fn from(value: BeatmapPerformance) -> Self {
        Self {
            status: StatusCode::OK.as_u16(),
            message: "ok",
            song_name: value.beatmap.song_name,
            pp: value.pp,
            length: value.beatmap.hit_length,
            stars: value.stars,
            ar: value.beatmap.ar,
            bpm: value.beatmap.bpm,
        }
    }
}

#[derive(Serialize)]
pub struct PerformanceErrorResponse {
    pub message: &'static str,
}

const ERROR_BEATMAP_NOT_FOUND: PerformanceErrorResponse = PerformanceErrorResponse {
    message: "Invalid/non-existent beatmap id.",
};

pub async fn calculate(Query(args): Query<CalculatePerformanceArgs>) -> Response {
    let result = performance::calculate_for_beatmap(
        args.beatmap_id,
        args.mode,
        args.mods,
        args.accuracy,
        args.max_combo,
    )
    .await;

    match result {
        Ok(performance) => Json(PerformanceResponse::from(performance)).into_response(),
        Err(AppError::BeatmapsNotFound) => {
            (StatusCode::BAD_REQUEST, Json(ERROR_BEATMAP_NOT_FOUND)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Uri;

    fn parse_args(uri: &str) -> Option<CalculatePerformanceArgs> {
        let uri: Uri = uri.parse().unwrap();
        let query = Query::<CalculatePerformanceArgs>::try_from_uri(&uri).ok()?;
        Some(query.0)
    }

    #[test]
    fn assumes_a_nomod_osu_play_unless_told_otherwise() {
        let args = parse_args("/api/v1/pp?b=75").unwrap();
        assert_eq!(args.beatmap_id, 75);
        assert_eq!(args.mods, Mods::None);
        assert_eq!(args.mode, Gamemode::Standard);
        assert_eq!(args.accuracy, None);
        assert_eq!(args.max_combo, None);
    }

    #[test]
    fn reads_the_mods_mode_accuracy_and_combo_of_the_play() {
        let args = parse_args("/api/v1/pp?b=75&m=72&g=1&a=99.5&max_combo=300").unwrap();
        assert_eq!(args.mods, Mods::Hidden | Mods::Doubletime);
        assert_eq!(args.mode, Gamemode::Taiko);
        assert_eq!(args.accuracy, Some(99.5));
        assert_eq!(args.max_combo, Some(300));
    }

    #[test]
    fn rejects_modes_that_do_not_exist() {
        assert!(parse_args("/api/v1/pp?b=75&g=4").is_none());
        assert!(parse_args("/api/v1/pp").is_none());
    }
}
