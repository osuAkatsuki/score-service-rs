pub mod bancho_connect;
pub mod check_updates;
pub mod direct;
pub mod favorites;
pub mod get_beatmap_info;
pub mod get_friends;
pub mod get_replay;
pub mod lastfm;
pub mod mark_as_read;
pub mod post_comment;
pub mod rate_map;
pub mod scores;
pub mod screenshot;
pub mod seasonal_backgrounds;

use crate::common::state::AppState;
use axum::Router;
use axum::routing::{get, post};

pub fn router() -> Router<AppState> {
    /*
        Unhandled endpoints:
        POST /web/osu-error.php
        POST /web/osu-session.php
        POST /web/osu-osz2-bmsubmit-post.php
        POST /web/osu-osz2-bmsubmit-upload.php
        GET /web/osu-osz2-bmsubmit-getid.php
        GET /web/osu-get-beatmap-topic.php
    */

    Router::new()
        .route("/osu-screenshot.php", post(screenshot::handler))
        .route("/check-updates.php", get(check_updates::handler))
        .route("/bancho_connect.php", get(bancho_connect::handler))
        .route("/osu-getseasonal.php", get(seasonal_backgrounds::handler))
        .route("/osu-getfriends.php", get(get_friends::handler))
        .route("/osu-markasread.php", get(mark_as_read::handler))
        .route("/lastfm.php", get(lastfm::handler))
        .route("/osu-search.php", get(direct::search))
        .route("/osu-search-set.php", get(direct::search_set))
        .route("/osu-getbeatmapinfo.php", post(get_beatmap_info::handler))
        .route("/osu-rate.php", get(rate_map::handler))
        .route("/osu-getfavourites.php", get(favorites::fetch))
        .route("/osu-addfavourite.php", get(favorites::add))
        .route("/osu-getreplay.php", get(get_replay::handler))
        .route("/osu-comment.php", post(post_comment::handler))
        .route("/osu-osz2-getscores.php", get(scores::fetch))
        .route("/osu-submit-modular-selector.php", post(scores::submit))
}
