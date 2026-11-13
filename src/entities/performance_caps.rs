#[derive(sqlx::FromRow)]
pub struct PerformanceCaps {
    pub gamemode: i8,
    pub pp: i32,
    pub relax_pp: i32,
    pub autopilot_pp: i32,
    pub flashlight_pp: i32,
    pub relax_flashlight_pp: i32,
    pub autopilot_flashlight_pp: i32,
}
