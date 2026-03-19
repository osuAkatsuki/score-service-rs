use crate::entities::mods::Mods;
use crate::entities::scores::{Score, ScoreStatus};
use crate::entities::stats::Stats;

fn passed(score: &Score) -> bool {
    score.status != ScoreStatus::Failed
}

pub fn intro_sudden_death(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::SuddenDeath) && passed(score)
}

pub fn intro_perfect(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::Perfect) && passed(score)
}

pub fn intro_hardrock(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::HardRock) && passed(score)
}

pub fn intro_doubletime(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::Doubletime) && passed(score)
}

pub fn intro_nightcore(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::Nightcore) && passed(score)
}

pub fn intro_hidden(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::Hidden) && passed(score)
}

pub fn intro_flashlight(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::Flashlight) && passed(score)
}

pub fn intro_easy(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::Easy) && passed(score)
}

pub fn intro_nofail(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::NoFail) && passed(score)
}

pub fn intro_halftime(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::Halftime) && passed(score)
}

pub fn intro_spunout(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.mods.has_any(Mods::SpunOut) && passed(score)
}
