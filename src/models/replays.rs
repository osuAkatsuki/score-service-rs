use crate::entities::mode::Gamemode;
use crate::entities::scores::Score;
use crate::models::beatmaps::Beatmap;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};

/// The version of the osu! client the replay files claim to be created by
const OSU_VERSION: i32 = 2021_11_03;

/// The amount of .NET ticks (100 nanoseconds) between 01.01.0001 and the unix epoch
const UNIX_EPOCH_TICKS: i64 = 621_355_968_000_000_000;
const TICKS_PER_SECOND: i64 = 10_000_000;

/// A replay in the format of the `.osr` files of the osu! client
pub struct ReplayFile {
    pub file_name: String,
    pub contents: Vec<u8>,
}

impl ReplayFile {
    pub fn new(score: &Score, username: &str, beatmap: &Beatmap, replay: &[u8]) -> Self {
        let mode = score.mode.as_gamemode();
        let mode_name = match mode {
            Gamemode::Standard => "osu",
            Gamemode::Taiko => "taiko",
            Gamemode::Catch => "fruits",
            Gamemode::Mania => "mania",
        };

        Self {
            file_name: format!(
                "replay-{mode_name}_{}_{}.osr",
                beatmap.beatmap_id, score.score_id
            ),
            contents: encode(score, username, replay),
        }
    }
}

impl IntoResponse for ReplayFile {
    fn into_response(self) -> Response {
        let content_disposition = format!("attachment; filename={}", self.file_name);
        let headers = [
            (CONTENT_TYPE, "application/octet-stream".to_owned()),
            (CONTENT_DISPOSITION, content_disposition),
        ];
        (headers, self.contents).into_response()
    }
}

/// The client verifies replays with a checksum over the result of the play
fn replay_md5(score: &Score, username: &str) -> String {
    let checksum_input = format!(
        "{}p{}o{}o{}t{}a{}r{}e{}y{}o{}u0{}true",
        score.accuracy.count_100 + score.accuracy.count_300,
        score.accuracy.count_50,
        score.accuracy.count_geki,
        score.accuracy.count_katu,
        score.accuracy.count_miss,
        score.beatmap_md5,
        score.max_combo,
        score.perfect_combo,
        username,
        score.score,
        score.mods.bits(),
    );
    format!("{:x}", md5::compute(checksum_input))
}

fn encode(score: &Score, username: &str, replay: &[u8]) -> Vec<u8> {
    let mut contents = Vec::with_capacity(replay.len() + 256);
    contents.push(score.mode.as_gamemode() as u8);
    contents.extend(OSU_VERSION.to_le_bytes());
    write_string(&mut contents, &score.beatmap_md5);
    write_string(&mut contents, username);
    write_string(&mut contents, &replay_md5(score, username));

    contents.extend(score.accuracy.count_300.to_le_bytes());
    contents.extend(score.accuracy.count_100.to_le_bytes());
    contents.extend(score.accuracy.count_50.to_le_bytes());
    contents.extend(score.accuracy.count_geki.to_le_bytes());
    contents.extend(score.accuracy.count_katu.to_le_bytes());
    contents.extend(score.accuracy.count_miss.to_le_bytes());
    contents.extend(score.score.to_le_bytes());
    contents.extend((score.max_combo as u16).to_le_bytes());
    contents.push(score.perfect_combo as u8);
    contents.extend(score.mods.bits().to_le_bytes());
    // The graph of the life bar is not stored
    write_string(&mut contents, "");

    let ticks = score.created_at.timestamp() * TICKS_PER_SECOND + UNIX_EPOCH_TICKS;
    contents.extend(ticks.to_le_bytes());
    contents.extend((replay.len() as i32).to_le_bytes());
    contents.extend(replay);
    contents.extend(score.score_id.to_le_bytes());
    contents
}

/// Strings are prefixed with a marker for whether they are present and their length
fn write_string(contents: &mut Vec<u8>, value: &str) {
    if value.is_empty() {
        contents.push(0x00);
        return;
    }

    contents.push(0x0b);
    write_uleb128(contents, value.len());
    contents.extend(value.as_bytes());
}

