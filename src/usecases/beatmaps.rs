use crate::adapters::beatmaps_service;
use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::models::beatmaps::Beatmap;
use crate::repositories::beatmaps;

pub async fn fetch_by_md5(beatmap_md5: &str) -> ServiceResult<Beatmap> {
    let beatmap = beatmaps_service::fetch_by_md5(beatmap_md5).await?;
    Ok(Beatmap::from(beatmap))
}

pub async fn fetch_by_id(beatmap_id: i32) -> ServiceResult<Beatmap> {
    let beatmap = beatmaps_service::fetch_by_id(beatmap_id).await?;
    Ok(Beatmap::from(beatmap))
}

pub async fn increment_playcount(
    ctx: &RequestContext,
    beatmap_md5: &str,
    increment_passcount: bool,
) -> ServiceResult<()> {
    beatmaps::increment_playcount(ctx, beatmap_md5, increment_passcount).await?;
    Ok(())
}

pub async fn update_rating(
    ctx: &RequestContext,
    beatmap_md5: &str,
    rating: f64,
) -> ServiceResult<()> {
    beatmaps::update_rating(ctx, beatmap_md5, rating).await?;
    Ok(())
}

pub async fn file_name_exists(ctx: &RequestContext, file_name: &str) -> ServiceResult<bool> {
    let exists = beatmaps::file_name_exists(ctx, file_name).await?;
    Ok(exists)
}
