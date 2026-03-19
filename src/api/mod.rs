pub mod context;
mod health;
mod osu_web;
mod v1;

use crate::common::state::AppState;
use crate::lifecycle;
use crate::settings::AppSettings;
use axum::Router;
use axum::routing::get;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/api/v1", v1::router())
        .nest("/web", osu_web::router())
}

pub async fn serve(settings: &'static AppSettings) -> anyhow::Result<()> {
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
