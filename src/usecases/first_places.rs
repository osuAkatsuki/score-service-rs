use crate::adapters::bancho_service;
use crate::api::context::RequestContext;
use crate::common::error::ServiceResult;
use crate::common::website;
use crate::entities::mode::{GamemodeMod, ModdedGamemode};
use crate::entities::mods::Mods;
use crate::entities::scores::Score;
use crate::models::beatmaps::Beatmap;
use crate::models::users::User;
use crate::repositories::first_places;

/// The chat channel first places are announced in
const ANNOUNCEMENT_CHANNEL: &str = "#announce";

pub async fn handle_first_place(
    ctx: &RequestContext,
    beatmap_md5: &str,
    mode: ModdedGamemode,
    score_id: i64,
    user_id: i64,
) -> ServiceResult<()> {
    first_places::replace_first_place(ctx, beatmap_md5, mode, score_id, user_id).await?;
    Ok(())
}

/// Builds the chat message that announces the score as the new first place on the beatmap
pub fn announcement(user: &User, beatmap: &Beatmap, score: &Score) -> String {
    format_announcement(
        score.mode.mode_mod(),
        &website::get_profile_link(user.user_id),
        &user.username,
        &beatmap.url(),
        &beatmap.song_name,
        score.mods,
        score.performance,
    )
}

fn format_announcement(
    mode_mod: GamemodeMod,
    user_url: &str,
    username: &str,
    beatmap_url: &str,
    song_name: &str,
    mods: Mods,
    performance: f32,
) -> String {
    let mode_mod = match mode_mod {
        GamemodeMod::Vanilla => "VN",
        GamemodeMod::Relax => "RX",
        GamemodeMod::Autopilot => "AP",
    };
    let mods = match mods.is_empty() {
        true => "+NM".to_owned(),
        false => mods.to_string(),
    };
    format!(
        "[{mode_mod}] User [{user_url} {username}] has submitted a #1 place \
        on [{beatmap_url} {song_name}] {mods} ({performance:.2}pp)"
    )
}

pub async fn announce(announcement: &str) -> ServiceResult<()> {
    bancho_service::send_message(ANNOUNCEMENT_CHANNEL, announcement).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announces_the_player_beatmap_mods_and_performance() {
        let announcement = format_announcement(
            GamemodeMod::Relax,
            "https://akatsuki.gg/u/1001",
            "cmyui",
            "https://osu.akatsuki.gg/beatmaps/75",
            "Kenji Ninuma - DISCO PRINCE [Normal]",
            Mods::Hidden | Mods::Relax,
            727.277,
        );
        assert_eq!(
            announcement,
            "[RX] User [https://akatsuki.gg/u/1001 cmyui] has submitted a #1 place on \
            [https://osu.akatsuki.gg/beatmaps/75 Kenji Ninuma - DISCO PRINCE [Normal]] +HDRX (727.28pp)"
        );
    }

    #[test]
    fn announces_plays_without_mods_as_nomod() {
        let announcement = format_announcement(
            GamemodeMod::Vanilla,
            "https://akatsuki.gg/u/1001",
            "cmyui",
            "https://osu.akatsuki.gg/beatmaps/75",
            "song",
            Mods::None,
            100.0,
        );
        assert!(announcement.starts_with("[VN] User"));
        assert!(announcement.ends_with("song] +NM (100.00pp)"));
    }
}
