use crate::api::context::RequestContext;
use crate::common::error::AppError;
use crate::common::website;
use crate::models::osu::beatmaps::{DirectBeatmapsetArgs, DirectSearchArgs};
use crate::usecases::direct;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum::response::{IntoResponse, Response};

const ERROR_SEARCH_FAILED: &str = "-1\nFailed to retrieve search data -- please try again later";

pub async fn search(ctx: RequestContext, Query(args): Query<DirectSearchArgs>) -> Response {
    match direct::search(&ctx, args).await {
        Ok(search_view) => search_view.into_response(),
        Err(e @ (AppError::UsersNotFound | AppError::UsersInvalidCredentials)) => e.into_response(),
        Err(_) => ERROR_SEARCH_FAILED.into_response(),
    }
}

pub async fn search_set(ctx: RequestContext, Query(args): Query<DirectBeatmapsetArgs>) -> Response {
    match direct::fetch_beatmapset(&ctx, args).await {
        Ok(beatmapset) => beatmapset.into_response(),
        Err(e) => e.into_response(),
    }
}

/// Beatmapsets are downloaded from the beatmap mirror
pub async fn download(Path(beatmapset_id): Path<String>) -> Response {
    // The client asks for downloads without video with a trailing n, which the mirror does not offer
    let beatmapset_id = beatmapset_id.trim_end_matches('n');
    match beatmapset_id.parse::<i32>() {
        Ok(beatmapset_id) => {
            let download_link = website::get_beatmapset_download_link(beatmapset_id);
            (StatusCode::MOVED_PERMANENTLY, [(LOCATION, download_link)]).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
