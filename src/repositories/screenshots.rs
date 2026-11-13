use crate::api::context::RequestContext;
use redis::{AsyncCommands, ExistenceCheck, SetExpiry, SetOptions};
use std::net::IpAddr;

/// The amount of seconds that have to pass between two screenshots
const UPLOAD_COOLDOWN_SECS: u64 = 10;

fn make_cooldown_key(ip_addr: IpAddr) -> String {
    format!("less:ss_limit:{ip_addr}")
}

/// Starts the cooldown between two screenshot uploads for the ip address.
/// Returns false if there already is a cooldown in progress.
pub async fn start_upload_cooldown(ctx: &RequestContext, ip_addr: IpAddr) -> anyhow::Result<bool> {
    let key = make_cooldown_key(ip_addr);
    let options = SetOptions::default()
        .conditional_set(ExistenceCheck::NX)
        .with_expiration(SetExpiry::EX(UPLOAD_COOLDOWN_SECS));

    let mut redis = ctx.redis.get().await?;
    let started: Option<String> = redis.set_options(key, 1, options).await?;
    Ok(started.is_some())
}
