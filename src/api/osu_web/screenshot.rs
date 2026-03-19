use crate::api::context::RequestContext;
use crate::common::multipart::Multipart;
use crate::models::screenshot::ScreenshotForm;
use crate::usecases::screenshots;

pub async fn handler(
    ctx: RequestContext,
    form: Multipart<ScreenshotForm>,
) -> Result<String, &'static str> {
    match screenshots::upload(&ctx, &form.0.u, &form.0.p, form.0.ss).await {
        Ok(screenshot) => Ok(format!("https://akatsuki.gg/ss/{}", 1)),
        _ => Err("https://akatsuki.gg/"),
    }
}
