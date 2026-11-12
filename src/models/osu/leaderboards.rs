use crate::common::osu::deserialize_bool;
use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;
use crate::entities::scores::LeaderboardScore;
use crate::models::beatmaps::{Beatmap, RankedStatus};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

pub enum UserLeaderboardView {
    Status(RankedStatus),
    /// The beatmap has a leaderboard, but its scores were not requested
    Beatmap(Beatmap),
    ScoresView(UserLeaderboardScoresView),
}

pub struct UserLeaderboardScoresView {
    /// The user this leaderboard is displayed to
    pub user_id: i64,
    pub beatmap: Beatmap,
    pub total_scores: i64,
    pub display_pp_as_score: bool,
    pub scores: Vec<LeaderboardScore>,
    pub personal_best: Option<LeaderboardScore>,
}

fn format_beatmap(beatmap: &Beatmap, total_scores: i64) -> String {
    // Beatmaps nobody has rated yet are displayed with the best possible rating
    let rating = match beatmap.rating > 0.0 {
        true => beatmap.rating,
        false => 10.0,
    };
    // The trailing fields are whether the song is by a featured artist and the audio offset
    format!(
        "{}|false|{}|{}|{total_scores}|0|\n0\n{}\n{rating:.1}",
        beatmap.status as i8, beatmap.beatmap_id, beatmap.beatmap_set_id, beatmap.song_name,
    )
}

