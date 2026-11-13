use std::sync::Arc;
use crate::achievements::Achievement;
use crate::common::amqp::AmqpPublisher;
use crate::common::error::{AppError, ServiceResult};
use crate::common::redis_pool::RedisPool;
use crate::common::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::header::USER_AGENT;
use axum::http::request::Parts;
use sqlx::{MySql, Pool};
use tokio_util::task::TaskTracker;

#[derive(Clone)]
pub struct RequestContext {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
    pub amqp_publisher: Option<Arc<AmqpPublisher>>,
    pub s3_client: Option<aws_sdk_s3::Client>,
    pub achievements: &'static [Achievement],
    pub background_tasks: TaskTracker,
    pub user_agent: Option<String>,
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
        let user_agent = parts
            .headers
            .get(USER_AGENT)
            .and_then(|user_agent| user_agent.to_str().ok())
            .map(str::to_owned);
        Ok(Self {
            db: state.db.clone(),
            redis: state.redis.clone(),
            amqp_publisher: state.amqp_publisher.clone(),
            s3_client: state.s3_client.clone(),
            achievements: state.achievements,
            background_tasks: state.background_tasks.clone(),
            user_agent,
        })
    }
}
