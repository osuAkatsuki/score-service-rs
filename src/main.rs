use score_service::settings::AppSettings;
use score_service::{api, lifecycle};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = AppSettings::get();
    lifecycle::initialize_logging(&settings);

    match settings.app_component.as_str() {
        "api" => api::serve(settings).await,
        _ => panic!("Unknown app component"),
    }
}
