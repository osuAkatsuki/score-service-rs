use crate::api::context::RequestContext;
use crate::common::error::ServiceResponse;
use crate::usecases::seasonal_backgrounds;
use axum::Json;

pub async fn handler(ctx: RequestContext) -> ServiceResponse<Vec<String>> {
    let seasonal_bgs = seasonal_backgrounds::fetch_all_enabled(&ctx).await?;
    Ok(Json(seasonal_bgs.into_iter().map(|bg| bg.url).collect()))
}
