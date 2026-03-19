use crate::api::context::RequestContext;
use crate::entities::mode::ModdedGamemode;

pub async fn replace_first_place(
    ctx: &RequestContext,
    beatmap_md5: &str,
    mode: ModdedGamemode,
    score_id: i64,
    user_id: i64,
) -> sqlx::Result<()> {
    sqlx::query(
        "REPLACE INTO scores_first (beatmap_md5, mode, rx, scoreid, userid) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(beatmap_md5)
    .bind(mode.as_gamemode() as u8)
    .bind(mode.mode_mod() as u8)
    .bind(score_id)
    .bind(user_id)
    .execute(&ctx.db)
    .await?;
    Ok(())
}
