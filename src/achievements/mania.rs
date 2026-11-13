use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;
use crate::entities::scores::Score;
use crate::entities::stats::Stats;

pub fn mania_skill_pass_1(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (1.0..2.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_pass_2(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (2.0..3.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_pass_3(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (3.0..4.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_pass_4(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (4.0..5.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_pass_5(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (5.0..6.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_pass_6(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (6.0..7.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_pass_7(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (7.0..8.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_pass_8(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (8.0..9.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_fc_1(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (1.0..2.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_fc_2(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (2.0..3.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_fc_3(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (3.0..4.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_fc_4(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (4.0..5.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_fc_5(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (5.0..6.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_fc_6(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (6.0..7.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_fc_7(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (7.0..8.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_skill_fc_8(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (8.0..9.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_hits_40000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.total_hits >= 40_000 && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_hits_400000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.total_hits >= 400_000 && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_hits_4000000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.total_hits >= 4_000_000 && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_hits_40000000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.total_hits >= 40_000_000 && score.mode.as_gamemode() == Gamemode::Mania
}

pub fn mania_rank_50k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Mania && global_rank.is_some_and(|rank| rank <= 50_000)
}

pub fn mania_rank_10k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Mania && global_rank.is_some_and(|rank| rank <= 10_000)
}

pub fn mania_rank_5k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Mania && global_rank.is_some_and(|rank| rank <= 5_000)
}

pub fn mania_rank_1k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Mania && global_rank.is_some_and(|rank| rank <= 1_000)
}
