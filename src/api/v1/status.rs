use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ServerStatus {
    pub server_status: u8,
}

/// Being able to answer at all is what tells that the server is up
pub async fn handler() -> Json<ServerStatus> {
    Json(ServerStatus { server_status: 1 })
}
