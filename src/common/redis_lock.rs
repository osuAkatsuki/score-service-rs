use crate::api::context::RequestContext;
use crate::common::redis_pool::RedisPool;
use redis::{AsyncCommands, ExistenceCheck, RedisResult, SetExpiry, SetOptions};
use std::time::Duration;
use tokio_util::task::TaskTracker;

/// Locks expire on their own, in case their holder never gets to release them
const LOCK_EXPIRY_SECS: u64 = 10;
const RETRY_DELAY: Duration = Duration::from_millis(50);

/// A lock that is shared between all instances of the service.
/// It is released as soon as it is dropped.
pub struct RedisLock {
    key: String,
    redis: RedisPool,
    background_tasks: TaskTracker,
}

impl RedisLock {
    /// Waits for the lock with the given key to be free and takes it
    pub async fn acquire(ctx: &RequestContext, key: String) -> anyhow::Result<Self> {
        let mut redis = ctx.redis.get().await?;
        loop {
            let options = SetOptions::default()
                .conditional_set(ExistenceCheck::NX)
                .with_expiration(SetExpiry::EX(LOCK_EXPIRY_SECS));
            let acquired: Option<String> = redis.set_options(key.as_str(), 1, options).await?;
            if acquired.is_some() {
                break;
            }

            tokio::time::sleep(RETRY_DELAY).await;
        }

        Ok(Self {
            key,
            redis: ctx.redis.clone(),
            background_tasks: ctx.background_tasks.clone(),
        })
    }
}

impl Drop for RedisLock {
    fn drop(&mut self) {
        let key = std::mem::take(&mut self.key);
        let redis = self.redis.clone();
        self.background_tasks.spawn(async move {
            if let Ok(mut redis) = redis.get().await {
                let _: RedisResult<()> = redis.del(key).await;
            }
        });
    }
}
