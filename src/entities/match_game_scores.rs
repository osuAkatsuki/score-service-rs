use crate::entities::accuracy::Accuracy;
use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;

pub struct CreateMatchGameScoreArgs {
    pub match_id: i32,
    pub game_id: i32,
    pub user_id: i32,
    pub mode: Gamemode,
    pub accuracy: Accuracy,
    pub score: i32,
    pub max_combo: i32,
    pub mods: Mods,
    pub passed: bool,
    pub team: i8,
}