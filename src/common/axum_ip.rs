use crate::common::error::{AppError, ServiceResult};
use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use std::net::IpAddr;
use tracing::warn;

/// The reverse proxy in front of the service puts the address of the client into this header
const CLIENT_IP_HEADER: &str = "X-Real-IP";

/// The address of the client a request came from.
/// Only routes that ask for it depend on the reverse proxy to pass it on.
#[derive(Clone)]
pub struct IpAddrInfo {
    pub ip_addr: IpAddr,
}

fn client_ip_addr(headers: &HeaderMap) -> Option<IpAddr> {
    let ip_addr = headers.get(CLIENT_IP_HEADER)?.to_str().ok()?;
    ip_addr.trim().parse().ok()
}

impl<S: Sync + Send> FromRequestParts<S> for IpAddrInfo {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> ServiceResult<Self> {
        match client_ip_addr(&parts.headers) {
            Some(ip_addr) => Ok(IpAddrInfo { ip_addr }),
            None => {
                warn!("Request without a valid {CLIENT_IP_HEADER} header, is the reverse proxy setting it?");
                Err(AppError::MalformedRequest)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, Request, StatusCode};

    fn headers(name: &'static str, value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(name, HeaderValue::from_static(value));
        headers
    }

    #[test]
    fn reads_the_address_the_reverse_proxy_passes_on() {
        let ip_addr = client_ip_addr(&headers("x-real-ip", "203.0.113.7")).unwrap();
        assert_eq!(ip_addr.to_string(), "203.0.113.7");

        let ip_addr = client_ip_addr(&headers("x-real-ip", "2001:db8::7")).unwrap();
        assert_eq!(ip_addr.to_string(), "2001:db8::7");
    }

    #[test]
    fn does_not_know_the_address_without_the_header() {
        assert!(client_ip_addr(&HeaderMap::new()).is_none());
    }

    #[test]
    fn ignores_the_other_headers_that_carry_addresses() {
        assert!(client_ip_addr(&headers("x-forwarded-for", "203.0.113.7")).is_none());
        assert!(client_ip_addr(&headers("cf-connecting-ip", "203.0.113.7")).is_none());
    }

    #[tokio::test]
    async fn rejects_requests_that_came_without_the_address_of_their_client() {
        let (mut parts, _) = Request::new(()).into_parts();
        let rejection = IpAddrInfo::from_request_parts(&mut parts, &()).await.err();
        assert!(matches!(rejection, Some(AppError::MalformedRequest)));
        assert_eq!(AppError::MalformedRequest.http_status_code(), StatusCode::BAD_REQUEST);

        parts.headers = headers("x-real-ip", "203.0.113.7");
        let client = IpAddrInfo::from_request_parts(&mut parts, &()).await.ok();
        assert_eq!(client.unwrap().ip_addr.to_string(), "203.0.113.7");
    }

    #[test]
    fn rejects_values_that_are_not_an_address() {
        assert!(client_ip_addr(&headers("x-real-ip", "")).is_none());
        assert!(client_ip_addr(&headers("x-real-ip", "unknown")).is_none());
        assert!(client_ip_addr(&headers("x-real-ip", "203.0.113.7, 198.51.100.2")).is_none());
    }
}
