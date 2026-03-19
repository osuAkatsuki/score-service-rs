use crate::common::website;
use crate::entities::scores::Score;
use crate::entities::stats::Stats;
use crate::models::beatmaps::Beatmap;
use crate::models::osu::charts::{Chart, ChartEntry};

pub fn beatmap_ranking(
    beatmap: &Beatmap,
    personal_best: &Option<Score>,
    new_score: &Score,
    personal_best_rank: Option<i64>,
    score_rank: i64,
) -> Chart {
    Chart {
        chart_id: "beatmap",
        chart_url: beatmap.set_url(),
        chart_name: "Beatmap Ranking",
        entries: vec![
            ChartEntry::new_int("rank", personal_best_rank, score_rank),
            ChartEntry::new_int(
                "rankedScore",
                personal_best.as_ref().map(|pb| pb.score as _),
                new_score.score as _,
            ),
            ChartEntry::new_int(
                "totalScore",
                personal_best.as_ref().map(|pb| pb.score as _),
                new_score.score as _,
            ),
            ChartEntry::new_int(
                "maxCombo",
                personal_best.as_ref().map(|pb| pb.max_combo as _),
                new_score.max_combo as _,
            ),
            ChartEntry::new_float(
                "accuracy",
                personal_best
                    .as_ref()
                    .map(|pb| pb.accuracy.as_percent(pb.mode.as_gamemode())),
                new_score.accuracy.as_percent(new_score.mode.as_gamemode()),
            ),
            ChartEntry::new_int(
                "pp",
                personal_best.as_ref().map(|pb| pb.performance as _),
                new_score.performance as _,
            ),
        ],
    }
}

pub fn overall_ranking(
    user_id: i64,
    old_stats: Stats,
    new_stats: Stats,
    old_rank: Option<usize>,
    new_rank: Option<usize>,
) -> Chart {
    Chart {
        chart_id: "overall",
        chart_url: website::get_profile_link(user_id),
        chart_name: "Overall Ranking",
        entries: vec![
            ChartEntry::new_int(
                "rank",
                old_rank.map(|x| x as _),
                new_rank.or(old_rank).map(|x| x as _).unwrap_or(0),
            ),
            ChartEntry::new_int(
                "rankedScore",
                Some(old_stats.ranked_score as _),
                new_stats.ranked_score as _,
            ),
            ChartEntry::new_int(
                "totalScore",
                Some(old_stats.total_score as _),
                new_stats.total_score as _,
            ),
            ChartEntry::new_int(
                "maxCombo",
                Some(old_stats.max_combo as _),
                new_stats.max_combo as _,
            ),
            ChartEntry::new_float(
                "accuracy",
                Some(old_stats.avg_accuracy as _),
                new_stats.avg_accuracy as _,
            ),
            ChartEntry::new_int("pp", Some(old_stats.pp as _), new_stats.pp as _),
        ],
    }
}
