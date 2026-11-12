use crate::api::context::RequestContext;
use crate::common::error::AppError;
use crate::usecases::beatmap_ratings;
use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RateBeatmapArgs {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "p")]
    pub password_md5: String,
    #[serde(rename = "c")]
    pub beatmap_md5: String,
    /// The client first asks whether the user can rate the beatmap before it sends their rating
    #[serde(rename = "v")]
    pub rating: Option<u8>,
}

const ERROR_BEATMAP_NOT_FOUND: &str = "no exist";

pub async fn handler(ctx: RequestContext, Query(args): Query<RateBeatmapArgs>) -> Response {
    let result = beatmap_ratings::rate(
        &ctx,
        &args.username,
        &args.password_md5,
        &args.beatmap_md5,
        args.rating,
    )
    .await;

    match result {
        Ok(rating_view) => rating_view.into_response(),
        Err(AppError::BeatmapsNotFound) => ERROR_BEATMAP_NOT_FOUND.into_response(),
        Err(e) => e.into_response(),
    }
}
