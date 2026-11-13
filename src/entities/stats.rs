use crate::entities::scores::ScoreGrade;
use chrono::Utc;

#[derive(sqlx::FromRow, Copy, Clone, Default)]
pub struct Stats {
    pub user_id: i64,
    pub mode: i16,
    pub ranked_score: u64,
    pub total_score: u64,
    pub playcount: u32,
    pub replays_watched: u32,
    pub total_hits: u32,
    pub level: u32,
    pub avg_accuracy: f64,
    pub pp: u32,
    pub playtime: u64,
    pub xh_count: u64,
    pub x_count: u64,
    pub sh_count: u64,
    pub s_count: u64,
    pub a_count: u64,
    pub b_count: u64,
    pub c_count: u64,
    pub d_count: u64,
    pub max_combo: u32,
    pub latest_pp_awarded: u64,
}

#[derive(Default)]
pub struct UpdateStats {
    pub increment_playcount: bool,
    pub increment_total_hits: Option<u32>,
    pub increment_total_score: Option<i32>,
    pub increment_playtime: Option<i32>,
    pub ranked_score_change: Option<i32>,
    pub increment_grade: Option<ScoreGrade>,
    pub decrement_grade: Option<ScoreGrade>,
    pub max_combo: Option<u16>,
    pub update_pp: Option<u32>,
    pub update_latest_pp_awarded: bool,
    pub update_avg_accuracy: Option<f64>,
}

impl UpdateStats {
    pub fn increment_play_count(&mut self) {
        self.increment_playcount = true;
    }

    pub fn increment_total_hits(&mut self, total_hits: u32) {
        self.increment_total_hits = Some(total_hits);
    }

    pub fn increment_total_score(&mut self, total_score: i32) {
        self.increment_total_score = Some(total_score);
    }

    pub fn increment_playtime(&mut self, playtime: i32) {
        self.increment_playtime = Some(playtime);
    }

    pub fn adjust_ranked_score(&mut self, ranked_score_change: i32) {
        self.ranked_score_change = Some(ranked_score_change);
    }

    pub fn decrement_grade(&mut self, grade: ScoreGrade) {
        self.decrement_grade = Some(grade);
    }

    pub fn increment_grade(&mut self, grade: ScoreGrade) {
        self.increment_grade = Some(grade);
    }

    pub fn update_max_combo(&mut self, max_combo: u16) {
        self.max_combo = Some(max_combo);
    }

    pub fn update_pp(&mut self, new_pp: u32) {
        self.update_pp = Some(new_pp);
    }

    pub fn update_latest_pp_awarded(&mut self) {
        self.update_latest_pp_awarded = true;
    }

    pub fn update_avg_accuracy(&mut self, avg_accuracy: f64) {
        self.update_avg_accuracy = Some(avg_accuracy);
    }

    pub fn apply(&self, stats: &Stats) -> Stats {
        let mut new_stats = *stats;

        if self.increment_playcount {
            new_stats.playcount += 1;
        }

        if let Some(incr_total_hits) = self.increment_total_hits {
            new_stats.total_hits += incr_total_hits;
        }

        if let Some(incr_total_score) = self.increment_total_score {
            new_stats.total_score = new_stats
                .total_score
                .saturating_add_signed(incr_total_score as _);
        }

        if let Some(incr_playtime) = self.increment_playtime {
            new_stats.playtime += incr_playtime as u64;
        }

        if let Some(ranked_score_change) = self.ranked_score_change {
            new_stats.ranked_score = new_stats
                .ranked_score
                .saturating_add_signed(ranked_score_change as _);
        }

        if let Some(incr_grade) = self.increment_grade {
            match incr_grade {
                ScoreGrade::XH => new_stats.xh_count += 1,
                ScoreGrade::X => new_stats.x_count += 1,
                ScoreGrade::SH => new_stats.sh_count += 1,
                ScoreGrade::S => new_stats.s_count += 1,
                ScoreGrade::A => new_stats.a_count += 1,
                ScoreGrade::B => new_stats.b_count += 1,
                ScoreGrade::C => new_stats.c_count += 1,
                ScoreGrade::D => new_stats.d_count += 1,
            }
        }

        // Not every score has been counted towards its grade, so there may be nothing to take away
        if let Some(decr_grade) = self.decrement_grade {
            match decr_grade {
                ScoreGrade::XH => new_stats.xh_count = new_stats.xh_count.saturating_sub(1),
                ScoreGrade::X => new_stats.x_count = new_stats.x_count.saturating_sub(1),
                ScoreGrade::SH => new_stats.sh_count = new_stats.sh_count.saturating_sub(1),
                ScoreGrade::S => new_stats.s_count = new_stats.s_count.saturating_sub(1),
                ScoreGrade::A => new_stats.a_count = new_stats.a_count.saturating_sub(1),
                ScoreGrade::B => new_stats.b_count = new_stats.b_count.saturating_sub(1),
                ScoreGrade::C => new_stats.c_count = new_stats.c_count.saturating_sub(1),
                ScoreGrade::D => new_stats.d_count = new_stats.d_count.saturating_sub(1),
            }
        }

        if let Some(max_combo) = self.max_combo {
            new_stats.max_combo = max_combo as _;
        }

        if let Some(updated_pp) = self.update_pp {
            new_stats.pp = updated_pp;
        }

        if self.update_latest_pp_awarded {
            let now = Utc::now();
            let timestamp = now.timestamp();
            new_stats.latest_pp_awarded = timestamp as _;
        }

        if let Some(updated_avg_accuracy) = self.update_avg_accuracy {
            new_stats.avg_accuracy = updated_avg_accuracy;
        }

        new_stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_an_improved_score_from_its_old_grade_to_the_new_one() {
        let stats = Stats {
            a_count: 3,
            s_count: 1,
            ..Default::default()
        };
        let mut update = UpdateStats::default();
        update.decrement_grade(ScoreGrade::A);
        update.increment_grade(ScoreGrade::S);

        let stats = update.apply(&stats);
        assert_eq!(stats.a_count, 2);
        assert_eq!(stats.s_count, 2);
    }

    #[test]
    fn grade_counts_do_not_drop_below_zero() {
        let mut update = UpdateStats::default();
        update.decrement_grade(ScoreGrade::A);
        update.increment_grade(ScoreGrade::S);

        let stats = update.apply(&Stats::default());
        assert_eq!(stats.a_count, 0);
        assert_eq!(stats.s_count, 1);
    }
}
