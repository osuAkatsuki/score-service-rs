// osu!direct + score sub + leaderboard beatmap models and formats
use crate::adapters::beatmaps_service::{CheesegullBeatmap, CheesegullBeatmapset};
use crate::common::osu::format_bool;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DirectSearchArgs {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "h")]
    pub password_md5: String,
    /// The status filter of osu!direct, which is not a ranked status
    #[serde(rename = "r")]
    pub ranked_status: i32,
    #[serde(rename = "q")]
    pub query: String,
    /// -1 for all modes
    #[serde(rename = "m")]
    pub mode: i32,
    /// The first page is 0
    #[serde(rename = "p")]
    pub page: i32,
}

#[derive(Deserialize)]
pub struct DirectBeatmapsetArgs {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "h")]
    pub password_md5: String,
    #[serde(rename = "s")]
    pub beatmapset_id: Option<i32>,
    #[serde(rename = "b")]
    pub beatmap_id: Option<i32>,
}

pub struct DirectBeatmap {
    pub difficulty_name: String,
    pub difficulty_rating: f32,
    pub mode: i32,
    pub cs: f32,
    pub od: f32,
    pub ar: f32,
    pub hp: f32,
}

pub struct DirectBeatmapset {
    pub beatmapset_id: i32,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub ranked_status: i32,
    pub last_update: String,
    pub has_video: bool,
    /// Ordered from the easiest to the hardest difficulty
    pub beatmaps: Vec<DirectBeatmap>,
}

pub struct DirectSearchView {
    /// The amount of beatmapsets that were asked for
    pub page_size: usize,
    /// The amount of beatmapsets that were found, including those that are not listed
    pub result_count: usize,
    pub beatmapsets: Vec<DirectBeatmapset>,
}

impl From<CheesegullBeatmap> for DirectBeatmap {
    fn from(value: CheesegullBeatmap) -> Self {
        Self {
            difficulty_name: value.diff_name,
            difficulty_rating: value.difficulty_rating,
            mode: value.mode,
            cs: value.cs,
            od: value.od,
            ar: value.ar,
            hp: value.hp,
        }
    }
}

impl From<CheesegullBeatmapset> for DirectBeatmapset {
    fn from(value: CheesegullBeatmapset) -> Self {
        let mut beatmaps: Vec<_> = value
            .children_beatmaps
            .into_iter()
            .map(DirectBeatmap::from)
            .collect();
        beatmaps.sort_by(|a, b| a.difficulty_rating.total_cmp(&b.difficulty_rating));

        Self {
            beatmaps,
            beatmapset_id: value.set_id,
            artist: value.artist,
            title: value.title,
            creator: value.creator,
            ranked_status: value.ranked_status,
            last_update: value.last_update,
            has_video: value.has_video,
        }
    }
}

impl DirectBeatmap {
    /// The client displays this as the name of the difficulty and takes the mode from its end
    pub fn osu_format(&self) -> String {
        format!(
            "[{:.2}⭐] {} {{cs: {:?} / od: {:?} / ar: {:?} / hp: {:?}}}@{}",
            self.difficulty_rating,
            self.difficulty_name,
            self.cs,
            self.od,
            self.ar,
            self.hp,
            self.mode,
        )
    }
}

impl DirectBeatmapset {
    /// The fields behind the status are the rating, the last update, the id of the forum thread
    /// (for which the id of the beatmapset is used), whether the beatmapset has a video
    /// or a storyboard and the sizes of its downloads with and without video.
    fn osu_format_with(&self, has_video: &str, beatmaps: Option<String>) -> String {
        let beatmaps_fmt = match beatmaps {
            Some(beatmaps) => format!("|{beatmaps}"),
            None => String::new(),
        };
        format!(
            "{0}.osz|{1}|{2}|{3}|{4}|10.0|{5}|{0}|0|{has_video}|0|0|0{beatmaps_fmt}",
            self.beatmapset_id,
            self.artist,
            self.title,
            self.creator,
            self.ranked_status,
            self.last_update,
        )
    }

