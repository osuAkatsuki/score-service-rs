use crate::api::context::RequestContext;
use crate::entities::mode::Gamemode;
use crate::entities::performance_caps::PerformanceCaps;

pub async fn fetch_for_mode(ctx: &RequestContext, mode: Gamemode) -> sqlx::Result<PerformanceCaps> {
    sqlx::query_as(
        r#"SELECT gamemode, pp ,relax_pp, autopilot_pp,
        flashlight_pp, relax_flashlight_pp, autopilot_flashlight_pp
        FROM pp_limits
        WHERE gamemode = ?"#,
    )
    .bind(mode as u8)
    .fetch_one(&ctx.db)
    .await
}
