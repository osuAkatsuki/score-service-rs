use crate::api::context::RequestContext;
use crate::common::multipart::Multipart;
use crate::models::screenshot::ScreenshotForm;
use crate::usecases::screenshots;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// The client opens whatever it receives as the name of a screenshot
pub async fn handler(
    ctx: RequestContext,
    form: Multipart<ScreenshotForm>,
) -> Result<String, &'static str> {
    match screenshots::upload(&ctx, &form.0.u, &form.0.p, form.0.ss).await {
        Ok(file_name) => Ok(file_name),
        _ => Err("https://akatsuki.gg/"),
    }
}

pub async fn fetch(ctx: RequestContext, Path(file_name): Path<String>) -> Response {
    match screenshots::fetch(&ctx, &file_name).await {
        Ok(Some(screenshot)) => screenshot.into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => e.into_response(),
    }
}
