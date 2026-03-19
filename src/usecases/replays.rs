use crate::adapters::s3;
use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
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
    s3::download(ctx, &file_name, "replays").await
}
