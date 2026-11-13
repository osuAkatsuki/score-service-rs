use crate::common::error::{AppError, ServiceResult};
use crate::common::multipart::{FromMultipart, FromMultipartError, FromMultipartField, Multipart};
use crate::common::osu::{DATETIME_FORMAT_STR, format_bool};
use crate::entities::accuracy::Accuracy;
use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;
use crate::models::achievements::Achievement;
use crate::models::beatmaps::Beatmap;
use crate::models::osu::charts::Chart;
use axum::body::Bytes;
use axum::extract::{FromRequest, Multipart as MultipartExtractor, Request};
use axum::response::{IntoResponse, Response};
use bitflags::bitflags;
use chrono::NaiveDateTime;
use simple_rijndael::impls::RijndaelCbc;
use simple_rijndael::paddings::Pkcs7Padding;
use std::str::FromStr;

#[derive(Debug)]
pub struct SubmitScoreArgs {
    pub score: ScoreAttributes,
    pub quit: bool,
    pub fail_time: i32,
    pub score_time: i32,
    pub unique_ids: UniqueIds,
    pub client_info: ClientHashes,
    pub password_md5: String,
    pub beatmap_md5: String,
    pub storyboard_md5: Option<String>,
    pub osu_version: String,
    // pub visual_settings: VisualSettings,
    pub replay_data: Option<Bytes>,
    pub anticheat_token: Option<String>,
    pub event_data: EventData,
}

#[derive(Debug)]
pub struct ScoreAttributes {
    pub beatmap_md5: String,
    pub username: String,
    // In the payload sent by the osu! client,
    // a space will be added after the username.
    // This indicates whether the client thinks they're a supporter
    pub is_donor_flag: bool,
    pub score_md5: String,
    pub stats: ScoreStatistics,
}

#[derive(Debug)]
pub struct ScoreStatistics {
    pub accuracy: Accuracy,
    pub score: i32,
    pub max_combo: u16,
    pub perfect_combo: bool,
    pub achieved_grade: String,
    pub mods: Mods,
    pub passed: bool,
    pub mode: Gamemode,
    // Format: yyMMddHHmmss
    pub client_timestamp: NaiveDateTime,
    pub osu_version: String,
    // In the payload sent by the osu! client,
    // this is represented as spaces following the osu version (lol)
    pub score_flags: ScoreFlags,
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct ScoreFlags: u32 {
        const Clean = 0;
        const SpeedHackDetected = 1 << 1;
        const IncorrectModValue = 1 << 2;
        const MultipleOsuClients = 1 << 3;
        const ChecksumFailure = 1 << 4;
        const FlashlightChecksumIncorrect = 1 << 5;
        const OsuExecutableChecksum = 1 << 6;
        const MissingProcessesInList = 1 << 7;
        const FlashLightImageHack = 1 << 8;
        const SpinnerHack = 1 << 9;
        const TransparentWindow = 1 << 10;
        const FastPress = 1 << 11;
        const RawMouseDiscrepancy = 1 << 12;
        const RawKeyboardDiscrepancy = 1 << 13;
    }
}

#[derive(Debug)]
pub enum ScoreSubmissionResult {
    Ok,
    Charts(Beatmap, Vec<Chart>, Vec<Achievement>),
}

impl ScoreSubmissionResult {
    pub fn osu_format(&self) -> String {
        match self {
            ScoreSubmissionResult::Ok => "ok".to_owned(),
            ScoreSubmissionResult::Charts(beatmap, charts, new_achievements) => {
                let beatmap_fmt = beatmap.osu_format();
                let charts_fmt = charts
                    .iter()
                    .map(|c| c.osu_format())
                    .collect::<Vec<_>>()
                    .join("\n");
                let achievements_fmt = new_achievements
                    .iter()
                    .map(|a| a.osu_format())
                    .collect::<Vec<_>>()
                    .join("/");

                format!("{beatmap_fmt}\n{charts_fmt}|{achievements_fmt}")
            }
        }
    }
}

impl IntoResponse for ScoreSubmissionResult {
    fn into_response(self) -> Response {
        self.osu_format().into_response()
    }
}

