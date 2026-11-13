use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;
use crate::entities::scores::Score;
use crate::entities::stats::Stats;

pub fn catch_skill_pass_1(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (1.0..2.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_pass_2(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (2.0..3.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_pass_3(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (3.0..4.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_pass_4(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (4.0..5.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_pass_5(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (5.0..6.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_pass_6(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (6.0..7.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_pass_7(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (7.0..8.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_pass_8(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (8.0..9.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_fc_1(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (1.0..2.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_fc_2(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (2.0..3.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_fc_3(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (3.0..4.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_fc_4(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (4.0..5.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_fc_5(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (5.0..6.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_fc_6(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (6.0..7.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_fc_7(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (7.0..8.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_skill_fc_8(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (8.0..9.0).contains(&star_rating)
        && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn fruits_hits_20000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.total_hits >= 20_000 && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn fruits_hits_200000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.total_hits >= 200_000 && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn fruits_hits_2000000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.total_hits >= 2_000_000 && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn fruits_hits_20000000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.total_hits >= 20_000_000 && score.mode.as_gamemode() == Gamemode::Catch
}

pub fn catch_rank_50k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Catch && global_rank.is_some_and(|rank| rank <= 50_000)
}

pub fn catch_rank_10k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Catch && global_rank.is_some_and(|rank| rank <= 10_000)
}

pub fn catch_rank_5k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Catch && global_rank.is_some_and(|rank| rank <= 5_000)
}

pub fn catch_rank_1k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Catch && global_rank.is_some_and(|rank| rank <= 1_000)
}

/*
Implementations in python:
@registry.achievement(file="fruits-skill-pass-1")
def catch_skill_pass_1(score: Score, mode_vn: int, stats: Stats) -> bool:
    return (
        (score.mods & Mods.NOFAIL == 0) and 1 <= score.sr < 2 and mode_vn == Mode.CATCH
    )


@registry.achievement(file="fruits-skill-pass-2")
def catch_skill_pass_2(score: Score, mode_vn: int, stats: Stats) -> bool:
    return (
        (score.mods & Mods.NOFAIL == 0) and 2 <= score.sr < 3 and mode_vn == Mode.CATCH
    )


@registry.achievement(file="fruits-skill-pass-3")
def catch_skill_pass_3(score: Score, mode_vn: int, stats: Stats) -> bool:
    return (
        (score.mods & Mods.NOFAIL == 0) and 3 <= score.sr < 4 and mode_vn == Mode.CATCH
    )


@registry.achievement(file="fruits-skill-pass-4")
def catch_skill_pass_4(score: Score, mode_vn: int, stats: Stats) -> bool:
    return (
        (score.mods & Mods.NOFAIL == 0) and 4 <= score.sr < 5 and mode_vn == Mode.CATCH
    )


@registry.achievement(file="fruits-skill-pass-5")
def catch_skill_pass_5(score: Score, mode_vn: int, stats: Stats) -> bool:
    return (
        (score.mods & Mods.NOFAIL == 0) and 5 <= score.sr < 6 and mode_vn == Mode.CATCH
    )


@registry.achievement(file="fruits-skill-pass-6")
def catch_skill_pass_6(score: Score, mode_vn: int, stats: Stats) -> bool:
    return (
        (score.mods & Mods.NOFAIL == 0) and 6 <= score.sr < 7 and mode_vn == Mode.CATCH
    )


@registry.achievement(file="fruits-skill-pass-7")
def catch_skill_pass_7(score: Score, mode_vn: int, stats: Stats) -> bool:
    return (
        (score.mods & Mods.NOFAIL == 0) and 7 <= score.sr < 8 and mode_vn == Mode.CATCH
    )


@registry.achievement(file="fruits-skill-pass-8")
def catch_skill_pass_8(score: Score, mode_vn: int, stats: Stats) -> bool:
    return (
        (score.mods & Mods.NOFAIL == 0) and 8 <= score.sr < 9 and mode_vn == Mode.CATCH
    )


# ====================
# Skill FC Achievements
# ====================


@registry.achievement(file="fruits-skill-fc-1")
def catch_skill_fc_1(score: Score, mode_vn: int, stats: Stats) -> bool:
    return score.full_combo and 1 <= score.sr < 2 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-skill-fc-2")
def catch_skill_fc_2(score: Score, mode_vn: int, stats: Stats) -> bool:
    return score.full_combo and 2 <= score.sr < 3 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-skill-fc-3")
def catch_skill_fc_3(score: Score, mode_vn: int, stats: Stats) -> bool:
    return score.full_combo and 3 <= score.sr < 4 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-skill-fc-4")
def catch_skill_fc_4(score: Score, mode_vn: int, stats: Stats) -> bool:
    return score.full_combo and 4 <= score.sr < 5 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-skill-fc-5")
def catch_skill_fc_5(score: Score, mode_vn: int, stats: Stats) -> bool:
    return score.full_combo and 5 <= score.sr < 6 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-skill-fc-6")
def catch_skill_fc_6(score: Score, mode_vn: int, stats: Stats) -> bool:
    return score.full_combo and 6 <= score.sr < 7 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-skill-fc-7")
def catch_skill_fc_7(score: Score, mode_vn: int, stats: Stats) -> bool:
    return score.full_combo and 7 <= score.sr < 8 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-skill-fc-8")
def catch_skill_fc_8(score: Score, mode_vn: int, stats: Stats) -> bool:
    return score.full_combo and 8 <= score.sr < 9 and mode_vn == Mode.CATCH


# ====================
# Hit Count Achievements
# ====================


@registry.achievement(file="fruits-hits-20000")
def fruits_hits_20000(score: Score, mode_vn: int, stats: Stats) -> bool:
    return 20_000 <= stats.total_hits and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-hits-200000")
def fruits_hits_200000(score: Score, mode_vn: int, stats: Stats) -> bool:
    return 200_000 <= stats.total_hits and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-hits-2000000")
def fruits_hits_2000000(score: Score, mode_vn: int, stats: Stats) -> bool:
    return 2_000_000 <= stats.total_hits and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-hits-20000000")
def fruits_hits_20000000(score: Score, mode_vn: int, stats: Stats) -> bool:
    return 20_000_000 <= stats.total_hits and mode_vn == Mode.CATCH


# ====================
# Rank Achievements
# ====================


@registry.achievement(file="fruits-rank-50000")
def catch_rank_50k(score: Score, mode_vn: int, stats: Stats) -> bool:
    return stats.rank <= 50_000 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-rank-10000")
def catch_rank_10k(score: Score, mode_vn: int, stats: Stats) -> bool:
    return stats.rank <= 10_000 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-rank-5000")
def catch_rank_5k(score: Score, mode_vn: int, stats: Stats) -> bool:
    return stats.rank <= 5_000 and mode_vn == Mode.CATCH


@registry.achievement(file="fruits-rank-1000")
def catch_rank_1k(score: Score, mode_vn: int, stats: Stats) -> bool:
    return stats.rank <= 1_000 and mode_vn == Mode.CATCH
*/
