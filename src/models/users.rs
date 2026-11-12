use crate::common::error::AppError;
use crate::entities::mode::{GamemodeMod, ModdedGamemode};
use crate::entities::users::User as UserEntity;
use crate::models::privileges::Privileges;
use bitflags::bitflags;
use chrono::{DateTime, Utc};

bitflags! {
    /// The kinds of game modes a user is exempt from automatic restrictions in
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Whitelist: u8 {
        const Vanilla = 1 << 0;
        const Relax = 1 << 1;
        const Autopilot = 1 << 2;
    }
}

impl Whitelist {
    pub fn is_whitelisted_for(&self, mode: ModdedGamemode) -> bool {
        match mode.mode_mod() {
            GamemodeMod::Vanilla => self.contains(Whitelist::Vanilla),
            GamemodeMod::Relax => self.contains(Whitelist::Relax),
            GamemodeMod::Autopilot => self.contains(Whitelist::Autopilot),
        }
    }
}

#[derive(Debug)]
pub struct User {
    pub user_id: i64,
    pub username: String,
    pub username_safe: String,
    pub register_datetime: i64,
    pub latest_activity: i64,
    pub silence_end: Option<DateTime<Utc>>,
    pub silence_reason: Option<String>,
    pub privileges: Privileges,
    pub donor_expire: i64,
    pub frozen: bool,
    pub notes: Option<String>,
    pub ban_datetime: i64,
    pub previous_overwrite_time: i64,
    pub whitelist: Whitelist,
    pub clan_id: i64,
    pub userpage_allowed: bool,
    pub userpage_content: Option<String>,
    pub freeze_reason: Option<String>,
    pub country: String,
    pub custom_badge_allowed: bool,
    pub show_custom_badge: bool,
    pub custom_badge_icon: String,
    pub custom_badge_name: String,
    pub favourite_mode: i16,
    pub playstyle: i16,
    pub vanilla_pp_leaderboards: bool,
    pub leaderboard_size: Option<i32>,
    pub has_free_username_change: bool,
}

#[repr(i8)]
pub enum VerifiedStatus {
    PendingVerification = -1,
    Multiaccount = 0,
    Verified = 1,
}

impl User {
    pub fn silence_seconds_remaining(&self) -> i64 {
        let now = Utc::now();
        match self.silence_end {
            None => 0,
            Some(silence_end) if silence_end <= now => 0,
            Some(silence_end) => (silence_end - now).num_seconds(),
        }
    }
}

impl TryFrom<UserEntity> for User {
    type Error = AppError;
    fn try_from(value: UserEntity) -> Result<Self, Self::Error> {
        let userpage_content = match value.userpage_content {
            Some(userpage_content) => Some(String::from_utf8(userpage_content)?),
            None => None,
        };
        let silence_end = match value.silence_end {
            None => None,
            Some(silence_end) => match <DateTime<Utc>>::from_timestamp(silence_end, 0) {
                None => None,
                Some(silence_end) if silence_end < Utc::now() => None,
                Some(silence_end) => Some(silence_end),
            },
        };
        Ok(Self {
            silence_end,
            userpage_content,
            user_id: value.id,
            country: value.country,
            username: value.username,
            username_safe: value.username_safe,
            register_datetime: value.register_datetime,
            latest_activity: value.latest_activity,
            silence_reason: value.silence_reason,
            privileges: Privileges::from_bits_retain(value.privileges),
            donor_expire: value.donor_expire,
            frozen: value.frozen,
            notes: value.notes,
            ban_datetime: value.ban_datetime,
            previous_overwrite_time: value.previous_overwrite,
            whitelist: Whitelist::from_bits_truncate(value.whitelist as u8),
            clan_id: value.clan_id,
            userpage_allowed: value.userpage_allowed,
            freeze_reason: value.freeze_reason,
            custom_badge_allowed: value.can_custom_badge,
            show_custom_badge: value.show_custom_badge,
            custom_badge_icon: value.custom_badge_icon,
            custom_badge_name: value.custom_badge_name,
            favourite_mode: value.favourite_mode,
            playstyle: value.play_style,
            vanilla_pp_leaderboards: value.vanilla_pp_leaderboards,
            leaderboard_size: value.leaderboard_size,
            has_free_username_change: value.has_free_username_change,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitelists_apply_per_kind_of_game_mode() {
        let whitelist = Whitelist::from_bits_truncate(5);
        assert!(whitelist.is_whitelisted_for(ModdedGamemode::Mania));
        assert!(!whitelist.is_whitelisted_for(ModdedGamemode::TaikoRelax));
        assert!(whitelist.is_whitelisted_for(ModdedGamemode::StandardAutopilot));

        let whitelist = Whitelist::from_bits_truncate(0);
        assert!(!whitelist.is_whitelisted_for(ModdedGamemode::Standard));
    }
}
