use crate::api::context::RequestContext;
use crate::common::error::AppError;
use crate::usecases::replays;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetReplayArgs {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "h")]
    pub password_md5: String,
    #[serde(rename = "c")]
    pub score_id: i64,
}

const ERROR_SCORE_NOT_FOUND: &str = "error: no";
/// An empty response tells the client that there is nothing to watch
const ERROR_REPLAY_NOT_FOUND: &str = "";

pub async fn handler(ctx: RequestContext, Query(args): Query<GetReplayArgs>) -> Response {
    match replays::watch(&ctx, &args.username, &args.password_md5, args.score_id).await {
        Ok(replay) => replay.into_response(),
        Err(AppError::ScoresNotFound) => ERROR_SCORE_NOT_FOUND.into_response(),
        Err(AppError::ReplaysNotFound) => ERROR_REPLAY_NOT_FOUND.into_response(),
        Err(e) => e.into_response(),
    }
}

/// Unlike the replays the client requests to watch them, these are complete replay files
pub async fn download(ctx: RequestContext, Path(score_id): Path<i64>) -> Response {
    match replays::fetch_replay_file(&ctx, score_id).await {
        Ok(replay_file) => replay_file.into_response(),
        Err(AppError::ScoresNotFound) => (StatusCode::NOT_FOUND, "Score not found!").into_response(),
        Err(AppError::ReplaysNotFound) => {
            (StatusCode::NOT_FOUND, "Replay not found!").into_response()
        }
        Err(AppError::UsersNotFound) => (StatusCode::NOT_FOUND, "User not found!").into_response(),
        Err(AppError::BeatmapsNotFound) => {
            (StatusCode::NOT_FOUND, "Beatmap not found!").into_response()
        }
        Err(e) => e.into_response(),
    }
}
