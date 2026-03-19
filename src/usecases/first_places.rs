use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::entities::mode::ModdedGamemode;
use crate::repositories::first_places;

pub async fn handle_first_place(
    ctx: &RequestContext,
    beatmap_md5: &str,
    mode: ModdedGamemode,
    score_id: i64,
    user_id: i64,
) -> ServiceResult<()> {
    first_places::replace_first_place(ctx, beatmap_md5, mode, score_id, user_id).await?;
    // TODO: send announcement via bancho
    Ok(())
}
