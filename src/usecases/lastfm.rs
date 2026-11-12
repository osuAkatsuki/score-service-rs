use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::models::osu::lastfm::LastFmFlags;
use crate::repositories::lastfm_flags;
use crate::usecases::users;
use tracing::warn;

/// Keeps a record of the flags the anticheat of the osu! client has raised for the user
pub async fn report_flags(
    ctx: &RequestContext,
    username: &str,
    password_md5: &str,
    flags: LastFmFlags,
) -> ServiceResult<()> {
    let user = users::fetch_one_and_auth(ctx, username, password_md5).await?;

    let explanation = flags.explain();
    lastfm_flags::create(ctx, user.user_id, flags.bits(), &explanation).await?;

    warn!(
        user_id = user.user_id,
        flags = ?flags,
        "User has been flagged by their client:\n{explanation}"
    );
    Ok(())
}
