use crate::common::env::FromEnv;
use std::env;
use std::net::IpAddr;
use std::ops::Deref;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::Level;

pub struct AppSettings {
    pub app_env: String,
    pub app_component: String,
    pub level: Level,
    pub app_host: IpAddr,
    pub app_port: u16,

    pub database_url: String,
    pub db_max_connections: usize,
    pub db_wait_timeout: Duration,

    pub redis_url: String,
    pub redis_max_connections: usize,
    pub redis_connection_timeout: Duration,
    pub redis_response_timeout: Duration,
    pub redis_wait_timeout: Duration,

    pub amqp_user: String,
    pub amqp_pass: String,
    pub amqp_host: String,
    pub amqp_port: String,
    pub score_submission_routing_keys: Vec<String>,

    pub aws_region: Option<String>,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub aws_endpoint_url: Option<String>,
    pub aws_bucket_name: Option<String>,

    pub leaderboard_size: i32,
    pub app_ci_key: String,
    pub bancho_service_base_url: String,
    pub beatmaps_service_base_url: String,
    pub performance_service_base_url: String,

    pub frontend_base_url: String,
    pub discord_ac_logs_webhook_url: Option<String>,
}

impl AppSettings {
    pub fn load_from_env() -> anyhow::Result<Self> {
        let _ = dotenv::dotenv();

        let app_env = env::var("APP_ENV")?;
        let app_component = env::var("APP_COMPONENT")?;
        let level = Level::from_env("LOG_LEVEL")?;
        let app_host = IpAddr::from_env("APP_HOST")?;
        let app_port = u16::from_env("APP_PORT")?;

        let database_url = env::var("DATABASE_URL")?;
        let db_max_connections = usize::from_env("DB_MAX_CONNECTIONS")?;
        let db_wait_timeout_secs = u64::from_env("DB_WAIT_TIMEOUT_SECS")?;
        let db_wait_timeout = Duration::from_secs(db_wait_timeout_secs);

        let redis_url = env::var("REDIS_URL")?;
        let redis_max_connections = usize::from_env("REDIS_MAX_CONNECTIONS")?;
        let redis_connection_timeout_secs = u64::from_env("REDIS_CONNECTION_TIMEOUT_SECS")?;
        let redis_connection_timeout = Duration::from_secs(redis_connection_timeout_secs);
        let redis_response_timeout_secs = u64::from_env("REDIS_RESPONSE_TIMEOUT_SECS")?;
        let redis_response_timeout = Duration::from_secs(redis_response_timeout_secs);
        let redis_wait_timeout_secs = u64::from_env("REDIS_WAIT_TIMEOUT_SECS")?;
        let redis_wait_timeout = Duration::from_secs(redis_wait_timeout_secs);

        let amqp_user = env::var("AMQP_USER")?;
        let amqp_pass = env::var("AMQP_PASS")?;
        let amqp_host = env::var("AMQP_HOST")?;
        let amqp_port = env::var("AMQP_PORT")?;
        let score_submission_routing_keys = env::var("SCORE_SUBMISSION_ROUTING_KEYS")?;
        let score_submission_routing_keys = score_submission_routing_keys
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
            .collect();

        let aws_region = env::var("AWS_REGION").ok();
        let aws_access_key_id = env::var("AWS_ACCESS_KEY_ID").ok();
        let aws_secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").ok();
        let aws_endpoint_url = env::var("AWS_ENDPOINT_URL").ok();
        let aws_bucket_name = env::var("AWS_BUCKET_NAME").ok();

        let leaderboard_size = i32::from_env("LEADERBOARD_SIZE").unwrap_or(100);
        let app_ci_key = env::var("APP_CI_KEY")?;
        let bancho_service_base_url = env::var("BANCHO_SERVICE_BASE_URL")?;
        let beatmaps_service_base_url = env::var("BEATMAPS_SERVICE_BASE_URL")?;
        let performance_service_base_url = env::var("PERFORMANCE_SERVICE_BASE_URL")?;
        let frontend_base_url = env::var("FRONTEND_URL")?;
        let discord_ac_logs_webhook_url = env::var("DISCORD_AC_LOGS_WEBHOOK_URL")
            .ok()
            .filter(|url| !url.trim().is_empty());

        Ok(AppSettings {
            app_env,
            app_component,
            level,
            app_port,
            app_host,

            database_url,
            db_max_connections,
            db_wait_timeout,

            redis_url,
            redis_max_connections,
            redis_connection_timeout,
            redis_response_timeout,
            redis_wait_timeout,

            amqp_user,
            amqp_pass,
            amqp_host,
            amqp_port,
            score_submission_routing_keys,

            aws_region,
            aws_access_key_id,
            aws_secret_access_key,
            aws_endpoint_url,
            aws_bucket_name,

            leaderboard_size,
            app_ci_key,
            bancho_service_base_url,
            beatmaps_service_base_url,
            performance_service_base_url,
            frontend_base_url,
            discord_ac_logs_webhook_url,
        })
    }

    pub fn get() -> &'static AppSettings {
        settings()
    }
}

pub fn settings() -> &'static AppSettings {
    static SETTINGS: LazyLock<AppSettings> =
        LazyLock::new(|| AppSettings::load_from_env().expect("Failed to load settings"));
    SETTINGS.deref()
}
