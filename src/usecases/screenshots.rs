use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::usecases::users;
use axum::body::Bytes;

pub async fn upload(
    ctx: &RequestContext,
    username: &str,
    password_md5: &str,
    screenshot: Bytes,
) -> ServiceResult<()> {
    let user = users::fetch_one_and_auth(&ctx, &username, &password_md5).await?;
    // TODO: upload the screenshot
    Ok(())
}
