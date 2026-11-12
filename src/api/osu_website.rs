use crate::common::state::AppState;
use axum::Router;
use axum::extract::OriginalUri;
use axum::http::header::LOCATION;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};

/// The osu! client expects to find the osu! website where it finds this server.
/// What this server has no counterpart of is left to the official website.
const OSU_WEBSITE_URL: &str = "https://osu.ppy.sh";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/beatmapsets/{beatmapset_id}", get(redirect))
        .route("/beatmaps/{beatmap_id}", get(redirect))
        .route("/community/forums/topics/{topic_id}", get(redirect))
        .route("/difficulty-rating", post(redirect_difficulty_rating))
}

fn official_url(uri: &Uri) -> String {
    format!("{OSU_WEBSITE_URL}{}", uri.path())
}

pub async fn redirect(OriginalUri(uri): OriginalUri) -> Response {
    (StatusCode::MOVED_PERMANENTLY, [(LOCATION, official_url(&uri))]).into_response()
}

/// The beatmap is named in the body of the request, which only this kind of redirect preserves
pub async fn redirect_difficulty_rating(OriginalUri(uri): OriginalUri) -> Redirect {
    Redirect::temporary(&official_url(&uri))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_path_of_redirected_requests() {
        let uri = Uri::from_static("/web/maps/Camellia%20-%20GHOST%20(Akali)%20[Extra].osu");
        assert_eq!(
            official_url(&uri),
            "https://osu.ppy.sh/web/maps/Camellia%20-%20GHOST%20(Akali)%20[Extra].osu"
        );

        let uri = Uri::from_static("/beatmapsets/1357624?mode=osu");
        assert_eq!(official_url(&uri), "https://osu.ppy.sh/beatmapsets/1357624");
    }
}
