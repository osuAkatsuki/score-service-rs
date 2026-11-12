use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;
use crate::entities::scores::LeaderboardScore;
use crate::models::beatmaps::{Beatmap, RankedStatus};

pub enum UserLeaderboardView {
    Status(RankedStatus),
    ScoresView(UserLeaderboardScoresView),
}

pub struct UserLeaderboardScoresView {
    pub beatmap: Beatmap,
    pub total_scores: i64,
    pub display_pp_as_score: bool,
    pub scores: Vec<LeaderboardScore>,
    pub personal_best: Option<(i64, LeaderboardScore)>,
}

impl UserLeaderboardView {
    pub fn osu_format(&self) -> String {
        match self {
            UserLeaderboardView::Status(status) => format!("{}|false", *status as i8),
            UserLeaderboardView::ScoresView(view) => view.osu_format(),
        }
    }
}

impl IntoResponse for UserLeaderboardView {
    fn into_response(self) -> Response {
        let osu_formatted = self.osu_format();
        osu_formatted.into_response()
    }
}

impl UserLeaderboardScoresView {
    pub fn osu_format(&self) -> String {
        let personal_best_fmt = self.personal_best.as_ref()
            .map(|(rank, pb)| pb.osu_format(*rank, false, self.display_pp_as_score))
            .unwrap_or(String::new());

        let scores_fmt = self.scores.iter().enumerate()
            .map(|(index, score)| score.osu_format(index as _, true, self.display_pp_as_score))
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            "{}|false|{}|{}|{}|0\n0\n{}\n0\n{personal_best_fmt}\n{scores_fmt}",
            self.beatmap.status as i8,
            self.beatmap.beatmap_id,
            self.beatmap.beatmap_set_id,
            self.total_scores,
            self.beatmap.song_name,
        )
    }
}

#[repr(u8)]
#[derive(Debug, Deserialize_repr, Copy, Clone, PartialEq, Eq)]
pub enum LeaderboardType {
    Local = 0,
    Global = 1,
    Mods = 2,
    Friends = 3,
    Country = 4,
}

#[derive(Deserialize)]
pub struct FetchLeaderboardArgs {
    #[serde(rename = "us")]
    pub username: String,
    #[serde(rename = "ha")]
    pub password_md5: String,

    #[serde(rename = "vv")]
    pub leaderboard_version: i32,
    #[serde(rename = "v")]
    pub leaderboard_type: LeaderboardType,
    #[serde(rename = "c")]
    pub beatmap_md5: String,
    #[serde(rename = "f")]
    pub file_name: String,
    #[serde(rename = "m")]
    pub mode: Gamemode,
    #[serde(rename = "i")]
    pub map_set_id: i32,
    #[serde(rename = "mods")]
    pub mods: Mods,
    #[serde(rename = "h")]
    pub map_package_hash: String,
    #[serde(rename = "a")]
    pub aqn_files_found: bool,
    #[serde(rename = "s")]
    pub requesting_from_editor_song_select: bool,
}