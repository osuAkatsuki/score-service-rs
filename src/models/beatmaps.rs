use crate::adapters::beatmaps_service::AkatsukiBeatmap;
use crate::entities::mods::Mods;
use crate::settings::AppSettings;
use chrono::DateTime;

#[repr(u8)]
#[derive(Debug)]
pub enum BeatmapGamemode {
    Any = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
}

impl From<u8> for BeatmapGamemode {
    fn from(value: u8) -> Self {
        match value {
            1 => BeatmapGamemode::Taiko,
            2 => BeatmapGamemode::Catch,
            3 => BeatmapGamemode::Mania,
            _ => BeatmapGamemode::Any,
        }
    }
}

#[repr(i8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RankedStatus {
    NotSubmitted = -1,
    Pending = 0,
    UpdateAvailable = 1,
    Ranked = 2,
    Approved = 3,
    Qualified = 4,
    Loved = 5,
}

impl From<i8> for RankedStatus {
    fn from(value: i8) -> Self {
        match value {
            0 => RankedStatus::Pending,
            1 => RankedStatus::UpdateAvailable,
            2 => RankedStatus::Ranked,
            3 => RankedStatus::Approved,
            4 => RankedStatus::Qualified,
            5 => RankedStatus::Loved,
            _ => RankedStatus::NotSubmitted,
        }
    }
}

#[derive(Debug)]
pub struct Beatmap {
    pub beatmap_id: i32,
    pub beatmap_set_id: i32,
    pub beatmap_md5: String,
    pub song_name: String,
    pub file_name: String,
    pub ar: f32,
    pub od: f32,
    pub mode: BeatmapGamemode,
    pub max_combo: i32,
    pub hit_length: i32,
    pub bpm: i32,
    pub status: RankedStatus,
    pub status_frozen: bool,
    pub latest_update: i64,
    pub playcount: i64,
    pub passcount: i64,
    pub ranked_by: Option<i64>,
    pub rating: f32,
    pub bancho_ranked_status: Option<i8>,
    pub count_circles: Option<i32>,
    pub count_spinners: Option<i32>,
    pub count_sliders: Option<i32>,
    pub bancho_creator_id: Option<i32>,
    pub bancho_creator_name: Option<String>,
}

impl From<AkatsukiBeatmap> for Beatmap {
    fn from(value: AkatsukiBeatmap) -> Self {
        Self {
            beatmap_id: value.beatmap_id,
            beatmap_set_id: value.beatmapset_id,
            beatmap_md5: value.beatmap_md5,
            song_name: value.song_name,
            file_name: value.file_name,
            ar: value.ar,
            od: value.od,
            mode: BeatmapGamemode::from(value.mode),
            max_combo: value.max_combo,
            hit_length: value.hit_length,
            bpm: value.bpm,
            status: RankedStatus::from(value.ranked),
            status_frozen: value.ranked_status_freezed,
            latest_update: value.latest_update,
            playcount: value.playcount,
            passcount: value.passcount,
            ranked_by: value.rankedby,
            rating: value.rating,
            bancho_ranked_status: value.bancho_ranked_status,
            count_circles: value.count_circles,
            count_spinners: value.count_spinners,
            count_sliders: value.count_sliders,
            bancho_creator_id: value.bancho_creator_id,
            bancho_creator_name: value.bancho_creator_name,
        }
    }
}

impl Beatmap {
    const LEADERBOARD_STATUSES: &'static [RankedStatus] = &[
        RankedStatus::Ranked,
        RankedStatus::Approved,
        RankedStatus::Qualified,
        RankedStatus::Loved,
    ];
    const RANKED_STATUSES: &'static [RankedStatus] =
        &[RankedStatus::Ranked, RankedStatus::Approved];

    pub fn has_leaderboards(&self) -> bool {
        Self::LEADERBOARD_STATUSES.contains(&self.status)
    }

    pub fn is_ranked(&self) -> bool {
        Self::RANKED_STATUSES.contains(&self.status)
    }

    pub fn object_count(&self) -> i32 {
        self.count_circles.unwrap_or(0)
            + self.count_sliders.unwrap_or(0)
            + self.count_spinners.unwrap_or(0)
    }

    pub fn are_mods_rankable(&self, mods: Mods) -> bool {
        if mods.has_any(Mods::Autoplay | Mods::Cinema | Mods::TargetPractice) {
            return false;
        }

        match self.mode {
            BeatmapGamemode::Any | BeatmapGamemode::Catch => {
                let object_count = self.object_count();
                let has_sv2 = mods.contains(Mods::ScoreV2);
                (object_count >= 7000) == has_sv2
            }
            _ => !mods.contains(Mods::ScoreV2),
        }
    }

    pub fn set_url(&self) -> String {
        let settings = AppSettings::get();
        let server_url = settings
            .frontend_base_url
            .replace("https://", "")
            .replace("http://", "");
        format!(
            "https://osu.{server_url}/beatmapsets/{}",
            self.beatmap_set_id
        )
    }

    pub fn osu_format(&self) -> String {
        let timestamp =
            DateTime::from_timestamp_secs(self.latest_update).expect("Invalid Timestamp");
        let fmt_ts = timestamp.format("%Y-%m-%d %H:%M:%S");
        format!(
            "beatmapId:{}|beatmapSetId:{}|beatmapPlaycount:{}|beatmapPasscount:{}|approvedDate:{}",
            self.beatmap_id, self.beatmap_set_id, self.playcount, self.passcount, fmt_ts,
        )
    }
}
