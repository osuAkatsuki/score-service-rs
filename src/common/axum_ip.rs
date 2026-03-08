use crate::common::error::{AppError, ServiceResult};
use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Clone)]
pub struct IpAddrInfo {
    pub ip_addr: IpAddr,
}

async fn get_ip_addr(parts: &mut Parts) -> ServiceResult<IpAddrInfo> {
    if let Some(ip) = parts.headers.get("CF-Connecting-IP") {
        let ip_addr = IpAddr::from_str(ip.to_str()?)?;
        Ok(IpAddrInfo { ip_addr })
    } else if let Some(ip) = parts.headers.get("X-Forwarded-For") {
        let ip_addr = IpAddr::from_str(ip.to_str()?)?;
        Ok(IpAddrInfo { ip_addr })
    } else {
        let info = <ConnectInfo<SocketAddr>>::from_request_parts(parts, &()).await?;
        let ip_addr = info.ip();
        Ok(IpAddrInfo { ip_addr })
    }
}

impl<S: Sync + Send> FromRequestParts<S> for IpAddrInfo {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> ServiceResult<Self> {
        get_ip_addr(parts).await
    }
}
