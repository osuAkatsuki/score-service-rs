pub mod pp;

use crate::common::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn router() -> Router<AppState> {
    Router::new().route("/pp", get(pp::calculate))
}
