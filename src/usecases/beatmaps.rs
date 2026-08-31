use crate::adapters::beatmaps_service;
use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::models::beatmaps::{Beatmap, BeatmapGamemode, RankedStatus};
use crate::repositories::beatmaps;

pub async fn fetch_by_md5(beatmap_md5: &str) -> ServiceResult<Beatmap> {
    // let beatmap = beatmaps_service::fetch_by_md5(beatmap_md5).await?;
    // Ok(Beatmap::from(beatmap))
    Ok(Beatmap {
        beatmap_id: 75,
        beatmap_set_id: 1,
        beatmap_md5: "233f55099932d0696a3ef192041bc30d".to_owned(),
        song_name: "a".to_string(),
        file_name: "b".to_string(),
        ar: 6.0,
        od: 6.0,
        mode: BeatmapGamemode::Any,
        max_combo: 314,
        hit_length: 142,
        bpm: 120,
        status: RankedStatus::Ranked,
        status_frozen: false,
        latest_update: 0,
        playcount: 745970,
        passcount: 93965,
        ranked_by: None,
        rating: 0.0,
        bancho_ranked_status: None,
        count_circles: Some(160),
        count_spinners: Some(4),
        count_sliders: Some(30),
        bancho_creator_id: None,
        bancho_creator_name: None,
    })
}

pub async fn increment_playcount(
    ctx: &RequestContext,
    beatmap_md5: &str,
    increment_passcount: bool,
) -> ServiceResult<()> {
    beatmaps::increment_playcount(ctx, beatmap_md5, increment_passcount).await?;
    Ok(())
}

pub async fn file_name_exists(
    ctx: &RequestContext,
    file_name: &str,
) -> ServiceResult<bool> {
    let exists = beatmaps::file_name_exists(ctx, file_name).await?;
    Ok(exists)
}