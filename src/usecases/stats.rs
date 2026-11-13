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
    stats::publish_update_event(ctx, user_id).await?;
    Ok(())
}

pub async fn increment_replays_watched(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> ServiceResult<()> {
    stats::increment_replays_watched(ctx, user_id, mode).await?;
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

    let top_plays: Vec<_> = top_plays
        .iter()
        .map(|score| {
            let accuracy = score.accuracy.as_percent(score.mode.as_gamemode());
            (score.performance as f64, accuracy as f64)
        })
        .collect();
    let (weighted_pp, weighted_acc) = weigh_top_plays(&top_plays);

    let bonus_pp = calculate_bonus_pp(ctx, user_id, mode).await?;
    let total_pp = (weighted_pp + bonus_pp).round() as u32;
    Ok((total_pp, weighted_acc))
}

/// Weighs the performance and accuracy of scores that are sorted by their performance.
/// Every score is worth 5% less than the one before it.
fn weigh_top_plays(top_plays: &[(f64, f64)]) -> (f64, f64) {
    let mut weighted_pp = 0.0;
    let mut weighted_acc = 0.0;
    for (i, (performance, accuracy)) in top_plays.iter().enumerate() {
        let weighting = 0.95f64.powi(i as _);
        weighted_pp += performance * weighting;
        weighted_acc += accuracy * weighting;
    }

    // Normalizes the accuracy, as the weights of less than 100 scores do not add up to 20
    let total_weight = 20.0 * (1.0 - 0.95f64.powi(top_plays.len() as _));
    (weighted_pp, weighted_acc / total_weight)
}

pub async fn calculate_bonus_pp(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> ServiceResult<f64> {
    let ranked_scores_count = scores::fetch_user_ranked_scores_count(ctx, user_id, mode).await?;
    Ok(416.6667 * (1.0 - 0.995f64.powi(ranked_scores_count as _)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_score_is_worth_all_of_its_performance_and_accuracy() {
        let (pp, accuracy) = weigh_top_plays(&[(250.0, 97.5)]);
        assert!((pp - 250.0).abs() < 1e-9);
        assert!((accuracy - 97.5).abs() < 1e-9);
    }

    #[test]
    fn scores_are_worth_less_the_further_down_they_are() {
        let (pp, accuracy) = weigh_top_plays(&[(300.0, 100.0), (200.0, 90.0), (100.0, 80.0)]);
        assert!((pp - (300.0 + 190.0 + 90.25)).abs() < 1e-9);
        // (100 + 85.5 + 72.2) / (1 + 0.95 + 0.9025)
        assert!((accuracy - 90.341805).abs() < 1e-5);
    }
}
