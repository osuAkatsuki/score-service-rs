use crate::api::context::RequestContext;
use crate::common::error::{unexpected, ServiceResult};
use crate::models::seasonal_backgrounds::SeasonalBackground;
use crate::repositories::seasonal_backgrounds;

pub async fn fetch_all_enabled(ctx: &RequestContext) -> ServiceResult<Vec<SeasonalBackground>>  {
    match seasonal_backgrounds::fetch_all_enabled(ctx).await {
        Ok(seasonal_bgs) => Ok(seasonal_bgs),
        Err(e) => unexpected(e),
    }
}