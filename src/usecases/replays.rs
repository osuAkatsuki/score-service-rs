use crate::adapters::s3;
use crate::api::context::RequestContext;
use crate::common::error::{AppError, ServiceResult};
use crate::usecases::{scores, stats, users};
use axum::body::Bytes;

pub async fn save_replay(
    ctx: &RequestContext,
    score_id: i64,
    replay_bytes: Bytes,
) -> ServiceResult<()> {
    let file_name = format!("{score_id}.osr");
    s3::upload(ctx, &file_name, "replays", replay_bytes).await?;
    Ok(())
}

pub async fn download_replay(ctx: &RequestContext, score_id: i64) -> ServiceResult<Vec<u8>> {
    let file_name = format!("{score_id}.osr");
    match s3::download(ctx, &file_name, "replays").await? {
        Some(replay) => Ok(replay),
        None => Err(AppError::ReplaysNotFound),
    }
}

/// Fetches the replay of a score for a user that is going to watch it in their client
pub async fn watch(
    ctx: &RequestContext,
    username: &str,
    password_md5: &str,
    score_id: i64,
) -> ServiceResult<Vec<u8>> {
    let user = users::fetch_one_and_auth(ctx, username, password_md5).await?;
    let score = scores::fetch_one(ctx, score_id).await?;
    let replay = download_replay(ctx, score_id).await?;

    // Players watching themselves do not count as views
    if score.user_id != user.user_id {
        stats::increment_replays_watched(ctx, score.user_id, score.mode).await?;
    }

    Ok(replay)
}
