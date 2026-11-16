use crate::entities::mode::{GamemodeMod, ModdedGamemode};
use crate::entities::users::User as UserEntity;
use crate::models::privileges::Privileges;
use bitflags::bitflags;

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
    pub privileges: Privileges,
    pub whitelist: Whitelist,
    pub country: String,
    pub vanilla_pp_leaderboards: bool,
    pub leaderboard_size: Option<i32>,
}

#[repr(i8)]
pub enum VerifiedStatus {
    PendingVerification = -1,
    Multiaccount = 0,
    Verified = 1,
}

impl From<UserEntity> for User {
    fn from(value: UserEntity) -> Self {
        Self {
            user_id: value.id,
            username: value.username,
            privileges: Privileges::from_bits_retain(value.privileges),
            whitelist: Whitelist::from_bits_truncate(value.whitelist as u8),
            country: value.country,
            vanilla_pp_leaderboards: value.vanilla_pp_leaderboards,
            leaderboard_size: value.leaderboard_size,
        }
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
