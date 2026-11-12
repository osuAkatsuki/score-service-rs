use crate::api::context::RequestContext;

/// Adds the beatmapset to the favourites of the user.
/// Returns false if it already was one of their favourites.
pub async fn create(ctx: &RequestContext, user_id: i64, beatmapset_id: i32) -> sqlx::Result<bool> {
    const QUERY: &str = "INSERT IGNORE INTO user_favourites (user_id, beatmapset_id) VALUES (?, ?)";
    let result = sqlx::query(QUERY)
        .bind(user_id)
        .bind(beatmapset_id)
        .execute(&ctx.db)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn fetch_beatmapset_ids(ctx: &RequestContext, user_id: i64) -> sqlx::Result<Vec<i32>> {
    sqlx::query_scalar("SELECT beatmapset_id FROM user_favourites WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(&ctx.db)
        .await
}
