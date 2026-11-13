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
use simple_rijndael::Errors as RijndaelError;
use simple_rijndael::impls::RijndaelCbc;
use simple_rijndael::paddings::Padding;
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
    Charts {
        beatmap: Beatmap,
        score_id: i64,
        beatmap_ranking: Chart,
        overall_ranking: Chart,
        achievements: Vec<Achievement>,
    },
}

impl ScoreSubmissionResult {
    pub fn osu_format(&self) -> String {
        match self {
            ScoreSubmissionResult::Ok => "ok".to_owned(),
            ScoreSubmissionResult::Charts {
                beatmap,
                score_id,
                beatmap_ranking,
                overall_ranking,
                achievements,
            } => {
                let beatmap_fmt = beatmap.osu_format();
                let beatmap_ranking_fmt = beatmap_ranking.osu_format();
                let overall_ranking_fmt = overall_ranking.osu_format();
                let achievements_fmt = achievements
                    .iter()
                    .map(|a| a.osu_format())
                    .collect::<Vec<_>>()
                    .join("/");

                // The beatmap and every chart are on a line of their own.
                // The client expects the id of the score at the end of the beatmap ranking
                // and the unlocked achievements at the end of the overall ranking.
                format!(
                    "{beatmap_fmt}|\n|{beatmap_ranking_fmt}|onlineScoreId:{score_id}|\n\
                    |{overall_ranking_fmt}|achievements-new:{achievements_fmt}"
                )
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
            self.storyboard_md5.as_deref().unwrap_or(""),
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

/// PKCS#7 padding that reports invalid padding instead of panicking like the one of the library.
/// Anything that was not encrypted with the expected key decrypts to garbage.
#[derive(Debug)]
struct StrictPkcs7Padding(usize);

impl Padding for StrictPkcs7Padding {
    fn new(block_size: usize) -> Self {
        Self(block_size)
    }

    fn size(&self) -> usize {
        self.0
    }

    fn encode(&self, mut input_vec: Vec<u8>) -> Vec<u8> {
        let pad_size = self.0 - (input_vec.len() % self.0);
        input_vec.resize(input_vec.len() + pad_size, pad_size as u8);
        input_vec
    }

    fn decode(&self, mut source: Vec<u8>) -> Result<Vec<u8>, RijndaelError> {
        let pad_size = source.last().copied().unwrap_or(0) as usize;
        if pad_size == 0 || pad_size > self.0 || pad_size > source.len() {
            return Err(RijndaelError::InvalidDataSize);
        }

        source.truncate(source.len() - pad_size);
        Ok(source)
    }
}

type ScoreDecryptor = RijndaelCbc<StrictPkcs7Padding>;

fn decrypt(decryptor: &ScoreDecryptor, iv: &[u8], encrypted_b64: &str) -> ServiceResult<String> {
    let encrypted = base64_simd::forgiving_decode_to_vec(encrypted_b64.as_bytes())
        .map_err(|_| AppError::MalformedRequest)?;
    // The library expects there to be at least one block
    if encrypted.is_empty() {
        return Err(AppError::MalformedRequest);
    }

    let decrypted = decryptor
        .decrypt(iv, encrypted)
        .map_err(|_| AppError::MalformedRequest)?;
    String::from_utf8(decrypted).map_err(|_| AppError::MalformedRequest)
}

fn decrypt_score_form(form: ScoreSubmissionForm) -> ServiceResult<SubmitScoreArgs> {
    let decryption_key = format!("osu!-scoreburgr---------{}", form.osuver);
    let iv = base64_simd::forgiving_decode_to_vec(form.iv.as_bytes())
        .map_err(|_| AppError::MalformedRequest)?;
    if decryption_key.len() != 32 || iv.len() != 32 {
        return Err(AppError::MalformedRequest);
    }

    let decryptor: ScoreDecryptor = RijndaelCbc::new(decryption_key.as_bytes(), 32)?;

    let score = decrypt(&decryptor, &iv, &form.score)?;
    let score = ScoreAttributes::from_str(&score)?;

    let client_info = decrypt(&decryptor, &iv, &form.s)?;
    let client_info = ClientHashes::from_str(&client_info)?;

    /*let visual_settings = decrypt(&decryptor, &iv, &form.fs)?;
    let visual_settings = VisualSettings::from_str(&visual_settings)?;*/

    let quit = form.x == "1";
    let fail_time = form
        .ft
        .parse::<i32>()
        .map_err(|_| AppError::MalformedRequest)?;
    let score_time = form
        .st
        .parse::<i32>()
        .map_err(|_| AppError::MalformedRequest)?;
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
        // The client sends its anticheat token next to the form
        let anticheat_token = req
            .headers()
            .get("token")
            .and_then(|token| token.to_str().ok())
            .map(str::to_owned);

        match <Multipart<ScoreSubmissionForm>>::from_request(req, state).await {
            Ok(form) => {
                let mut args = decrypt_score_form(form.0)?;
                args.anticheat_token = anticheat_token.or(args.anticheat_token);
                Ok(args)
            }
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
    /// A screenshot the client only attaches when it suspects a flashlight cheat
    pub i: Option<Bytes>,
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
            iv: iv.ok_or(FromMultipartError::MissingField("iv"))?,
            i,
            replay,
            sbk,
            token,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::beatmaps::{BeatmapGamemode, RankedStatus};
    use crate::models::osu::charts::ChartEntry;
    use simple_rijndael::paddings::Pkcs7Padding;

    const SCORE_PAYLOAD: &str = "a5b99395a42bd55bc5eb1d2411cbdf8b:cmyui :6fa4a52a3e7e4e0a0f4d8c0a5b9f7e11:\
        300:10:5:20:3:1:1234567:512:True:A:72:True:0:231114221320:20231111";
    const CLIENT_PAYLOAD: &str = "path:adapters.:adapters_md5:uninstall_md5:disk_md5";

    fn beatmap() -> Beatmap {
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
            latest_update: 1_700_000_000,
            playcount: 10,
            passcount: 5,
            ranked_by: None,
            rating: 8.5,
            bancho_ranked_status: None,
            count_circles: Some(160),
            count_spinners: Some(4),
            count_sliders: Some(30),
            bancho_creator_id: None,
            bancho_creator_name: None,
        }
    }

    fn achievement(icon_file: &str, name: &str, description: &str) -> Achievement {
        Achievement {
            id: 1,
            icon_file: icon_file.to_owned(),
            name: name.to_owned(),
            description: description.to_owned(),
            hidden: false,
            mode: None,
        }
    }

    // Encrypts with the padding of the library, which is what the client is compatible with
    fn encrypt(encryptor: &RijndaelCbc<Pkcs7Padding>, iv: &[u8], payload: &str) -> String {
        let encrypted = encryptor.encrypt(iv, payload.as_bytes().to_vec()).unwrap();
        base64_simd::STANDARD.encode_to_string(encrypted)
    }

    fn form(score_payload: &str) -> ScoreSubmissionForm {
        let osu_version = "20231111";
        let key = format!("osu!-scoreburgr---------{osu_version}");
        let iv: Vec<u8> = (0..32).collect();
        let encryptor: RijndaelCbc<Pkcs7Padding> = RijndaelCbc::new(key.as_bytes(), 32).unwrap();

        ScoreSubmissionForm {
            x: "0".to_owned(),
            ft: "0".to_owned(),
            score: encrypt(&encryptor, &iv, score_payload),
            fs: "visual settings".to_owned(),
            bmk: "a5b99395a42bd55bc5eb1d2411cbdf8b".to_owned(),
            sbk: None,
            c1: "uninstall_id|disk_signature".to_owned(),
            st: "142000".to_owned(),
            pass: "5f4dcc3b5aa765d61d8327deb882cf99".to_owned(),
            osuver: osu_version.to_owned(),
            s: encrypt(&encryptor, &iv, CLIENT_PAYLOAD),
            i: None,
            iv: base64_simd::STANDARD.encode_to_string(&iv),
            replay: Some(Bytes::from_static(b"replay")),
            token: None,
        }
    }

    #[test]
    fn decrypts_the_score_of_a_submission() {
        let args = decrypt_score_form(form(SCORE_PAYLOAD)).unwrap();

        assert_eq!(args.score.username, "cmyui");
        assert!(args.score.is_donor_flag);
        assert_eq!(args.score.beatmap_md5, "a5b99395a42bd55bc5eb1d2411cbdf8b");
        assert_eq!(args.score.score_md5, "6fa4a52a3e7e4e0a0f4d8c0a5b9f7e11");

        let stats = &args.score.stats;
        assert_eq!(stats.accuracy.count_300, 300);
        assert_eq!(stats.accuracy.count_100, 10);
        assert_eq!(stats.accuracy.count_50, 5);
        assert_eq!(stats.accuracy.count_geki, 20);
        assert_eq!(stats.accuracy.count_katu, 3);
        assert_eq!(stats.accuracy.count_miss, 1);
        assert_eq!(stats.score, 1_234_567);
        assert_eq!(stats.max_combo, 512);
        assert!(stats.perfect_combo);
        assert!(stats.passed);
        assert_eq!(stats.mods, Mods::Hidden | Mods::Doubletime);
        assert_eq!(stats.mode, Gamemode::Standard);
        assert_eq!(stats.osu_version, "20231111");
        assert_eq!(stats.score_flags, ScoreFlags::Clean);

        assert!(!args.quit);
        assert_eq!(args.score_time, 142_000);
        assert_eq!(args.unique_ids.uninstall_md5, "uninstall_id");
        assert_eq!(args.unique_ids.disk_signature_md5, "disk_signature");
        assert_eq!(args.client_info.uninstall_md5, "uninstall_md5");
        assert_eq!(args.client_info.disk_signature_md5, "disk_md5");
    }

    #[test]
    fn reads_anticheat_flags_from_the_padding_of_the_version() {
        let payload = format!("{SCORE_PAYLOAD}  ");
        let args = decrypt_score_form(form(&payload)).unwrap();
        assert_eq!(args.score.stats.osu_version, "20231111");
        assert_eq!(args.score.stats.score_flags, ScoreFlags::SpeedHackDetected);
    }

    #[test]
    fn rejects_scores_with_missing_fields() {
        let payload = "a5b99395a42bd55bc5eb1d2411cbdf8b:cmyui:checksum:300:10:5";
        let result = decrypt_score_form(form(payload));
        assert!(matches!(result, Err(AppError::MalformedRequest)));
    }

    #[test]
    fn rejects_forms_that_were_not_encrypted_for_their_version() {
        let mut form = form(SCORE_PAYLOAD);
        form.osuver = "20200101".to_owned();
        let result = decrypt_score_form(form);
        assert!(matches!(result, Err(AppError::MalformedRequest)));
    }

    #[test]
    fn rejects_forms_without_a_score() {
        let mut form = form(SCORE_PAYLOAD);
        form.score = String::new();
        let result = decrypt_score_form(form);
        assert!(matches!(result, Err(AppError::MalformedRequest)));
    }

    #[test]
    fn pads_like_the_client() {
        let padding = StrictPkcs7Padding::new(32);
        let padded = padding.encode(b"osu!".to_vec());
        assert_eq!(padded.len(), 32);
        assert_eq!(padded, Pkcs7Padding::new(32).encode(b"osu!".to_vec()));
        assert_eq!(padding.decode(padded).unwrap(), b"osu!");

        // A full block of padding follows data that already fills its blocks
        assert_eq!(padding.encode(vec![1; 32]).len(), 64);
    }

    #[test]
    fn formats_charts_the_way_the_client_reads_them() {
        let result = ScoreSubmissionResult::Charts {
            beatmap: beatmap(),
            score_id: 1001,
            beatmap_ranking: Chart {
                chart_id: "beatmap",
                chart_url: "https://osu.akatsuki.gg/beatmapsets/1".to_owned(),
                chart_name: "Beatmap Ranking",
                entries: vec![
                    ChartEntry::new_int("rank", None, 1),
                    ChartEntry::new_float("accuracy", Some(98.123), 99.5),
                ],
            },
            overall_ranking: Chart {
                chart_id: "overall",
                chart_url: "https://akatsuki.gg/u/1001".to_owned(),
                chart_name: "Overall Ranking",
                entries: vec![ChartEntry::new_int("pp", Some(100), 120)],
            },
            achievements: vec![
                achievement("osu-skill-pass-1", "Rising Star", "Can't go forward without the first steps."),
                achievement("all-intro-hidden", "Blindsight", "I can see just perfectly"),
            ],
        };

        let expected = "beatmapId:75|beatmapSetId:1|beatmapPlaycount:10|beatmapPasscount:5\
            |approvedDate:2023-11-14 22:13:20|\n\
            |chartId:beatmap|chartUrl:https://osu.akatsuki.gg/beatmapsets/1|chartName:Beatmap Ranking\
            |rankBefore:|rankAfter:1|accuracyBefore:98.12|accuracyAfter:99.5|onlineScoreId:1001|\n\
            |chartId:overall|chartUrl:https://akatsuki.gg/u/1001|chartName:Overall Ranking\
            |ppBefore:100|ppAfter:120\
            |achievements-new:osu-skill-pass-1+Rising Star+Can't go forward without the first steps.\
            /all-intro-hidden+Blindsight+I can see just perfectly";
        assert_eq!(result.osu_format(), expected);
    }

    #[test]
    fn formats_submissions_without_charts_as_ok() {
        assert_eq!(ScoreSubmissionResult::Ok.osu_format(), "ok");
    }
}
