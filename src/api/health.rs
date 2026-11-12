use crate::common::error::ServiceResponse;
use crate::common::state::AppState;
use axum::Json;
use axum::extract::State;
use redis::AsyncCommands;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
}

/// The service is only healthy while it can reach its data stores
pub async fn health_check(State(state): State<AppState>) -> ServiceResponse<HealthStatus> {
    let mut redis = state.redis.get().await?;
    let _: () = redis.ping().await?;
    sqlx::query("SELECT 1").execute(&state.db).await?;

    Ok(Json(HealthStatus { status: "ok" }))
}
