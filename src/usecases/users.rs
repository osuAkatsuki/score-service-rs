use crate::api::context::RequestContext;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::entities::mode::ModdedGamemode;
use crate::models::users::User;
use crate::repositories::users;
use std::collections::HashSet;

pub async fn fetch_one_by_username(ctx: &RequestContext, username: &str) -> ServiceResult<User> {
    match users::fetch_one_by_username(ctx, username).await {
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
    let user = match users::fetch_one_by_username(ctx, username).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(AppError::UsersNotFound),
        Err(e) => return unexpected(e),
    };

    if !bcrypt::verify(&password, &user.password_md5)
        .map_err(|_| AppError::UsersInvalidCredentials)?
    {
        return Err(AppError::UsersInvalidCredentials);
    }

    User::try_from(user)
}

pub async fn update_latest_activity(ctx: &RequestContext, user_id: i64) -> ServiceResult<()> {
    users::update_latest_activity(ctx, user_id).await?;
    Ok(())
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
