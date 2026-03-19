#[derive(sqlx::FromRow)]
pub struct PerformanceCaps {
    pub gamemode: i8,
    pub pp: i16,
    pub relax_pp: i16,
    pub autopilot_pp: i16,
    pub flashlight_pp: i16,
    pub relax_flashlight_pp: i16,
    pub autopilot_flashlight_pp: i16,
}
