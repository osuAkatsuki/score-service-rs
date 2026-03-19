use crate::achievements::Achievement;
use crate::common::redis_pool::RedisPool;
use sqlx::{MySql, Pool};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
    pub achievements: &'static [Achievement],
}
