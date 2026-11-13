use crate::entities::accuracy::Accuracy;
use crate::entities::mode::{Gamemode, ModdedGamemode};
use crate::entities::mods::Mods;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::{Database, Encode, Error, MySql, Type};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScoreGrade {
    XH,
    X,
    SH,
    S,
    A,
    B,
    C,
    D,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScoreStatus {
    /// The play was abandoned before the end of the beatmap
    Quit = 0,
    Failed = 1,
    /// A pass that is not the best score of the user on the beatmap
    Completed = 2,
    Best = 3,
}

impl ScoreStatus {
    pub const fn is_pass(&self) -> bool {
        matches!(self, ScoreStatus::Completed | ScoreStatus::Best)
    }
}

pub struct Score {
    pub score_id: i64,
    pub user_id: i64,
    pub beatmap_md5: String,
    pub mode: ModdedGamemode,
    pub score: i32,
    pub max_combo: i32,
    pub perfect_combo: bool,
    pub mods: Mods,
    pub accuracy: Accuracy,
    pub performance: f32,
    pub status: ScoreStatus,
    pub score_md5: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct CreateScoreArgs<'a> {
    pub user_id: i64,
    pub beatmap_md5: &'a str,
    pub mode: ModdedGamemode,
    pub score: i32,
    pub accuracy: Accuracy,
    pub max_combo: u16,
    pub perfect_combo: bool,
    pub mods: Mods,
    pub performance: f32,
    pub score_md5: &'a str,
    pub status: ScoreStatus,
}

#[derive(Clone)]
pub struct LeaderboardScore {
    /// The position of this score on the leaderboard it was fetched from
    pub rank: i64,
    pub username: String,
    pub clan_tag: Option<String>,
    pub score_id: i64,
    pub user_id: i64,
    pub score: i32,
    pub max_combo: i32,
    pub perfect_combo: bool,
    pub mods: Mods,
    pub accuracy: Accuracy,
    pub performance: f32,
    pub created_at: DateTime<Utc>,
}

pub enum ScoreRankingMetric {
    Performance(f32),
    Score(i32),
}

pub enum LeaderboardFilter<'a> {
    None,
    Friends(Vec<i32>),
    Mods(Mods),
    Country(&'a str),
}

impl LeaderboardFilter<'_> {
    pub fn query_part(&self) -> String {
        match self {
            LeaderboardFilter::None => String::new(),
            LeaderboardFilter::Friends(friends) => format!("AND s.userid IN (?{})", ", ?".repeat(friends.len())),
            LeaderboardFilter::Mods(_) => "AND s.mods = ?".to_owned(),
            LeaderboardFilter::Country(_) => "AND users.country = ?".to_owned(),
        }
    }

    /// The condition scores have to meet on their `completed` column.
    /// A user's best score with specific mods is not necessarily their best score on the beatmap.
    pub const fn status_query_part(&self) -> &'static str {
        match self {
            LeaderboardFilter::Mods(_) => "IN (2, 3)",
            _ => "= 3",
        }
    }
}

impl LeaderboardScore {
    pub fn osu_format(&self, include_clan_tag: bool, display_pp_as_score: bool) -> String {
        let score_username = match self.clan_tag.as_ref() {
            Some(clan_tag) if include_clan_tag => &format!("[{}] {}", clan_tag, self.username),
            _ => &self.username,
        };
        let display_score = match display_pp_as_score {
            true => self.performance as i32,
            false => self.score,
        };
        format!(
            "{}|{score_username}|{display_score}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|1",
            self.score_id,
            self.max_combo,
            self.accuracy.count_50,
            self.accuracy.count_100,
            self.accuracy.count_300,
            self.accuracy.count_miss,
            self.accuracy.count_katu,
            self.accuracy.count_geki,
            self.perfect_combo as u8,
            self.mods.bits(),
            self.user_id,
            self.rank,
            self.created_at.timestamp(),
        )
    }
}

impl ScoreRankingMetric {
    pub const fn column_name(&self) -> &'static str {
        match self {
            Self::Performance(_) => "pp",
            Self::Score(_) => "score",
        }
    }
}

impl Encode<'_, MySql> for ScoreRankingMetric {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        match self {
            Self::Performance(v) => <f32 as Encode<MySql>>::encode_by_ref(v, buf),
            Self::Score(v) => <i32 as Encode<MySql>>::encode_by_ref(v, buf),
        }
    }

    fn produces(&self) -> Option<<MySql as Database>::TypeInfo> {
        match self {
            ScoreRankingMetric::Performance(_) => Some(<f32 as Type<MySql>>::type_info()),
            ScoreRankingMetric::Score(_) => Some(<i32 as Type<MySql>>::type_info()),
        }
    }
}

impl Type<MySql> for ScoreRankingMetric {
    fn type_info() -> <MySql as Database>::TypeInfo {
        <i32 as Type<MySql>>::type_info()
    }
}

impl ScoreGrade {
    pub const fn column_name(&self) -> &'static str {
        match self {
            ScoreGrade::XH => "xh_count",
            ScoreGrade::X => "x_count",
            ScoreGrade::SH => "sh_count",
            ScoreGrade::S => "s_count",
            ScoreGrade::A => "a_count",
            ScoreGrade::B => "b_count",
            ScoreGrade::C => "c_count",
            ScoreGrade::D => "d_count",
        }
    }
}

