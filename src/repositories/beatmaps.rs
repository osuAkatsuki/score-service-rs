use crate::api::context::RequestContext;

pub async fn increment_playcount(
    ctx: &RequestContext,
    beatmap_md5: &str,
    increment_passcount: bool,
) -> sqlx::Result<()> {
    sqlx::query("UPDATE beatmaps SET passcount = passcount + ?, playcount = playcount + 1 WHERE beatmap_md5 = ?")
        .bind(increment_passcount as u8)
        .bind(beatmap_md5)
        .execute(&ctx.db)
        .await?;
    Ok(())
}
