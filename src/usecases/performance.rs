use crate::adapters::performance_service;
use crate::adapters::performance_service::PerformanceRequest;
use crate::common::error::ServiceResult;
use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;
use crate::models::performance::{BeatmapPerformance, PerformancePoints};
use crate::usecases::beatmaps;

/// The accuracies players usually want to know the performance of
const COMMON_ACCURACIES: [f32; 4] = [100.0, 99.0, 98.0, 95.0];

/// Calculates what a play without misses on the beatmap is worth.
/// Without an accuracy the performance of the common accuracies is calculated,
/// and without a combo the play is assumed to be a full combo.
pub async fn calculate_for_beatmap(
    beatmap_id: i32,
    mode: Gamemode,
    mods: Mods,
    accuracy: Option<f32>,
    max_combo: Option<i32>,
) -> ServiceResult<BeatmapPerformance> {
    let beatmap = beatmaps::fetch_by_id(beatmap_id).await?;
    let max_combo = max_combo
        .filter(|max_combo| *max_combo > 0)
        .unwrap_or(beatmap.max_combo);

    let request_for = |accuracy: f32| PerformanceRequest {
        beatmap_id: beatmap.beatmap_id,
        beatmap_md5: &beatmap.beatmap_md5,
        mode: mode as _,
        mods: mods.bits(),
        max_combo,
        accuracy,
        miss_count: 0,
    };

    let (pp, stars) = match accuracy {
        Some(accuracy) => {
            let result = performance_service::calculate(request_for(accuracy)).await?;
            (PerformancePoints::Single(result.pp), result.stars)
        }
        None => {
            let requests = COMMON_ACCURACIES.map(request_for);
            let results = performance_service::calculate_many(&requests).await?;
            let pp = results.iter().map(|result| result.pp).collect();
            // The accuracy of a play has no say in how difficult the beatmap is
            (PerformancePoints::Common(pp), results[0].stars)
        }
    };

    Ok(BeatmapPerformance { beatmap, stars, pp })
}
