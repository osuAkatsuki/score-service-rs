use crate::common::error::{AppError, ServiceResult};
use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Clone)]
pub struct IpAddrInfo {
    pub ip_addr: IpAddr,
}

/// Every proxy a request passes adds the address it got the request from to the end of the header,
/// which leaves the address of the client at the front.
fn parse_forwarded_for(forwarded_for: &str) -> Result<IpAddr, AddrParseError> {
    let client = forwarded_for.split(',').next().unwrap_or_default();
    IpAddr::from_str(client.trim())
}

async fn get_ip_addr(parts: &mut Parts) -> ServiceResult<IpAddrInfo> {
    if let Some(ip) = parts.headers.get("X-Real-IP") {
        let ip_addr = IpAddr::from_str(ip.to_str()?.trim())?;
        Ok(IpAddrInfo { ip_addr })
    } else if let Some(ip) = parts.headers.get("CF-Connecting-IP") {
        let ip_addr = IpAddr::from_str(ip.to_str()?)?;
        Ok(IpAddrInfo { ip_addr })
    } else if let Some(forwarded_for) = parts.headers.get("X-Forwarded-For") {
        let ip_addr = parse_forwarded_for(forwarded_for.to_str()?)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_address_of_a_client_behind_one_proxy() {
        let ip_addr = parse_forwarded_for("203.0.113.7").unwrap();
        assert_eq!(ip_addr.to_string(), "203.0.113.7");
    }

    #[test]
    fn reads_the_address_of_a_client_behind_several_proxies() {
        let ip_addr = parse_forwarded_for("203.0.113.7, 198.51.100.2,10.0.0.1").unwrap();
        assert_eq!(ip_addr.to_string(), "203.0.113.7");

        let ip_addr = parse_forwarded_for("2001:db8::7, 198.51.100.2").unwrap();
        assert_eq!(ip_addr.to_string(), "2001:db8::7");
    }

    #[test]
    fn rejects_headers_that_do_not_start_with_an_address() {
        assert!(parse_forwarded_for("").is_err());
        assert!(parse_forwarded_for("unknown, 198.51.100.2").is_err());
    }
}
