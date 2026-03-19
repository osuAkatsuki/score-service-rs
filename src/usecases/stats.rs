use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::entities::mode::ModdedGamemode;
use crate::entities::stats::{Stats, UpdateStats};
use crate::repositories::{scores, stats};

pub async fn fetch_one(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> ServiceResult<Stats> {
    let stats = stats::fetch_one(ctx, user_id, mode).await?;
    Ok(stats)
}

pub async fn update(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
    update: &UpdateStats,
) -> ServiceResult<()> {
    stats::update(ctx, user_id, mode, update).await?;
    Ok(())
}

pub async fn fetch_global_rank(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> ServiceResult<Option<usize>> {
    let rank = stats::fetch_global_rank(ctx, user_id, mode).await?;
    Ok(rank)
}

pub async fn update_rank(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
    user_country: &str,
    performance: u32,
) -> ServiceResult<Option<usize>> {
    let updated_rank = stats::update_rank(ctx, user_id, mode, user_country, performance).await?;
    Ok(updated_rank)
}

/// Recalculates the users' weighted stats (e.g. pp, accuracy)
pub async fn recalculate_weighted(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> ServiceResult<(u32, f64)> {
    let top_plays = scores::fetch_user_top_plays(ctx, user_id, mode).await?;
    if top_plays.is_empty() {
        return Ok((0, 100.0));
    }

    let mut weighted_pp = 0.0;
    let mut weighted_acc = 0.0;
    for (i, score) in top_plays.iter().enumerate() {
        let weighting = 0.95f32.powi(i as _);
        let weighted_score_pp = score.performance * weighting;
        let weighted_score_acc = score.accuracy.as_percent(score.mode.as_gamemode()) * weighting;

        weighted_pp += weighted_score_pp;
        weighted_acc += weighted_score_acc;
    }

    let bonus_pp = calculate_bonus_pp(ctx, user_id, mode).await?;
    let total_pp = (weighted_pp + bonus_pp) as u32;
    let total_acc =
        (weighted_acc * (100.0 / (20.0 * (1.0 - 0.95f32.powi(top_plays.len() as _))))) / 100.0;
    Ok((total_pp, total_acc as _))
}

pub async fn calculate_bonus_pp(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> ServiceResult<f32> {
    let ranked_scores_count = scores::fetch_user_ranked_scores_count(ctx, user_id, mode).await?;
    Ok(416.6667f32 * (1.0f32 - (0.995f32.powi(ranked_scores_count as _))))
}
