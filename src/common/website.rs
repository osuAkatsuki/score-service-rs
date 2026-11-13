use crate::settings::AppSettings;

pub fn get_avatar_link(user_id: i64) -> String {
    format!("https://a.akatsuki.gg/{user_id}")
}

pub fn get_profile_link(user_id: i64) -> String {
    let frontend_base = &AppSettings::get().frontend_base_url;
    format!("{frontend_base}/u/{user_id}")
}

pub fn get_match_history_link(match_id: i64) -> String {
    let frontend_base = &AppSettings::get().frontend_base_url;
    format!("{frontend_base}/matches/{match_id}")
}

pub fn get_beatmap_link(beatmap_id: i32) -> String {
    let frontend_base = &AppSettings::get().frontend_base_url;
    format!("{frontend_base}/b/{beatmap_id}")
}

/// Beatmapsets are downloaded from the beatmap mirror, which is a subdomain of the frontend
pub fn get_beatmapset_download_link(beatmapset_id: i32) -> String {
    let frontend_base = &AppSettings::get().frontend_base_url;
    let server_domain = frontend_base
        .replace("https://", "")
        .replace("http://", "");
    format!("https://beatmaps.{server_domain}/api/d/{beatmapset_id}")
}

pub fn get_beatmapset_link(beatmapset_id: i32) -> String {
    let frontend_base = &AppSettings::get().frontend_base_url;
    format!("{frontend_base}/s/{beatmapset_id}")
}
