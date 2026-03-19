use crate::achievements::Achievement;
use crate::common::axum_ip::IpAddrInfo;
use crate::common::error::{AppError, ServiceResult};
use crate::common::redis_pool::RedisPool;
use crate::common::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sqlx::{MySql, Pool};

#[derive(Clone)]
pub struct RequestContext {
    pub db: Pool<MySql>,
    pub redis: RedisPool,
    pub s3_client: Option<aws_sdk_s3::Client>,
    pub achievements: &'static [Achievement],
    pub request_ip: IpAddrInfo,
}

impl FromRequestParts<AppState> for RequestContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> ServiceResult<Self> {
        let ip_info = IpAddrInfo::from_request_parts(parts, state).await?;
        Ok(Self {
            db: state.db.clone(),
            redis: state.redis.clone(),
            s3_client: state.s3_client.clone(),
            achievements: state.achievements,
            request_ip: ip_info,
        })
    }
}