    /// The format of beatmapsets in the search results of osu!direct
    pub fn osu_format(&self) -> String {
        let beatmaps_fmt = self
            .beatmaps
            .iter()
            .map(|beatmap| beatmap.osu_format())
            .collect::<Vec<_>>()
            .join(",");
        self.osu_format_with(format_bool(self.has_video), Some(beatmaps_fmt))
    }

    /// The format of a single beatmapset, e.g. when it is linked in chat
    pub fn osu_card_format(&self) -> String {
        self.osu_format_with("0", None)
    }
}

impl IntoResponse for DirectBeatmapset {
    fn into_response(self) -> Response {
        self.osu_card_format().into_response()
    }
}

impl DirectSearchView {
    pub fn osu_format(&self) -> String {
        // One more result than there are on a page tells the client that there are more pages
        let result_count = match self.result_count >= self.page_size {
            true => self.page_size + 1,
            false => self.result_count,
        };

        std::iter::once(result_count.to_string())
            .chain(self.beatmapsets.iter().map(|set| set.osu_format()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl IntoResponse for DirectSearchView {
    fn into_response(self) -> Response {
        self.osu_format().into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Query;
    use axum::http::Uri;

    fn beatmap(difficulty_name: &str, difficulty_rating: f32) -> CheesegullBeatmap {
        CheesegullBeatmap {
            difficulty_rating,
            diff_name: difficulty_name.to_owned(),
            mode: 0,
            ar: 9.3,
            od: 8.0,
            cs: 4.0,
            hp: 6.5,
        }
    }

    fn beatmapset(children_beatmaps: Vec<CheesegullBeatmap>) -> DirectBeatmapset {
        DirectBeatmapset::from(CheesegullBeatmapset {
            children_beatmaps,
            set_id: 1,
            ranked_status: 1,
            last_update: "2014-05-18T15:41:48".to_owned(),
            artist: "Kenji Ninuma".to_owned(),
            title: "DISCO PRINCE".to_owned(),
            creator: "peppy".to_owned(),
            has_video: true,
        })
    }

    #[test]
    fn parses_the_search_of_an_osu_client() {
        let uri: Uri = "/web/osu-search.php?u=cmyui&h=5f4dcc3b5aa765d61d8327deb882cf99\
            &r=4&q=Top+Rated&m=-1&p=0"
            .parse()
            .unwrap();
        let Query(args) = Query::<DirectSearchArgs>::try_from_uri(&uri).unwrap();
        assert_eq!(args.query, "Top Rated");
        assert_eq!(args.ranked_status, 4);
        assert_eq!(args.mode, -1);
        assert_eq!(args.page, 0);
    }

    #[test]
    fn lists_difficulties_from_easiest_to_hardest() {
        let beatmapset = beatmapset(vec![beatmap("Insane", 4.567), beatmap("Normal", 2.5)]);
        assert_eq!(
            beatmapset.osu_format(),
            "1.osz|Kenji Ninuma|DISCO PRINCE|peppy|1|10.0|2014-05-18T15:41:48|1|0|True|0|0|0|\
            [2.50⭐] Normal {cs: 4.0 / od: 8.0 / ar: 9.3 / hp: 6.5}@0,\
            [4.57⭐] Insane {cs: 4.0 / od: 8.0 / ar: 9.3 / hp: 6.5}@0"
        );
    }

    #[test]
    fn formats_single_beatmapsets_without_their_difficulties() {
        let beatmapset = beatmapset(vec![beatmap("Normal", 2.5)]);
        assert_eq!(
            beatmapset.osu_card_format(),
            "1.osz|Kenji Ninuma|DISCO PRINCE|peppy|1|10.0|2014-05-18T15:41:48|1|0|0|0|0|0"
        );
    }

    #[test]
    fn full_pages_announce_more_results() {
        let view = DirectSearchView {
            page_size: 100,
            result_count: 100,
            beatmapsets: vec![],
        };
        assert_eq!(view.osu_format(), "101");

        let view = DirectSearchView {
            page_size: 100,
            result_count: 1,
            beatmapsets: vec![beatmapset(vec![beatmap("Normal", 2.5)])],
        };
        assert!(view.osu_format().starts_with("1\n1.osz|"));
    }
}