fn write_uleb128(contents: &mut Vec<u8>, mut value: usize) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            contents.push(byte);
            break;
        }
        contents.push(byte | 0x80);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::accuracy::Accuracy;
    use crate::entities::mode::ModdedGamemode;
    use crate::entities::mods::Mods;
    use crate::entities::scores::ScoreStatus;
    use chrono::DateTime;

    fn score() -> Score {
        Score {
            score_id: 500_000_123,
            user_id: 1001,
            beatmap_md5: "a5b99395a42bd55bc5eb1d2411cbdf8b".to_owned(),
            mode: ModdedGamemode::Taiko,
            score: 1_234_567,
            max_combo: 512,
            perfect_combo: true,
            mods: Mods::Hidden | Mods::Doubletime,
            accuracy: Accuracy {
                count_300: 300,
                count_100: 10,
                count_50: 5,
                count_geki: 20,
                count_katu: 3,
                count_miss: 1,
            },
            performance: 250.5,
            status: ScoreStatus::Best,
            score_md5: None,
            created_at: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        }
    }

    #[test]
    fn encodes_lengths_as_uleb128() {
        let encoded = |value| {
            let mut contents = Vec::new();
            write_uleb128(&mut contents, value);
            contents
        };
        assert_eq!(encoded(0), [0x00]);
        assert_eq!(encoded(127), [0x7f]);
        assert_eq!(encoded(128), [0x80, 0x01]);
        assert_eq!(encoded(624_485), [0xe5, 0x8e, 0x26]);
    }

    #[test]
    fn marks_strings_as_present_or_missing() {
        let mut contents = Vec::new();
        write_string(&mut contents, "");
        write_string(&mut contents, "osu!");
        assert_eq!(contents, [0x00, 0x0b, 0x04, b'o', b's', b'u', b'!']);
    }

    #[test]
    fn checksums_the_result_of_the_play() {
        // md5 of "310p5o20o3t1aa5b99395a42bd55bc5eb1d2411cbdf8br512etrueycmyuio1234567u072true"
        let expected = format!(
            "{:x}",
            md5::compute(
                "310p5o20o3t1aa5b99395a42bd55bc5eb1d2411cbdf8br512etrueycmyuio1234567u072true"
            )
        );
        assert_eq!(replay_md5(&score(), "cmyui"), expected);
    }

    #[test]
    fn encodes_replays_like_the_osu_client() {
        let score = score();
        let contents = encode(&score, "cmyui", &[1, 2, 3, 4]);

        let mut expected = vec![0x01];
        expected.extend([0x9f, 0x65, 0x34, 0x01]); // 20211103
        expected.extend([0x0b, 32]);
        expected.extend(b"a5b99395a42bd55bc5eb1d2411cbdf8b");
        expected.extend([0x0b, 5]);
        expected.extend(b"cmyui");
        expected.extend([0x0b, 32]);
        expected.extend(replay_md5(&score, "cmyui").as_bytes());
        expected.extend([0x2c, 0x01, 10, 0, 5, 0, 20, 0, 3, 0, 1, 0]);
        expected.extend([0x87, 0xd6, 0x12, 0x00]); // 1234567
        expected.extend([0x00, 0x02]); // 512
        expected.push(1);
        expected.extend([72, 0, 0, 0]);
        expected.push(0x00);
        expected.extend(638_355_968_000_000_000i64.to_le_bytes());
        expected.extend([4, 0, 0, 0, 1, 2, 3, 4]);
        expected.extend(500_000_123i64.to_le_bytes());

        assert_eq!(contents, expected);
    }

    #[test]
    fn names_replay_files_after_the_mode_beatmap_and_score() {
        let beatmap = Beatmap {
            beatmap_id: 75,
            beatmap_set_id: 1,
            beatmap_md5: "a5b99395a42bd55bc5eb1d2411cbdf8b".to_owned(),
            song_name: "Kenji Ninuma - DISCO PRINCE [Normal]".to_owned(),
            file_name: "Kenji Ninuma - DISCOPRINCE (peppy) [Normal].osu".to_owned(),
            ar: 6.0,
            od: 6.0,
            mode: crate::models::beatmaps::BeatmapGamemode::Any,
            max_combo: 314,
            hit_length: 142,
            bpm: 120,
            status: crate::models::beatmaps::RankedStatus::Ranked,
            status_frozen: false,
            latest_update: 0,
            playcount: 10,
            passcount: 5,
            ranked_by: None,
            rating: 8.5,
            bancho_ranked_status: None,
            count_circles: None,
            count_spinners: None,
            count_sliders: None,
            bancho_creator_id: None,
            bancho_creator_name: None,
        };

        let replay_file = ReplayFile::new(&score(), "cmyui", &beatmap, &[]);
        assert_eq!(replay_file.file_name, "replay-taiko_75_500000123.osr");
    }
}