impl UserLeaderboardView {
    pub fn osu_format(&self) -> String {
        match self {
            UserLeaderboardView::Status(status) => format!("{}|false", *status as i8),
            UserLeaderboardView::Beatmap(beatmap) => format_beatmap(beatmap, 0),
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
        let beatmap_fmt = format_beatmap(&self.beatmap, self.total_scores);
        let personal_best_fmt = self
            .personal_best
            .as_ref()
            .map(|pb| pb.osu_format(false, self.display_pp_as_score))
            .unwrap_or_default();

        // The client finds the user's own score by their name, so it must not get a clan tag
        let scores_fmt = self.scores.iter().map(|score| {
            score.osu_format(score.user_id != self.user_id, self.display_pp_as_score)
        });

        [beatmap_fmt, personal_best_fmt]
            .into_iter()
            .chain(scores_fmt)
            .collect::<Vec<String>>()
            .join("\n")
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
    #[serde(rename = "a", deserialize_with = "deserialize_bool")]
    pub aqn_files_found: bool,
    #[serde(rename = "s", deserialize_with = "deserialize_bool")]
    pub requesting_from_editor_song_select: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::accuracy::Accuracy;
    use crate::models::beatmaps::BeatmapGamemode;
    use axum::extract::Query;
    use axum::http::Uri;
    use chrono::DateTime;

    fn beatmap(rating: f32) -> Beatmap {
        Beatmap {
            beatmap_id: 75,
            beatmap_set_id: 1,
            beatmap_md5: "a5b99395a42bd55bc5eb1d2411cbdf8b".to_owned(),
            song_name: "Kenji Ninuma - DISCO PRINCE [Normal]".to_owned(),
            file_name: "Kenji Ninuma - DISCOPRINCE (peppy) [Normal].osu".to_owned(),
            ar: 6.0,
            od: 6.0,
            mode: BeatmapGamemode::Any,
            max_combo: 314,
            hit_length: 142,
            bpm: 120,
            status: RankedStatus::Ranked,
            status_frozen: false,
            latest_update: 0,
            playcount: 10,
            passcount: 5,
            ranked_by: None,
            rating,
            bancho_ranked_status: None,
            count_circles: Some(160),
            count_spinners: Some(4),
            count_sliders: Some(30),
            bancho_creator_id: None,
            bancho_creator_name: None,
        }
    }

    fn score(rank: i64, user_id: i64, username: &str, clan_tag: Option<&str>) -> LeaderboardScore {
        LeaderboardScore {
            rank,
            user_id,
            username: username.to_owned(),
            clan_tag: clan_tag.map(str::to_owned),
            score_id: 1000 + rank,
            score: 1_000_000 - rank as i32,
            max_combo: 314,
            perfect_combo: true,
            mods: Mods::Hidden | Mods::Doubletime,
            accuracy: Accuracy {
                count_300: 190,
                count_100: 3,
                count_50: 1,
                count_geki: 20,
                count_katu: 2,
                count_miss: 0,
            },
            performance: 123.98,
            created_at: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        }
    }

    #[test]
    fn parses_the_query_of_an_osu_client() {
        let uri: Uri = "/web/osu-osz2-getscores.php?s=0&vv=4&v=1&c=a5b99395a42bd55bc5eb1d2411cbdf8b\
            &f=Kenji+Ninuma+-+DISCO%E2%98%85PRINCE+(peppy)+%5bNormal%5d.osu&m=0&i=1&mods=72\
            &h=&a=0&us=cmyui&ha=5f4dcc3b5aa765d61d8327deb882cf99"
            .parse()
            .unwrap();
        let Query(args) = Query::<FetchLeaderboardArgs>::try_from_uri(&uri).unwrap();

        assert_eq!(args.username, "cmyui");
        assert_eq!(args.leaderboard_version, 4);
        assert_eq!(args.leaderboard_type, LeaderboardType::Global);
        assert_eq!(args.mode, Gamemode::Standard);
        assert_eq!(args.mods, Mods::Hidden | Mods::Doubletime);
        assert_eq!(
            args.file_name,
            "Kenji Ninuma - DISCO★PRINCE (peppy) [Normal].osu"
        );
        assert!(!args.aqn_files_found);
        assert!(!args.requesting_from_editor_song_select);
    }

    #[test]
    fn formats_beatmaps_without_a_leaderboard_as_status() {
        let view = UserLeaderboardView::Status(RankedStatus::NotSubmitted);
        assert_eq!(view.osu_format(), "-1|false");
    }

    #[test]
    fn formats_unrated_beatmaps_with_the_best_rating() {
        let view = UserLeaderboardView::Beatmap(beatmap(0.0));
        assert_eq!(
            view.osu_format(),
            "2|false|75|1|0|0|\n0\nKenji Ninuma - DISCO PRINCE [Normal]\n10.0"
        );
    }

    #[test]
    fn formats_scores_with_personal_best() {
        let view = UserLeaderboardScoresView {
            user_id: 1001,
            beatmap: beatmap(8.76),
            total_scores: 52,
            display_pp_as_score: false,
            scores: vec![
                score(1, 1000, "peppy", Some("OSU")),
                score(2, 1001, "cmyui", Some("AKA")),
            ],
            personal_best: Some(score(2, 1001, "cmyui", Some("AKA"))),
        };

        let expected = "2|false|75|1|52|0|\n0\nKenji Ninuma - DISCO PRINCE [Normal]\n8.8\n\
            1002|cmyui|999998|314|1|3|190|0|2|20|1|72|1001|2|1700000000|1\n\
            1001|[OSU] peppy|999999|314|1|3|190|0|2|20|1|72|1000|1|1700000000|1\n\
            1002|cmyui|999998|314|1|3|190|0|2|20|1|72|1001|2|1700000000|1";
        assert_eq!(view.osu_format(), expected);
    }

    #[test]
    fn formats_performance_as_score_and_keeps_the_personal_best_line() {
        let view = UserLeaderboardScoresView {
            user_id: 1001,
            beatmap: beatmap(8.76),
            total_scores: 1,
            display_pp_as_score: true,
            scores: vec![score(1, 1000, "peppy", None)],
            personal_best: None,
        };

        let expected = "2|false|75|1|1|0|\n0\nKenji Ninuma - DISCO PRINCE [Normal]\n8.8\n\n\
            1001|peppy|123|314|1|3|190|0|2|20|1|72|1000|1|1700000000|1";
        assert_eq!(view.osu_format(), expected);
    }
}
