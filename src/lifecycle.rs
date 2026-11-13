use std::sync::Arc;
use crate::achievements::Achievement;
use crate::common::amqp::{self, AmqpPublisher};
use crate::common::redis_pool::{RedisPool, RedisPoolManager};
use crate::common::state::AppState;
use crate::repositories::achievements;
use crate::settings::AppSettings;
use aws_sdk_s3::config::{Credentials, Region};
use deadpool::Runtime;
use redis::{AsyncConnectionConfig, Commands};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::time::Duration;
use tokio_util::task::TaskTracker;
use tracing::{info, warn};

/// How long background tasks are given to finish once the service shuts down
const BACKGROUND_TASKS_TIMEOUT: Duration = Duration::from_millis(7500);

pub fn initialize_logging(settings: &AppSettings) {
    tracing_subscriber::fmt()
        .with_max_level(settings.level)
        .json()
        .with_timer(tracing_subscriber::fmt::time())
        .with_level(true)
        .init();
}

pub async fn initialize_state(settings: &'static AppSettings) -> anyhow::Result<AppState> {
    let db = initialize_db(settings).await?;
    let redis = initialize_redis(settings)?;
    let amqp_publisher = initialize_amqp_publisher(settings).await?;
    let s3_client = initialize_s3(settings);
    let achievements = initialize_achievements(&db).await?;

    Ok(AppState {
        db,
        redis,
        s3_client,
        amqp_publisher,
        achievements,
        background_tasks: TaskTracker::new(),
    })
}

/// Resolves once the service has been asked to stop
pub async fn shutdown_signal() {
    let interrupt = async {
        _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut terminate) => _ = terminate.recv().await,
            Err(_) => std::future::pending().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = interrupt => {},
        _ = terminate => {},
    }
    info!("Shutting down");
}

/// Gives the tasks that are still running in the background the chance to finish
pub async fn finish_background_tasks(background_tasks: &TaskTracker) {
    background_tasks.close();
    let all_finished = tokio::time::timeout(BACKGROUND_TASKS_TIMEOUT, background_tasks.wait());
    if all_finished.await.is_err() {
        warn!(
            pending_tasks = background_tasks.len(),
            "Timed out waiting for background tasks to finish"
        );
    }
}

pub async fn initialize_amqp_publisher(settings: &AppSettings) -> anyhow::Result<Option<Arc<AmqpPublisher>>> {
    if settings.score_submission_routing_keys.is_empty() {
        return Ok(None);
    }

    let uri = amqp::broker_uri(&settings.amqp_user, &settings.amqp_pass, &settings.amqp_host, &settings.amqp_port)?;
    let publisher = AmqpPublisher::connect(uri).await?;
    Ok(Some(Arc::new(publisher)))
}

pub fn initialize_s3(settings: &'static AppSettings) -> Option<aws_sdk_s3::Client> {
    let aws_region = settings.aws_region.as_ref()?;
    let aws_access_key_id = settings.aws_access_key_id.as_ref()?;
    let aws_secret_access_key = settings.aws_secret_access_key.as_ref()?;
    let aws_endpoint_url = settings.aws_endpoint_url.as_ref()?;

    let region = Region::new(aws_region);
    let credentials = Credentials::new(
        aws_access_key_id,
        aws_secret_access_key,
        None,
        None,
        "static",
    );
    let config = aws_sdk_s3::config::Config::builder()
        .region(region)
        .credentials_provider(credentials)
        .endpoint_url(aws_endpoint_url)
        .force_path_style(true)
        .build();
    Some(aws_sdk_s3::Client::from_conf(config))
}

pub async fn initialize_achievements(db: &Pool<MySql>) -> anyhow::Result<&'static [Achievement]> {
    let achievements = achievements::fetch_all(db).await?;
    let achievements: Vec<_> = achievements
        .into_iter()
        .map(|achievement| Achievement::map_from(achievement))
        .collect();
    let achievements = achievements.into_boxed_slice();
    Ok(Box::leak(achievements))
}

pub fn initialize_db(settings: &AppSettings) -> impl Future<Output = sqlx::Result<Pool<MySql>>> {
    MySqlPoolOptions::new()
        .acquire_timeout(settings.db_wait_timeout)
        .max_connections(settings.db_max_connections as _)
        .connect(&settings.database_url)
}

pub fn initialize_redis(settings: &AppSettings) -> anyhow::Result<RedisPool> {
    let redis_client = redis::Client::open(settings.redis_url.as_str())?;
    let mut conn = redis_client.get_connection_with_timeout(settings.redis_wait_timeout)?;
    let _: () = conn.ping()?;
    let redis_cfg = AsyncConnectionConfig::new()
        .set_connection_timeout(Some(settings.redis_connection_timeout))
        .set_response_timeout(Some(settings.redis_response_timeout));

    let redis_manager = RedisPoolManager::new(redis_client, redis_cfg);
    let redis = RedisPool::builder(redis_manager)
        .max_size(settings.redis_max_connections)
        .wait_timeout(Some(settings.redis_wait_timeout))
        .runtime(Runtime::Tokio1)
        .build()?;
    Ok(redis)
}
