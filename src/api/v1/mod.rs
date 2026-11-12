pub mod aggregate_score_stats;
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

pub fn public_router() -> Router<AppState> {
    Router::new().route(
        "/aggregate-score-stats/total-scores-set",
        get(aggregate_score_stats::total_scores_set),
    )
}
