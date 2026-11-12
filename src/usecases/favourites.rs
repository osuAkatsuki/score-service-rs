use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::repositories::favourites;
use crate::usecases::users;

/// Adds a beatmapset to the favourites of the user.
/// Returns false if it already was one of their favourites.
pub async fn add(
    ctx: &RequestContext,
    username: &str,
    password_md5: &str,
    beatmapset_id: i32,
) -> ServiceResult<bool> {
    let user = users::fetch_one_and_auth(ctx, username, password_md5).await?;
    let added = favourites::create(ctx, user.user_id, beatmapset_id).await?;
    Ok(added)
}

/// Fetches the ids of all beatmapsets the user has favourited
pub async fn fetch_all(
    ctx: &RequestContext,
    username: &str,
    password_md5: &str,
) -> ServiceResult<Vec<i32>> {
    let user = users::fetch_one_and_auth(ctx, username, password_md5).await?;
    let beatmapset_ids = favourites::fetch_beatmapset_ids(ctx, user.user_id).await?;
    Ok(beatmapset_ids)
}