impl SubmitScoreArgs {
    pub fn score_md5(&self) -> String {
        let formatted = format!(
            "chickenmcnuggets{0}o15{1}{2}smustard{3}{4}uu{5}{6}{7}{8}{9}{10}{11}Q{12}{13}{15}{14}{16}{17}",
            self.score.stats.accuracy.count_100 + self.score.stats.accuracy.count_300,
            self.score.stats.accuracy.count_50,
            self.score.stats.accuracy.count_geki,
            self.score.stats.accuracy.count_katu,
            self.score.stats.accuracy.count_miss,
            self.score.beatmap_md5,
            self.score.stats.max_combo,
            format_bool(self.score.stats.perfect_combo),
            self.score.username,
            self.score.stats.score,
            self.score.stats.achieved_grade,
            self.score.stats.mods.bits(),
            format_bool(self.score.stats.passed),
            self.score.stats.mode as u8,
            self.score
                .stats
                .client_timestamp
                .format(DATETIME_FORMAT_STR),
            self.score.stats.osu_version,
            self.client_info.client_hash,
            self.storyboard_md5
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(""),
        );
        format!("{:x}", md5::compute(&formatted))
    }
}

impl FromStr for ScoreAttributes {
    type Err = AppError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut split = input.splitn(4, ':');
        let beatmap_md5 = split.next().ok_or(AppError::MalformedRequest)?;
        let username = split.next().ok_or(AppError::MalformedRequest)?;
        let (username, is_donor_flag) = match username.strip_suffix(' ') {
            Some(username) => (username, true),
            None => (username, false),
        };

        let score_md5 = split.next().ok_or(AppError::MalformedRequest)?;
        let score_stats = split.next().ok_or(AppError::MalformedRequest)?;

        let stats = ScoreStatistics::from_str(score_stats)?;
        Ok(Self {
            stats,
            is_donor_flag,
            username: username.to_string(),
            score_md5: score_md5.to_string(),
            beatmap_md5: beatmap_md5.to_string(),
        })
    }
}

impl FromStr for ScoreStatistics {
    type Err = AppError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut split = input.split(':');
        let count_300 = split.next().ok_or(AppError::MalformedRequest)?;
        let count_100 = split.next().ok_or(AppError::MalformedRequest)?;
        let count_50 = split.next().ok_or(AppError::MalformedRequest)?;
        let count_geki = split.next().ok_or(AppError::MalformedRequest)?;
        let count_katu = split.next().ok_or(AppError::MalformedRequest)?;
        let count_miss = split.next().ok_or(AppError::MalformedRequest)?;
        let score = split.next().ok_or(AppError::MalformedRequest)?;
        let max_combo = split.next().ok_or(AppError::MalformedRequest)?;
        let perfect_combo = split.next().ok_or(AppError::MalformedRequest)?;
        let achieved_grade = split.next().ok_or(AppError::MalformedRequest)?;
        let mods = split.next().ok_or(AppError::MalformedRequest)?;
        let passed = split.next().ok_or(AppError::MalformedRequest)?;
        let mode = split.next().ok_or(AppError::MalformedRequest)?;
        let client_timestamp = split.next().ok_or(AppError::MalformedRequest)?;
        let osu_version = split.next().ok_or(AppError::MalformedRequest)?;
        if osu_version.len() < 8 {
            return Err(AppError::MalformedRequest);
        }

        // Parse all params
        let accuracy = Accuracy {
            count_300: count_300
                .parse::<u16>()
                .map_err(|_| AppError::MalformedRequest)?,
            count_100: count_100
                .parse::<u16>()
                .map_err(|_| AppError::MalformedRequest)?,
            count_50: count_50
                .parse::<u16>()
                .map_err(|_| AppError::MalformedRequest)?,
            count_geki: count_geki
                .parse::<u16>()
                .map_err(|_| AppError::MalformedRequest)?,
            count_katu: count_katu
                .parse::<u16>()
                .map_err(|_| AppError::MalformedRequest)?,
            count_miss: count_miss
                .parse::<u16>()
                .map_err(|_| AppError::MalformedRequest)?,
        };
        let score = score
            .parse::<i32>()
            .map_err(|_| AppError::MalformedRequest)?;
        let max_combo = max_combo
            .parse::<u16>()
            .map_err(|_| AppError::MalformedRequest)?;
        let perfect_combo = perfect_combo == "True";
        let mods = mods
            .parse::<u32>()
            .map_err(|_| AppError::MalformedRequest)?;
        let passed = passed == "True";
        let mode = mode.parse::<u8>().map_err(|_| AppError::MalformedRequest)?;
        let client_timestamp = NaiveDateTime::parse_from_str(client_timestamp, DATETIME_FORMAT_STR)
            .map_err(|_| AppError::MalformedRequest)?;
        let score_flags = ScoreFlags::from_bits_retain((osu_version.len() - 8) as _);
        let osu_version = &osu_version[..8];

