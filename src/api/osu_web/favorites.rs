use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::common::osu::join_lines;
use crate::usecases::favourites;
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FetchFavouritesArgs {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "h")]
    pub password_md5: String,
}

#[derive(Deserialize)]
pub struct AddFavouriteArgs {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "h")]
    pub password_md5: String,
    #[serde(rename = "a")]
    pub beatmapset_id: i32,
}

/// Responds with the ids of the favourite beatmapsets of the user, each on its own line
pub async fn fetch(
    ctx: RequestContext,
    Query(args): Query<FetchFavouritesArgs>,
) -> ServiceResult<String> {
    let beatmapset_ids = favourites::fetch_all(&ctx, &args.username, &args.password_md5).await?;
    Ok(join_lines(&beatmapset_ids))
}

/// The client displays the response to the user
pub async fn add(
    ctx: RequestContext,
    Query(args): Query<AddFavouriteArgs>,
) -> ServiceResult<&'static str> {
    let added = favourites::add(
        &ctx,
        &args.username,
        &args.password_md5,
        args.beatmapset_id,
    )
    .await?;
    match added {
        true => Ok("Added favourite!"),
        false => Ok("You've already favourited this beatmap!"),
    }
}
