use crate::api::context::RequestContext;
use crate::entities::mode::ModdedGamemode;
use crate::entities::users::{User, UsernameChange};
use crate::models::privileges::Privileges;
use crate::settings::AppSettings;
use chrono::Utc;
use redis::AsyncCommands;

pub async fn fetch_one_by_username_safe(
    ctx: &RequestContext,
    username_safe: &str,
) -> sqlx::Result<User> {
    const QUERY: &str = r#"
SELECT
    id, username, username_safe, password_md5, email,
    register_datetime, latest_activity, silence_end, silence_reason,
    privileges, donor_expire, frozen, notes, ban_datetime,
    previous_overwrite, whitelist, clan_id, userpage_content,
    userpage_allowed, freeze_reason, country, username_aka,
    can_custom_badge, show_custom_badge, custom_badge_icon, custom_badge_name,
    favourite_mode, play_style, vanilla_pp_leaderboards, leaderboard_size, has_free_username_change
FROM users
WHERE username_safe = ?"#;
    sqlx::query_as(QUERY)
        .bind(username_safe)
        .fetch_one(&ctx.db)
        .await
}

pub async fn update_latest_activity(ctx: &RequestContext, user_id: i64) -> sqlx::Result<()> {
    sqlx::query("UPDATE users SET latest_activity = UNIX_TIMESTAMP() WHERE id = ?")
        .bind(user_id)
        .execute(&ctx.db)
        .await?;
    Ok(())
}

/// Restricts the user unless they already are.
/// Returns whether the user has been restricted by this call.
pub async fn restrict_with_note(
    ctx: &RequestContext,
    user_id: i64,
    note: &str,
) -> sqlx::Result<bool> {
    const QUERY: &str = r#"UPDATE users
    SET privileges = (privileges & ~(?)), ban_datetime = UNIX_TIMESTAMP(),
    notes = CONCAT(IFNULL(notes, ''), ?)
    WHERE id = ? AND (privileges & ?) != 0"#;
    let privileges = Privileges::PubliclyVisible;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S");
    let notes_adt = format!("\n[{now} UTC] score-service autorestrict: {note}");
    let result = sqlx::query(QUERY)
        .bind(privileges.bits())
        .bind(notes_adt)
        .bind(user_id)
        .bind(privileges.bits())
        .execute(&ctx.db)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Adds the restriction to the audit log of the admin panel, attributed to the bot
pub async fn create_restriction_log(ctx: &RequestContext, user_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = r#"INSERT INTO rap_logs (userid, text, datetime, through)
    VALUES (?, ?, UNIX_TIMESTAMP(), ?)"#;
    let settings = AppSettings::get();
    sqlx::query(QUERY)
        .bind(settings.bot_user_id)
        .bind(format!("restricted user {user_id}"))
        .bind("score-service")
        .execute(&ctx.db)
        .await?;
    Ok(())
}

pub async fn fetch_username(ctx: &RequestContext, user_id: i64) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&ctx.db)
        .await
}

pub async fn fetch_country(ctx: &RequestContext, user_id: i64) -> sqlx::Result<String> {
    sqlx::query_scalar("SELECT country FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&ctx.db)
        .await
}

pub async fn publish_ban_event(ctx: &RequestContext, user_id: i64) -> anyhow::Result<()> {
    let mut redis = ctx.redis.get().await?;
    let _: () = redis.publish("peppy:ban", user_id.to_string()).await?;
    Ok(())
}

pub async fn fetch_user_friends(
    ctx: &RequestContext,
    user_id: i64,
) -> sqlx::Result<Vec<i32>> {
    sqlx::query_scalar("SELECT user2 FROM users_relationships WHERE user1 = ?")
        .bind(user_id)
        .fetch_all(&ctx.db)
        .await
}

pub async fn increment_beatmap_playcount(
    ctx: &RequestContext,
    user_id: i64,
    beatmap_md5: &str,
    mode: ModdedGamemode,
) -> sqlx::Result<()> {
    const QUERY: &str = r#"
    INSERT INTO user_beatmaps (userid, map, rx, mode, count)
        VALUES (?, ?, ?, ?, 1)
        ON DUPLICATE KEY UPDATE count = count + 1"#;
    sqlx::query(QUERY)
        .bind(user_id)
        .bind(beatmap_md5)
        .bind(mode.mode_mod() as u8)
        .bind(mode.as_gamemode() as u8)
        .execute(&ctx.db)
        .await?;
    Ok(())
}

pub async fn fetch_achievements(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> sqlx::Result<Vec<i32>> {
    sqlx::query_scalar(
        "SELECT achievement_id FROM users_achievements WHERE user_id = ? AND mode = ?",
    )
    .bind(user_id)
    .bind(mode as u8)
    .fetch_all(&ctx.db)
    .await
}

pub async fn unlock_achievement(
    ctx: &RequestContext,
    achievement_id: i32,
    user_id: i64,
    mode: ModdedGamemode,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"INSERT IGNORE INTO users_achievements (achievement_id, user_id, mode, created_at)
       VALUES (?, ?, ?, UNIX_TIMESTAMP())"#,
    )
    .bind(achievement_id)
    .bind(user_id)
    .bind(mode as u8)
    .execute(&ctx.db)
    .await?;
    Ok(())
}

fn make_pending_username_change_key(user_id: i64) -> String {
    format!("ripple:change_username_pending:{user_id}")
}

const USERNAME_CHANGE_KEY: &str = "peppy:change_username";
const API_USERNAME_CHANGE_KEY: &str = "api:change_username";

pub async fn handle_pending_username_change(
    ctx: &RequestContext,
    user_id: i64,
) -> anyhow::Result<()> {
    let key = make_pending_username_change_key(user_id);
    let mut redis = ctx.redis.get().await?;
    let new_username: Option<String> = redis.get(key).await?;
    let Some(new_username) = new_username else {
        return Ok(());
    };

    let username_change_payload = &UsernameChange {
        user_id,
        new_username,
    };
    let json_encoded = serde_json::to_string(&username_change_payload)?;
    let _: () = redis.publish(USERNAME_CHANGE_KEY, json_encoded).await?;
    let _: () = redis
        .publish(API_USERNAME_CHANGE_KEY, user_id.to_string())
        .await?;
    Ok(())
}
