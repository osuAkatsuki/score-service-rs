use crate::api::context::RequestContext;

pub async fn exists(ctx: &RequestContext, user_id: i64, beatmap_md5: &str) -> sqlx::Result<bool> {
    const QUERY: &str =
        "SELECT EXISTS(SELECT 1 FROM beatmaps_rating WHERE user_id = ? AND beatmap_md5 = ?)";
    sqlx::query_scalar(QUERY)
        .bind(user_id)
        .bind(beatmap_md5)
        .fetch_one(&ctx.db)
        .await
}

pub async fn create(
    ctx: &RequestContext,
    user_id: i64,
    beatmap_md5: &str,
    rating: u8,
) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO beatmaps_rating (user_id, rating, beatmap_md5) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(rating)
        .bind(beatmap_md5)
        .execute(&ctx.db)
        .await?;
    Ok(())
}

/// The average of all ratings of the beatmap, if it has been rated
pub async fn fetch_average(ctx: &RequestContext, beatmap_md5: &str) -> sqlx::Result<Option<f64>> {
    sqlx::query_scalar("SELECT AVG(rating) FROM beatmaps_rating WHERE beatmap_md5 = ?")
        .bind(beatmap_md5)
        .fetch_one(&ctx.db)
        .await
}
