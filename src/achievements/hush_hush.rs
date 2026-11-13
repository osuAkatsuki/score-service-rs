use crate::entities::scores::{Score, ScoreGrade};
use crate::entities::stats::Stats;

fn passed(score: &Score) -> bool {
    score.status.is_pass()
}

pub fn dont_let_bunny_distract(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires beatmap lookup to check artist/title
    // Would need to fetch beatmap data from beatmaps-service
    false
}

pub fn s_ranker(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires tracking recent scores within time window
    // Would need to query database for user's scores in last 24h
    false
}

pub fn most_improved(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires tracking score history per beatmap with timestamps
    // Would need to query previous scores on same beatmap
    false
}

pub fn nonstop_dancer(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires beatmap lookup to check artist/title
    // Would need to fetch beatmap data from beatmaps-service
    false
}

pub fn consolation_prize(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    if !passed(score) || score.score < 100_000 {
        return false;
    }

    score.accuracy.grade(score.mode.as_gamemode(), score.mods) == ScoreGrade::D
}

pub fn challenge_accepted(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires beatmap ranked status lookup
    // Would need to fetch beatmap data to check if approved
    // For now, return false as we can't verify approved status during score submission
    false
}

pub fn stumbler(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    score.perfect_combo && score.accuracy.as_percent(score.mode.as_gamemode()) < 85.0
}

pub fn jackpot(
    score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    if !passed(score) {
        return false;
    }

    let score_str = score.score.to_string();
    if score_str.len() < 2 {
        return false;
    }

    let Some(first) = score_str.chars().next() else {
        return false;
    };

    score_str.chars().all(|c| c == first)
}

pub fn quick_draw(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires checking if this is the first score on the beatmap
    // Would need to query database for existing scores on this map
    false
}

pub fn obsessed(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires tracking retry count and time window
    // Would need to count failed attempts on same beatmap in last 24h
    false
}

pub fn nonstop(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires beatmap drain time lookup
    // Would need to fetch beatmap data from beatmaps-service
    // time_elapsed could be used but it's not currently stored in DB
    false
}

pub fn jack_of_all_trades(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires fetching playcount across all modes
    // Stats object only contains current mode data
    // Would need to query user_stats for all 4 modes
    false
}

pub fn twin_perspectives(
    _score: &Score,
    _stats: &Stats,
    _star_rating: f32,
    _global_rank: Option<usize>,
) -> bool {
    // TODO: Requires beatmap ranked status check (RANKED or APPROVED only, not LOVED/QUALIFIED)
    // unlock_achievements() doesn't receive Beatmap object, only Score and Stats
    // Current call site checks beatmap.has_leaderboard which includes LOVED/QUALIFIED
    // Need to either:
    //   1. Add Beatmap parameter to unlock_achievements (breaking change to all achievements)
    //   2. Add ranked_status field to Score object
    //   3. Filter at call site before checking achievements
    false
}
