pub mod pp;
pub mod status;

use crate::common::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/pp", get(pp::calculate))
        .route("/status", get(status::handler))
}
