use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::common::osu::join_lines;
use crate::usecases::users;
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetFriendsArgs {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "h")]
    pub password_md5: String,
}

/// Responds with the ids of the friends of the user, each on its own line
pub async fn handler(
    ctx: RequestContext,
    Query(args): Query<GetFriendsArgs>,
) -> ServiceResult<String> {
    let friends = users::fetch_friends_and_auth(&ctx, &args.username, &args.password_md5).await?;
    Ok(join_lines(&friends))
}
