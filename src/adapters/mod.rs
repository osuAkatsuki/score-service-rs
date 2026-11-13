pub mod bancho_service;
pub mod beatmaps_service;
pub mod discord;
pub mod performance_service;
pub mod s3;

use std::time::Duration;

/// How long another service gets to answer, unless a request asks for more time
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);

/// A client that gives up on a service that does not answer, instead of waiting on it forever
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(DEFAULT_TIMEOUT)
        .build()
        .expect("Failed to build the HTTP client")
}
