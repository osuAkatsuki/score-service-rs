pub mod context;
mod health;
mod osu_web;
mod osu_website;
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
        .nest("/public/api/v1", v1::public_router())
        .nest("/web", osu_web::router())
        .route("/ss/{file_name}", get(osu_web::screenshot::fetch))
        .route("/d/{beatmapset_id}", get(osu_web::direct::download))
        .merge(osu_website::router())
}

pub async fn serve(settings: &'static AppSettings) -> anyhow::Result<()> {
    let state = lifecycle::initialize_state(&settings).await?;
    let background_tasks = state.background_tasks.clone();
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
    .with_graceful_shutdown(lifecycle::shutdown_signal())
    .await?;

    lifecycle::finish_background_tasks(&background_tasks).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_do_not_conflict() {
        // Routes that overlap are only noticed once the router is being built
        let _ = router();
    }
}
