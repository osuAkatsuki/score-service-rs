use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::entities::mode::{GamemodeMod, ModdedGamemode};
use crate::entities::mods::Mods;
use crate::repositories::performance_caps;

pub async fn fetch_one(
    ctx: &RequestContext,
    mode: ModdedGamemode,
    mods: Mods,
) -> ServiceResult<f32> {
    let mode_caps = performance_caps::fetch_for_mode(ctx, mode.as_gamemode()).await?;
    let is_flashlight = mods.has_any(Mods::Flashlight);
    match mode.mode_mod() {
        GamemodeMod::Vanilla if is_flashlight => Ok(mode_caps.flashlight_pp as _),
        GamemodeMod::Vanilla => Ok(mode_caps.pp as _),
        GamemodeMod::Relax if is_flashlight => Ok(mode_caps.relax_flashlight_pp as _),
        GamemodeMod::Relax => Ok(mode_caps.relax_pp as _),
        GamemodeMod::Autopilot if is_flashlight => Ok(mode_caps.autopilot_flashlight_pp as _),
        GamemodeMod::Autopilot => Ok(mode_caps.autopilot_pp as _),
    }
}
