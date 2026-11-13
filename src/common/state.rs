use std::sync::Arc;
use crate::achievements::Achievement;
use crate::common::redis_pool::RedisPool;
use sqlx::{MySql, Pool};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
    pub amqp_channel: Option<Arc<lapin::Channel>>,
    pub s3_client: Option<aws_sdk_s3::Client>,
    pub achievements: &'static [Achievement],
}
