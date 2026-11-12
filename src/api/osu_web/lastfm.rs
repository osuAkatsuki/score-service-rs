use crate::api::context::RequestContext;
use crate::models::osu::lastfm::LastFmFlags;
use crate::usecases::lastfm;
use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LastFmArgs {
    #[serde(rename = "us")]
    pub username: String,
    #[serde(rename = "ha")]
    pub password_md5: String,
    /// Either the id of the beatmap the user is listening to or the flags of their anticheat
    #[serde(rename = "b")]
    pub beatmap_id_or_flags: String,
}

/// Tells the client that there is no need to send anything else for now
const RESPONSE_STOP_SENDING: &str = "-3";

/// Songs are not scrobbled, the client only gets to report its anticheat flags through this
pub async fn handler(ctx: RequestContext, Query(args): Query<LastFmArgs>) -> Response {
    let Some(flags) = LastFmFlags::from_report(&args.beatmap_id_or_flags) else {
        return RESPONSE_STOP_SENDING.into_response();
    };

    match lastfm::report_flags(&ctx, &args.username, &args.password_md5, flags).await {
        Ok(()) => RESPONSE_STOP_SENDING.into_response(),
        Err(e) => e.into_response(),
    }
}
