mod health;
mod v1;
mod osu_web;
pub mod context;

use std::net::SocketAddr;
use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tracing::info;
use crate::common::state::AppState;
use crate::lifecycle;
use crate::settings::AppSettings;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/api/v1", v1::router())
        .nest("/web", osu_web::router())
}

pub async fn serve(settings: &AppSettings) -> anyhow::Result<()> {
    let state = lifecycle::initialize_state(&settings).await?;
    let addr = SocketAddr::from((settings.app_host, settings.app_port));
    info!("Listening on {addr}");
    let listener = TcpListener::bind(addr).await?;
    let app = Router::new()
        .merge(router())
        .route("/_health", get(health::health_check))
        .with_state(state);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
        .await?;
    Ok(())
}