use serde_repr::Deserialize_repr;
use crate::entities::mods::Mods;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize_repr)]
pub enum Gamemode {
    Standard = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum ModdedGamemode {
    #[default]
    Standard = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
    StandardRelax = 4,
    TaikoRelax = 5,
    CatchRelax = 6,
    StandardAutopilot = 8,
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum GamemodeMod {
    #[default]
    Vanilla = 0,
    Relax = 1,
    Autopilot = 2,
}

impl ModdedGamemode {
    pub const ALL: [ModdedGamemode; 8] = [
        ModdedGamemode::Standard,
        ModdedGamemode::Taiko,
        ModdedGamemode::Catch,
        ModdedGamemode::Mania,
        ModdedGamemode::StandardRelax,
        ModdedGamemode::TaikoRelax,
        ModdedGamemode::CatchRelax,
        ModdedGamemode::StandardAutopilot,
    ];

    pub fn mode_mod(&self) -> GamemodeMod {
        match self {
            ModdedGamemode::Standard
            | ModdedGamemode::Taiko
            | ModdedGamemode::Catch
            | ModdedGamemode::Mania => GamemodeMod::Vanilla,
            ModdedGamemode::StandardRelax
            | ModdedGamemode::TaikoRelax
            | ModdedGamemode::CatchRelax => GamemodeMod::Relax,
            ModdedGamemode::StandardAutopilot => GamemodeMod::Autopilot,
        }
    }

    pub fn as_gamemode(&self) -> Gamemode {
        match self {
            ModdedGamemode::Standard => Gamemode::Standard,
            ModdedGamemode::Taiko => Gamemode::Taiko,
            ModdedGamemode::Catch => Gamemode::Catch,
            ModdedGamemode::Mania => Gamemode::Mania,
            ModdedGamemode::StandardRelax => Gamemode::Standard,
            ModdedGamemode::TaikoRelax => Gamemode::Taiko,
            ModdedGamemode::CatchRelax => Gamemode::Catch,
            ModdedGamemode::StandardAutopilot => Gamemode::Standard,
        }
    }

    pub fn from_mode_and_mods(mode: Gamemode, mods: Mods) -> ModdedGamemode {
        if mods.has_any(Mods::Relax) {
            match mode {
                Gamemode::Standard => ModdedGamemode::StandardRelax,
                Gamemode::Taiko => ModdedGamemode::TaikoRelax,
                Gamemode::Catch => ModdedGamemode::CatchRelax,
                Gamemode::Mania => ModdedGamemode::Mania,
            }
        } else if mods.has_any(Mods::Autopilot) {
            match mode {
                Gamemode::Standard => ModdedGamemode::StandardAutopilot,
                Gamemode::Taiko => ModdedGamemode::Taiko,
                Gamemode::Catch => ModdedGamemode::Catch,
                Gamemode::Mania => ModdedGamemode::Mania,
            }
        } else {
            match mode {
                Gamemode::Standard => ModdedGamemode::Standard,
                Gamemode::Taiko => ModdedGamemode::Taiko,
                Gamemode::Catch => ModdedGamemode::Catch,
                Gamemode::Mania => ModdedGamemode::Mania,
            }
        }
    }

    pub fn scores_table(&self) -> &'static str {
        match self {
            ModdedGamemode::Standard
            | ModdedGamemode::Taiko
            | ModdedGamemode::Catch
            | ModdedGamemode::Mania => "scores",
            ModdedGamemode::StandardRelax
            | ModdedGamemode::TaikoRelax
            | ModdedGamemode::CatchRelax => "scores_relax",
            ModdedGamemode::StandardAutopilot => "scores_ap",
        }
    }

    pub fn redis_board(&self) -> &'static str {
        match self {
            ModdedGamemode::Standard => "leaderboard:std",
            ModdedGamemode::Taiko => "leaderboard:taiko",
            ModdedGamemode::Catch => "leaderboard:ctb",
            ModdedGamemode::Mania => "leaderboard:mania",
            ModdedGamemode::StandardRelax => "relaxboard:std",
            ModdedGamemode::TaikoRelax => "relaxboard:taiko",
            ModdedGamemode::CatchRelax => "relaxboard:ctb",
            ModdedGamemode::StandardAutopilot => "autoboard:std",
        }
    }
}

impl From<u8> for Gamemode {
    fn from(value: u8) -> Self {
        match value {
            0 => Gamemode::Standard,
            1 => Gamemode::Taiko,
            2 => Gamemode::Catch,
            3 => Gamemode::Mania,
            _ => Gamemode::Standard,
        }
    }
}

impl From<i8> for Gamemode {
    fn from(value: i8) -> Self {
        match value {
            0 => Gamemode::Standard,
            1 => Gamemode::Taiko,
            2 => Gamemode::Catch,
            3 => Gamemode::Mania,
            _ => Gamemode::Standard,
        }
    }
}
