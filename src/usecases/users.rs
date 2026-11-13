use crate::adapters::discord;
use crate::api::context::RequestContext;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::common::website;
use crate::entities::mode::ModdedGamemode;
use crate::models::users::User;
use crate::repositories::{stats, users};
use std::collections::HashSet;
use tracing::{error, info};

/// Usernames are unique regardless of their casing and of spaces being written as underscores
fn make_safe_username(username: &str) -> String {
    username.trim_end().to_lowercase().replace(' ', "_")
}

/// Verifying a password against its hash is slow by design,
/// so it must not hold up the other requests in the meantime.
async fn verify_password(password: &str, password_hash: &str) -> ServiceResult<bool> {
    let task_password = password.to_owned();
    let task_password_hash = password_hash.to_owned();
    let is_correct = tokio::task::spawn_blocking(move || {
        bcrypt::verify(&task_password, &task_password_hash).unwrap_or(false)
    })
    .await?;

    Ok(is_correct)
}

pub async fn fetch_one_by_username(ctx: &RequestContext, username: &str) -> ServiceResult<User> {
    let username_safe = make_safe_username(username);
    match users::fetch_one_by_username_safe(ctx, &username_safe).await {
        Ok(user) => User::try_from(user),
        Err(sqlx::Error::RowNotFound) => Err(AppError::UsersNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_one_and_auth(
    ctx: &RequestContext,
    username: &str,
    password: &str,
) -> ServiceResult<User> {
    let username_safe = make_safe_username(username);
    let user = match users::fetch_one_by_username_safe(ctx, &username_safe).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(AppError::UsersNotFound),
        Err(e) => return unexpected(e),
    };

    if !verify_password(password, &user.password_md5).await? {
        return Err(AppError::UsersInvalidCredentials);
    }

    User::try_from(user)
}

pub async fn fetch_username(ctx: &RequestContext, user_id: i64) -> ServiceResult<String> {
    match users::fetch_username(ctx, user_id).await? {
        Some(username) => Ok(username),
        None => Err(AppError::UsersNotFound),
    }
}

pub async fn is_online(ctx: &RequestContext, user_id: i64) -> ServiceResult<bool> {
    let is_online = users::is_online(ctx, user_id).await?;
    Ok(is_online)
}

pub async fn update_latest_activity(ctx: &RequestContext, user_id: i64) -> ServiceResult<()> {
    users::update_latest_activity(ctx, user_id).await?;
    Ok(())
}

pub async fn restrict_user(ctx: &RequestContext, user_id: i64, reason: &str) -> ServiceResult<()> {
    // There is nothing left to do for users that are restricted already
    let newly_restricted = users::restrict(ctx, user_id).await?;
    if !newly_restricted {
        return Ok(());
    }

    // The note only documents the restriction, which has to be carried out with or without it
    let note = format!("score-service autorestrict: {reason}");
    if let Err(e) = users::add_note(ctx, user_id, &note).await {
        error!(user_id, reason, "Failed to note down the reason of the restriction: {e}");
    }
    users::create_restriction_log(ctx, user_id).await?;
    users::publish_ban_event(ctx, user_id).await?;

    let country = users::fetch_country(ctx, user_id).await?;
    stats::remove_from_leaderboards(ctx, user_id, &country).await?;
    info!(user_id, reason, "User has been restricted");

    let task_reason = reason.to_owned();
    ctx.spawn(async move {
        let profile_link = website::get_profile_link(user_id);
        _ = discord::send_ac_red_embed("User restricted", &task_reason, Some(&profile_link)).await;
    });
    Ok(())
}

/// What the staff gets to read about a client that submits scores without its anticheat token
const MISSING_ANTICHEAT_TOKEN: &str =
    "Submitted score without anticheat token (anticheat disabled, or a custom or outdated client).";

/// Lets the staff know that the client of the user leaves out its anticheat token,
/// with a note about the user and a message in the anticheat logs.
pub fn report_missing_anticheat_token(ctx: &RequestContext, user: &User) {
    let task_ctx = ctx.clone();
    let user_id = user.user_id;
    ctx.spawn(async move {
        // Without the note, nothing would keep the message from being sent for every score
        let note = format!("score-service: {MISSING_ANTICHEAT_TOKEN}");
        if let Err(e) = users::add_note(&task_ctx, user_id, &note).await {
            error!(user_id, "Failed to note down the missing anticheat token: {e}");
            return;
        }

        let profile_link = website::get_profile_link(user_id);
        _ = discord::send_ac_purple_embed(
            "Anticheat token missing",
            MISSING_ANTICHEAT_TOKEN,
            Some(&profile_link),
        )
        .await;
    });
}

pub async fn fetch_user_friends(
    ctx: &RequestContext,
    user_id: i64,
) -> ServiceResult<Vec<i32>> {
    let friends = users::fetch_user_friends(ctx, user_id).await?;
    Ok(friends)
}

/// Fetches the friends of the user the credentials belong to
pub async fn fetch_friends_and_auth(
    ctx: &RequestContext,
    username: &str,
    password: &str,
) -> ServiceResult<Vec<i32>> {
    let user = fetch_one_and_auth(ctx, username, password).await?;
    fetch_user_friends(ctx, user.user_id).await
}

pub async fn increment_beatmap_playcount(
    ctx: &RequestContext,
    user_id: i64,
    beatmap_md5: &str,
    mode: ModdedGamemode,
) -> ServiceResult<()> {
    users::increment_beatmap_playcount(ctx, user_id, beatmap_md5, mode).await?;
    Ok(())
}

pub async fn fetch_achievements(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> ServiceResult<HashSet<i32>> {
    let achievement_ids = users::fetch_achievements(ctx, user_id, mode).await?;
    Ok(achievement_ids.into_iter().collect())
}

pub async fn unlock_achievement(
    ctx: &RequestContext,
    achievement_id: i32,
    user_id: i64,
    mode: ModdedGamemode,
) -> ServiceResult<()> {
    users::unlock_achievement(ctx, achievement_id, user_id, mode).await?;
    Ok(())
}

pub async fn handle_pending_username_change(
    ctx: &RequestContext,
    user_id: i64,
) -> ServiceResult<()> {
    users::handle_pending_username_change(ctx, user_id).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_usernames_ignore_casing_spaces_and_the_supporter_flag() {
        assert_eq!(make_safe_username("Cookiezi"), "cookiezi");
        assert_eq!(make_safe_username("Lil Darkie "), "lil_darkie");
        assert_eq!(make_safe_username("[ Shigetora ]"), "[_shigetora_]");
    }

    #[test]
    fn the_note_about_a_missing_anticheat_token_is_stored_as_written() {
        // Latin1 replaces what it can not store, after which the note would not be found again
        assert!(MISSING_ANTICHEAT_TOKEN.is_ascii());
    }

    #[tokio::test]
    async fn verifies_passwords_against_their_hash() {
        let password = "5f4dcc3b5aa765d61d8327deb882cf99";
        let password_hash = bcrypt::hash(password, 4).unwrap();

        assert!(!verify_password("wrong", &password_hash).await.unwrap());
        assert!(verify_password(password, &password_hash).await.unwrap());
    }

    #[tokio::test]
    async fn rejects_passwords_of_users_with_a_malformed_hash() {
        assert!(!verify_password("password", "not a hash").await.unwrap());
    }
}
