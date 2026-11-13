use crate::adapters::s3;
use crate::api::context::RequestContext;
use crate::common::error::{AppError, ServiceResult};
use crate::models::screenshot::{Screenshot, ScreenshotFormat};
use crate::repositories::screenshots;
use crate::usecases::users;
use axum::body::Bytes;
use rand::distr::{Alphanumeric, SampleString};
use std::net::IpAddr;
use tracing::{info, warn};

const SCREENSHOTS_FOLDER: &str = "screenshots";
const MAX_SCREENSHOT_SIZE: usize = 500_000;
const FILE_NAME_LENGTH: usize = 8;

/// Stores a screenshot taken by the osu! client and returns the name of its file
pub async fn upload(
    ctx: &RequestContext,
    client_ip_addr: IpAddr,
    username: &str,
    password_md5: &str,
    screenshot: Bytes,
) -> ServiceResult<String> {
    let user = users::fetch_one_and_auth(ctx, username, password_md5).await?;

    // Screenshots are taken ingame
    if !users::is_online(ctx, user.user_id).await? {
        warn!(
            user_id = user.user_id,
            "Tried to upload a screenshot while offline"
        );
        return Err(AppError::ScreenshotsNotAllowed);
    }

    if ctx.user_agent.as_deref() != Some("osu!") {
        warn!(
            user_id = user.user_id,
            "Tried to upload a screenshot without the osu! client"
        );
        return Err(AppError::ScreenshotsNotAllowed);
    }

    if !screenshots::start_upload_cooldown(ctx, client_ip_addr).await? {
        warn!(
            user_id = user.user_id,
            "Tried to upload screenshots too quickly"
        );
        return Err(AppError::ScreenshotsTooManyRequests);
    }

    if screenshot.len() > MAX_SCREENSHOT_SIZE {
        return Err(AppError::ScreenshotsInvalidFile);
    }

    let Some(format) = ScreenshotFormat::detect(&screenshot) else {
        warn!(
            user_id = user.user_id,
            "Tried to upload a screenshot of an unknown format"
        );
        return Err(AppError::ScreenshotsInvalidFile);
    };

    let random_name = Alphanumeric.sample_string(&mut rand::rng(), FILE_NAME_LENGTH);
    let file_name = format!("{random_name}.{}", format.extension());
    s3::upload(ctx, &file_name, SCREENSHOTS_FOLDER, screenshot).await?;

    info!(user_id = user.user_id, file_name, "Uploaded a screenshot");
    Ok(file_name)
}

pub async fn fetch(ctx: &RequestContext, file_name: &str) -> ServiceResult<Option<Screenshot>> {
    // Screenshots are stored flat, anything else is an attempt to leave their folder
    if file_name.contains("..") || file_name.contains(['/', '\\']) {
        return Ok(None);
    }

    let Some(contents) = s3::download(ctx, file_name, SCREENSHOTS_FOLDER).await? else {
        return Ok(None);
    };

    let format = ScreenshotFormat::detect(&contents).unwrap_or(ScreenshotFormat::Png);
    Ok(Some(Screenshot { format, contents }))
}
