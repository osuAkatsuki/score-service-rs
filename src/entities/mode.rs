use serde_repr::Deserialize_repr;
use crate::entities::mods::Mods;

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Deserialize_repr)]
pub enum Gamemode {
    #[default]
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

/// Every kind of score is stored in a table of its own, each with its own range of ids:
/// relax scores come first, followed by vanilla and finally autopilot scores.
const VANILLA_SCORE_ID_OFFSET: i64 = 500_000_000;
const AUTOPILOT_SCORE_ID_OFFSET: i64 = 6_148_914_691_236_517_204;

impl GamemodeMod {
    /// The id of a score is enough to tell which kind of score it is
    pub fn from_score_id(score_id: i64) -> GamemodeMod {
        if score_id < VANILLA_SCORE_ID_OFFSET {
            GamemodeMod::Relax
        } else if score_id < AUTOPILOT_SCORE_ID_OFFSET {
            GamemodeMod::Vanilla
        } else {
            GamemodeMod::Autopilot
        }
    }

    pub fn scores_table(&self) -> &'static str {
        match self {
            GamemodeMod::Vanilla => "scores",
            GamemodeMod::Relax => "scores_relax",
            GamemodeMod::Autopilot => "scores_ap",
        }
    }
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
        self.mode_mod().scores_table()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tells_the_kind_of_a_score_by_its_id() {
        let kinds = [
            (1, GamemodeMod::Relax),
            (499_999_999, GamemodeMod::Relax),
            (500_000_000, GamemodeMod::Vanilla),
            (6_148_914_691_236_517_203, GamemodeMod::Vanilla),
            (6_148_914_691_236_517_204, GamemodeMod::Autopilot),
        ];
        for (score_id, expected_kind) in kinds {
            assert_eq!(GamemodeMod::from_score_id(score_id), expected_kind);
        }
    }

    #[test]
    fn mods_decide_where_a_score_is_stored() {
        let mode = ModdedGamemode::from_mode_and_mods(Gamemode::Taiko, Mods::Relax | Mods::Hidden);
        assert_eq!(mode, ModdedGamemode::TaikoRelax);
        assert_eq!(mode.scores_table(), "scores_relax");

        // Mods that do not exist in a mode do not change it
        let mode = ModdedGamemode::from_mode_and_mods(Gamemode::Mania, Mods::Relax);
        assert_eq!(mode, ModdedGamemode::Mania);
        assert_eq!(mode.scores_table(), "scores");

        let mode = ModdedGamemode::from_mode_and_mods(Gamemode::Standard, Mods::Autopilot);
        assert_eq!(mode.scores_table(), "scores_ap");
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
