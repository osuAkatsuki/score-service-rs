use crate::api::context::RequestContext;
use crate::common::multipart::Multipart;
use crate::models::screenshot::ScreenshotForm;

pub async fn handler(
    ctx: RequestContext,
    form: Multipart<ScreenshotForm>,
) {
    println!("Got a screenshot form: {:?}", form);
}