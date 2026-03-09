use crate::api::context::RequestContext;
use crate::entities::seasonal_backgrounds::SeasonalBackground;

pub async fn fetch_all_enabled(ctx: &RequestContext) -> sqlx::Result<Vec<SeasonalBackground>> {
    sqlx::query_as("SELECT id, enabled, url FROM seasonal_bg WHERE enabled = 1")
        .fetch_all(&ctx.db)
        .await
}