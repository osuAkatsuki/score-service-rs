use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sqlx::{MySql, Pool};
use crate::common::axum_ip::IpAddrInfo;
use crate::common::error::{AppError, ServiceResult};
use crate::common::redis_pool::RedisPool;
use crate::common::state::AppState;

#[derive(Clone)]
pub struct RequestContext {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
    pub request_ip: IpAddrInfo,
}

impl FromRequestParts<AppState> for RequestContext {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> ServiceResult<Self> {
        let ip_info = IpAddrInfo::from_request_parts(parts, state).await?;
        Ok(Self {
            db: state.db.clone(),
            redis: state.redis.clone(),
            request_ip: ip_info,
        })
    }
}