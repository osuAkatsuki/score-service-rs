use crate::achievements::Achievement;
use crate::common::redis_pool::{RedisPool, RedisPoolManager};
use crate::common::state::AppState;
use crate::repositories::achievements;
use crate::settings::AppSettings;
use deadpool::Runtime;
use redis::{AsyncConnectionConfig, Commands};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};

pub fn initialize_logging(settings: &AppSettings) {
    tracing_subscriber::fmt()
        .with_max_level(settings.level)
        .json()
        .with_timer(tracing_subscriber::fmt::time())
        .with_level(true)
        .init();
}

pub async fn initialize_state(settings: &AppSettings) -> anyhow::Result<AppState> {
    let db = initialize_db(&settings).await?;
    let redis = initialize_redis(&settings)?;
    let achievements = initialize_achievements(&db).await?;

    Ok(AppState {
        db,
        redis,
        achievements,
    })
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