        Ok(Self {
            score,
            passed,
            accuracy,
            max_combo,
            score_flags,
            perfect_combo,
            client_timestamp,
            mode: Gamemode::from(mode),
            mods: Mods::from_bits_truncate(mods),
            osu_version: osu_version.to_string(),
            achieved_grade: achieved_grade.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct ClientHashes {
    /// This is just the formatted string of the client hashes
    /// sent by the osu! client on score submission,
    /// stored for simplicity
    pub client_hash: String,
    pub osu_path_md5: String,
    pub adapters: NetworkAdapters,
    pub adapters_md5: String,
    // These are actually the md5 of the md5
    pub uninstall_md5: String,
    pub disk_signature_md5: String,
}

#[derive(Debug)]
pub struct NetworkAdapters {
    pub adapters: String,
}

impl FromStr for ClientHashes {
    type Err = AppError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut split = input.split(':');
        let osu_path_md5 = split.next().ok_or(AppError::MalformedRequest)?;
        let adapters = split.next().ok_or(AppError::MalformedRequest)?;
        let adapters_md5 = split.next().ok_or(AppError::MalformedRequest)?;
        let uninstall_md5 = split.next().ok_or(AppError::MalformedRequest)?;
        let disk_signature_md5 = split.next().ok_or(AppError::MalformedRequest)?;

        Ok(Self {
            client_hash: input.to_string(),
            osu_path_md5: osu_path_md5.to_string(),
            adapters: NetworkAdapters {
                adapters: adapters.to_string(),
            },
            adapters_md5: adapters_md5.to_string(),
            uninstall_md5: uninstall_md5.to_string(),
            disk_signature_md5: disk_signature_md5.to_string(),
        })
    }
}

/*
TODO: the received data is somehow not
 in sync with the actual client settings
#[derive(Debug)]
pub struct VisualSettings {
    pub bg_dim: Option<u8>,
    pub disable_samples: bool,
    pub disable_storyboard: bool,
    pub disable_skin: bool,
    pub beatmap_has_storyboard: bool,
}

impl FromStr for VisualSettings {
    type Err = AppError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        println!("{input}");
        let mut split = input.split(':');
        let bg_dim = split.next().ok_or(AppError::MalformedRequest)?;
        let disable_samples = split.next().ok_or(AppError::MalformedRequest)? == "True";
        let bg_visible = split.next().ok_or(AppError::MalformedRequest)? == "True";
        let disable_storyboard = split.next().ok_or(AppError::MalformedRequest)? == "True";
        let disable_skin = split.next().ok_or(AppError::MalformedRequest)? == "True";
        let beatmap_has_storyboard = split.next().ok_or(AppError::MalformedRequest)? == "True";

        let bg_dim = match bg_visible {
            true => Some(bg_dim.parse::<u8>().map_err(|_| AppError::MalformedRequest)?),
            false => None,
        };

        Ok(Self {
            bg_dim,
            disable_samples,
            disable_storyboard,
            disable_skin,
            beatmap_has_storyboard,
        })
    }
}*/

#[derive(Debug)]
pub struct UniqueIds {
    pub uninstall_md5: String,
    pub disk_signature_md5: String,
}

impl FromStr for UniqueIds {
    type Err = AppError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut split = input.split('|');
        let uninstall_md5 = split.next().ok_or(AppError::MalformedRequest)?;
        let disk_signature_md5 = split.next().ok_or(AppError::MalformedRequest)?;
        Ok(Self {
            uninstall_md5: uninstall_md5.to_string(),
            disk_signature_md5: disk_signature_md5.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct EventData {
    pub score_data: String,
    pub visual_settings_b64: String,
    pub iv_b64: String,
    pub unique_ids: String,
    pub client_hash_b64: String,
}

fn decrypt_score_form(form: ScoreSubmissionForm) -> ServiceResult<SubmitScoreArgs> {
    let decryption_key = format!("osu!-scoreburgr---------{}", form.osuver);
    let iv = base64_simd::forgiving_decode_to_vec(form.iv.as_bytes())?;
    if decryption_key.len() != 32 || iv.len() != 32 {
        return Err(AppError::MalformedRequest);
    }

    let decryptor: RijndaelCbc<Pkcs7Padding> = RijndaelCbc::new(decryption_key.as_bytes(), 32)?;

    let score = base64_simd::forgiving_decode_to_vec(form.score.as_bytes())?;
    let decrypted_score = decryptor.decrypt(&iv, score)?;

    let client_info = base64_simd::forgiving_decode_to_vec(form.s.as_bytes())?;
    let decrypted_client_info = decryptor.decrypt(&iv, client_info)?;

    /*let visual_settings = base64_simd::forgiving_decode_to_vec(form.fs.as_bytes())?;
    let decrypted_visual_settings = decryptor.decrypt(&iv, visual_settings)?;*/

    let score = std::str::from_utf8(&decrypted_score)?;
    let score = ScoreAttributes::from_str(score)?;

    let client_info = std::str::from_utf8(&decrypted_client_info)?;
    let client_info = ClientHashes::from_str(client_info)?;

    /*let visual_settings = std::str::from_utf8(&decrypted_visual_settings)?;
    let visual_settings = VisualSettings::from_str(visual_settings)?;*/

    let quit = form.x == "1";
    let fail_time = form.ft.parse::<i32>()?;
    let score_time = form.st.parse::<i32>()?;
    let unique_ids = UniqueIds::from_str(&form.c1)?;
    let event_data = EventData {
        score_data: form.score,
        visual_settings_b64: form.fs,
        iv_b64: form.iv,
        unique_ids: form.c1,
        client_hash_b64: form.s,
    };
    Ok(SubmitScoreArgs {
        quit,
        score,
        fail_time,
        event_data,
        score_time,
        unique_ids,
        client_info,
        // visual_settings,
        beatmap_md5: form.bmk,
        password_md5: form.pass,
        osu_version: form.osuver,
        replay_data: form.replay,
        anticheat_token: form.token,
        storyboard_md5: form.sbk,
    })
}

impl<S: Send + Sync> FromRequest<S> for SubmitScoreArgs {
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match <Multipart<ScoreSubmissionForm>>::from_request(req, state).await {
            Ok(form) => decrypt_score_form(form.0),
            Err(_) => Err(AppError::MalformedRequest),
        }
    }
}

#[derive(Debug)]
pub struct ScoreSubmissionForm {
    pub x: String,
    pub ft: String,
    pub score: String,
    pub fs: String,
    pub bmk: String,
    pub sbk: Option<String>,
    pub c1: String,
    pub st: String,
    pub pass: String,
    pub osuver: String,
    pub s: String,
    pub i: Bytes,
    pub iv: String,
    pub replay: Option<Bytes>,
    pub token: Option<String>,
}

impl FromMultipart for ScoreSubmissionForm {
    async fn from_multipart(mut form: MultipartExtractor) -> Result<Self, FromMultipartError> {
        let mut x = None;
        let mut ft = None;
        let mut score = None;
        let mut fs = None;
        let mut bmk = None;
        let mut sbk = None;
        let mut c1 = None;
        let mut st = None;
        let mut pass = None;
        let mut osuver = None;
        let mut s = None;
        let mut i = None;
        let mut iv = None;
        let mut replay = None;
        let mut token = None;

        while let Some(field) = form.next_field().await? {
            let name = field.name();
            match name {
                Some("score") if field.file_name().is_some() => {
                    replay = Some(Bytes::from_field(field).await?)
                }
                Some("score") => score = Some(String::from_field(field).await?),

                Some("x") => x = Some(String::from_field(field).await?),
                Some("ft") => ft = Some(String::from_field(field).await?),
                Some("fs") => fs = Some(String::from_field(field).await?),
                Some("bmk") => bmk = Some(String::from_field(field).await?),
                Some("sbk") => sbk = Some(String::from_field(field).await?),
                Some("c1") => c1 = Some(String::from_field(field).await?),
                Some("st") => st = Some(String::from_field(field).await?),
                Some("pass") => pass = Some(String::from_field(field).await?),
                Some("osuver") => osuver = Some(String::from_field(field).await?),
                Some("s") => s = Some(String::from_field(field).await?),
                Some("i") => i = Some(Bytes::from_field(field).await?),
                Some("iv") => iv = Some(String::from_field(field).await?),
                Some("token") => token = Some(String::from_field(field).await?),
                _ => continue,
            }
        }

        Ok(Self {
            x: x.ok_or(FromMultipartError::MissingField("x"))?,
            ft: ft.ok_or(FromMultipartError::MissingField("ft"))?,
            score: score.ok_or(FromMultipartError::MissingField("score"))?,
            fs: fs.ok_or(FromMultipartError::MissingField("fs"))?,
            bmk: bmk.ok_or(FromMultipartError::MissingField("bmk"))?,
            c1: c1.ok_or(FromMultipartError::MissingField("c1"))?,
            st: st.ok_or(FromMultipartError::MissingField("st"))?,
            pass: pass.ok_or(FromMultipartError::MissingField("pass"))?,
            osuver: osuver.ok_or(FromMultipartError::MissingField("osuver"))?,
            s: s.ok_or(FromMultipartError::MissingField("s"))?,
            i: i.ok_or(FromMultipartError::MissingField("i"))?,
            iv: iv.ok_or(FromMultipartError::MissingField("iv"))?,
            replay,
            sbk,
            token,
        })
    }
}
