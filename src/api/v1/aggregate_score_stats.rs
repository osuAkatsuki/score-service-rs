use crate::api::context::RequestContext;
use crate::common::error::ServiceResponse;
use crate::usecases::scores;
use axum::Json;

/// Responds with the amount of scores that have ever been set on the server
pub async fn total_scores_set(ctx: RequestContext) -> ServiceResponse<i64> {
    let total_scores = scores::fetch_total_count(&ctx).await?;
    Ok(Json(total_scores))
}
