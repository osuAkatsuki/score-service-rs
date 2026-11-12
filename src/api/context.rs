use std::sync::Arc;
use crate::achievements::Achievement;
use crate::common::axum_ip::IpAddrInfo;
use crate::common::error::{AppError, ServiceResult};
use crate::common::redis_pool::RedisPool;
use crate::common::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sqlx::{MySql, Pool};
use tokio_util::task::TaskTracker;

#[derive(Clone)]
pub struct RequestContext {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
    pub amqp_channel: Option<Arc<lapin::Channel>>,
    pub s3_client: Option<aws_sdk_s3::Client>,
    pub achievements: &'static [Achievement],
    pub background_tasks: TaskTracker,
    pub request_ip: IpAddrInfo,
}

impl RequestContext {
    /// Runs a task the response of the request does not depend on.
    /// The service waits for these tasks to finish before it shuts down.
    pub fn spawn<F>(&self, task: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.background_tasks.spawn(task);
    }
}

impl FromRequestParts<AppState> for RequestContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> ServiceResult<Self> {
        let ip_info = IpAddrInfo::from_request_parts(parts, state).await?;
        Ok(Self {
            db: state.db.clone(),
            redis: state.redis.clone(),
            amqp_channel: state.amqp_channel.clone(),
            s3_client: state.s3_client.clone(),
            achievements: state.achievements,
            background_tasks: state.background_tasks.clone(),
            request_ip: ip_info,
        })
    }
}