impl From<i8> for ScoreStatus {
    fn from(value: i8) -> Self {
        match value {
            0 => ScoreStatus::Quit,
            1 => ScoreStatus::Failed,
            2 => ScoreStatus::Completed,
            3 => ScoreStatus::Best,
            _ => ScoreStatus::Failed,
        }
    }
}

impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for Score {
    fn from_row(row: &sqlx::mysql::MySqlRow) -> Result<Self, Error> {
        let raw_score = RawScore::from_row(row)?;
        let mode = Gamemode::from(raw_score.play_mode);
        let mods = Mods::from_bits_retain(raw_score.mods as _);
        let mode = ModdedGamemode::from_mode_and_mods(mode, mods);
        Ok(Score {
            mode,
            mods,
            score_id: raw_score.id,
            user_id: raw_score.user_id,
            beatmap_md5: raw_score.beatmap_md5.to_owned(),
            score: raw_score.score,
            max_combo: raw_score.max_combo,
            perfect_combo: raw_score.full_combo,
            accuracy: Accuracy {
                count_300: raw_score.count_300 as _,
                count_100: raw_score.count_100 as _,
                count_50: raw_score.count_50 as _,
                count_geki: raw_score.count_geki as _,
                count_katu: raw_score.count_katu as _,
                count_miss: raw_score.count_miss as _,
            },
            performance: raw_score.pp,
            status: ScoreStatus::from(raw_score.completed),
            score_md5: raw_score.checksum,
            created_at: DateTime::from_timestamp(raw_score.time as _, 0)
                .unwrap_or_default(),
        })
    }
}

impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for LeaderboardScore {
    fn from_row(row: &sqlx::mysql::MySqlRow) -> Result<Self, Error> {
        let raw_score = RawLeaderboardScore::from_row(row)?;
        let mods = Mods::from_bits_retain(raw_score.mods as _);
        Ok(LeaderboardScore {
            mods,
            rank: raw_score.score_rank,
            username: raw_score.username,
            clan_tag: raw_score.clan_tag,
            score_id: raw_score.score_id,
            user_id: raw_score.user_id,
            score: raw_score.score,
            max_combo: raw_score.max_combo,
            perfect_combo: raw_score.full_combo,
            accuracy: Accuracy {
                count_300: raw_score.count_300 as _,
                count_100: raw_score.count_100 as _,
                count_50: raw_score.count_50 as _,
                count_geki: raw_score.count_geki as _,
                count_katu: raw_score.count_katu as _,
                count_miss: raw_score.count_miss as _,
            },
            performance: raw_score.pp,
            created_at: DateTime::from_timestamp(raw_score.time as _, 0)
                .unwrap_or_default(),
        })
    }
}

#[derive(sqlx::FromRow)]
struct RawScore {
    pub id: i64,
    pub beatmap_md5: String,
    #[sqlx(rename = "userid")]
    pub user_id: i64,
    pub score: i32,
    pub max_combo: i32,
    pub full_combo: bool,
    pub mods: i32,
    #[sqlx(rename = "300_count")]
    pub count_300: i16,
    #[sqlx(rename = "100_count")]
    pub count_100: i16,
    #[sqlx(rename = "50_count")]
    pub count_50: i16,
    #[sqlx(rename = "gekis_count")]
    pub count_geki: i16,
    #[sqlx(rename = "katus_count")]
    pub count_katu: i16,
    #[sqlx(rename = "misses_count")]
    pub count_miss: i16,
    pub time: i32,
    pub play_mode: i8,
    pub completed: i8,
    pub accuracy: f32,
    pub pp: f32,
    pub checksum: Option<String>,
}

#[derive(sqlx::FromRow)]
struct RawLeaderboardScore {
    pub score_rank: i64,
    pub username: String,
    pub clan_tag: Option<String>,
    pub score_id: i64,
    pub user_id: i64,
    pub score: i32,
    pub max_combo: i32,
    pub full_combo: bool,
    pub mods: i32,
    #[sqlx(rename = "300_count")]
    pub count_300: i16,
    #[sqlx(rename = "100_count")]
    pub count_100: i16,
    #[sqlx(rename = "50_count")]
    pub count_50: i16,
    #[sqlx(rename = "gekis_count")]
    pub count_geki: i16,
    #[sqlx(rename = "katus_count")]
    pub count_katu: i16,
    #[sqlx(rename = "misses_count")]
    pub count_miss: i16,
    pub time: i32,
    pub pp: f32,
}

#[derive(Serialize)]
pub struct SubmissionEvent<'a> {
    pub score_data: &'a str,
    pub exited_out: bool,
    pub fail_time: i32,
    pub visual_settings_b64: &'a str,
    pub updated_beatmap_hash: &'a str,
    pub storyboard_md5: Option<&'a str>,
    pub iv_b64: &'a str,
    pub unique_ids: &'a str,
    pub score_time: i32,
    pub osu_version: &'a str,
    pub client_hash_b64: &'a str,
    pub score_id: i64,
    pub user_id: i64,
    pub osu_auth_token: Option<&'a str>,
    pub mode_vn: u8,
    pub relax: u8,
}