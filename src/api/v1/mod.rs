use axum::Router;
use crate::common::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}