use std::sync::Arc;
use crate::achievements::Achievement;
use crate::common::amqp::AmqpPublisher;
use crate::common::redis_pool::RedisPool;
use sqlx::{MySql, Pool};
use tokio_util::task::TaskTracker;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
    pub amqp_publisher: Option<Arc<AmqpPublisher>>,
    pub s3_client: Option<aws_sdk_s3::Client>,
    pub achievements: &'static [Achievement],
    pub background_tasks: TaskTracker,
}
