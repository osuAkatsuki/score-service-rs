use crate::api::context::RequestContext;
use crate::common::error::{AppError, ServiceResult};
use crate::models::osu::ratings::BeatmapRatingView;
use crate::repositories::beatmap_ratings;
use crate::usecases::{beatmaps, users};
use std::ops::RangeInclusive;
use tracing::info;

const VALID_RATINGS: RangeInclusive<u8> = 1..=10;

/// Tells the user whether they can rate the beatmap and adds their rating once they have chosen one
pub async fn rate(
    ctx: &RequestContext,
    username: &str,
    password_md5: &str,
    beatmap_md5: &str,
    rating: Option<u8>,
) -> ServiceResult<BeatmapRatingView> {
    let user = users::fetch_one_and_auth(ctx, username, password_md5).await?;
    if rating.is_some_and(|rating| !VALID_RATINGS.contains(&rating)) {
        return Err(AppError::MalformedRequest);
    }

    let beatmap = beatmaps::fetch_by_md5(beatmap_md5).await?;
    if !beatmap.has_leaderboards() {
        return Ok(BeatmapRatingView::Unranked);
    }

    if beatmap_ratings::exists(ctx, user.user_id, &beatmap.beatmap_md5).await? {
        return Ok(BeatmapRatingView::AlreadyRated {
            rating: beatmap.rating,
        });
    }

    let Some(rating) = rating else {
        return Ok(BeatmapRatingView::Rateable);
    };

    beatmap_ratings::create(ctx, user.user_id, &beatmap.beatmap_md5, rating).await?;
    let average_rating = beatmap_ratings::fetch_average(ctx, &beatmap.beatmap_md5)
        .await?
        .unwrap_or(rating as f64);
    beatmaps::update_rating(ctx, &beatmap.beatmap_md5, average_rating).await?;

    info!(
        user_id = user.user_id,
        beatmap_id = beatmap.beatmap_id,
        rating,
        average_rating,
        "User has rated a beatmap"
    );

    Ok(BeatmapRatingView::Rated {
        rating: average_rating as f32,
    })
}
