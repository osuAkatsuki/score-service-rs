use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;
use crate::entities::scores::Score;
use crate::entities::stats::Stats;

pub fn osu_skill_pass_1(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (1.0 <= star_rating)
        && (star_rating < 2.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_pass_2(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (2.0 <= star_rating)
        && (star_rating < 3.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_pass_3(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (3.0 <= star_rating)
        && (star_rating < 4.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_pass_4(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (4.0 <= star_rating)
        && (star_rating < 5.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_pass_5(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (5.0 <= star_rating)
        && (star_rating < 6.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_pass_6(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (6.0 <= star_rating)
        && (star_rating < 7.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_pass_7(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (7.0 <= star_rating)
        && (star_rating < 8.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_pass_8(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (8.0 <= star_rating)
        && (star_rating < 9.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_pass_9(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (9.0 <= star_rating)
        && (star_rating < 10.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_pass_10(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    !score.mods.has_any(Mods::NoFail)
        && (10.0 <= star_rating)
        && (star_rating < 11.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_1(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (1.0 <= star_rating)
        && (star_rating < 2.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_2(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (2.0 <= star_rating)
        && (star_rating < 3.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_3(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (3.0 <= star_rating)
        && (star_rating < 4.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_4(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (4.0 <= star_rating)
        && (star_rating < 5.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_5(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (5.0 <= star_rating)
        && (star_rating < 6.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_6(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (6.0 <= star_rating)
        && (star_rating < 7.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_7(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (7.0 <= star_rating)
        && (star_rating < 8.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_8(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (8.0 <= star_rating)
        && (star_rating < 9.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_9(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (9.0 <= star_rating)
        && (star_rating < 10.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_skill_fc_10(
    score: &Score,
    _stats: &Stats,
    star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo
        && (10.0 <= star_rating)
        && (star_rating < 11.0)
        && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_combo_500(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.max_combo >= 500 && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_combo_750(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.max_combo >= 750 && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_combo_1000(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.max_combo >= 1_000 && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_combo_2000(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.max_combo >= 2_000 && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_plays_5000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.playcount >= 5_000 && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_plays_15000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.playcount >= 15_000 && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_plays_25000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.playcount >= 25_000 && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_plays_50000(
    score: &Score,
    stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    stats.playcount >= 50_000 && score.mode.as_gamemode() == Gamemode::Standard
}

pub fn osu_rank_50k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Standard && global_rank.is_some_and(|rank| rank <= 50_000)
}

pub fn osu_rank_10k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Standard && global_rank.is_some_and(|rank| rank <= 10_000)
}

pub fn osu_rank_5k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Standard && global_rank.is_some_and(|rank| rank <= 5_000)
}

pub fn osu_rank_1k(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    global_rank: Option<usize>,
) -> bool {
    score.mode.as_gamemode() == Gamemode::Standard && global_rank.is_some_and(|rank| rank <= 1_000)
}
