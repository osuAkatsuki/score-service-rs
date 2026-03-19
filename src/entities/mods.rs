use crate::entities::mode::Gamemode;
use bitflags::bitflags;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Error, ErrorKind};
use std::str::FromStr;

bitflags! {
    #[derive(Default, Copy, Clone, Eq, PartialEq)]
    pub struct Mods: u32 {
        const None = 0;
        const NoFail = 1 << 0;
        const Easy = 1 << 1;
        const TouchDevice = 1 << 2;
        const Hidden = 1 << 3;
        const HardRock = 1 << 4;
        const SuddenDeath = 1 << 5;
        const Doubletime = 1 << 6;
        const Relax = 1 << 7;
        const Halftime = 1 << 8;
        const Nightcore = 1 << 9;
        const Flashlight = 1 << 10;
        const Autoplay = 1 << 11;
        const SpunOut = 1 << 12;
        const Autopilot = 1 << 13;
        const Perfect = 1 << 14;
        const Keys4 = 1 << 15;
        const Keys5 = 1 << 16;
        const Keys6 = 1 << 17;
        const Keys7 = 1 << 18;
        const Keys8 = 1 << 19;
        const FadeIn = 1 << 20;
        const Random = 1 << 21;
        const Cinema = 1 << 22;
        const TargetPractice = 1 << 23;
        const Keys9 = 1 << 24;
        const Coop = 1 << 25;
        const Key1 = 1 << 26;
        const Keys3 = 1 << 27;
        const Keys2 = 1 << 28;
        const ScoreV2 = 1 << 29;
        const Mirror = 1 << 30;
    }
}

pub const ALLOWED_MODS_ALL_MODES: Mods = Mods::from_bits_retain(
    Mods::NoFail.bits()
        | Mods::Easy.bits()
        | Mods::Hidden.bits()
        | Mods::HardRock.bits()
        | Mods::SuddenDeath.bits()
        | Mods::Perfect.bits()
        | Mods::Halftime.bits()
        | Mods::Doubletime.bits()
        | Mods::Nightcore.bits()
        | Mods::Flashlight.bits()
        | Mods::Autoplay.bits()
        | Mods::Cinema.bits()
        | Mods::ScoreV2.bits(),
);

pub const STANDARD_ALLOWED_MODS: Mods = Mods::from_bits_retain(
    ALLOWED_MODS_ALL_MODES.bits()
        | Mods::TouchDevice.bits()
        | Mods::SpunOut.bits()
        | Mods::Relax.bits()
        | Mods::Autopilot.bits(),
);
pub const TAIKO_ALLOWED_MODS: Mods =
    Mods::from_bits_retain(ALLOWED_MODS_ALL_MODES.bits() | Mods::Relax.bits());
pub const CATCH_ALLOWED_MODS: Mods =
    Mods::from_bits_retain(ALLOWED_MODS_ALL_MODES.bits() | Mods::Relax.bits());
pub const MANIA_ALLOWED_MODS: Mods = Mods::from_bits_retain(
    ALLOWED_MODS_ALL_MODES.bits()
        | Mods::Key1.bits()
        | Mods::Keys2.bits()
        | Mods::Keys3.bits()
        | Mods::Keys4.bits()
        | Mods::Keys5.bits()
        | Mods::Keys6.bits()
        | Mods::Keys7.bits()
        | Mods::Keys8.bits()
        | Mods::Keys9.bits()
        | Mods::Coop.bits()
        | Mods::FadeIn.bits()
        | Mods::Random.bits()
        | Mods::Mirror.bits(),
);

impl Mods {
    pub fn has_all(&self, mods: Mods) -> bool {
        self.contains(mods)
    }

    pub fn has_any(&self, mods: Mods) -> bool {
        self.intersects(mods)
    }

    pub fn is_illegal_combination(&self) -> bool {
        self.has_all(Mods::Relax | Mods::Autopilot)
            || self.has_all(Mods::Easy | Mods::HardRock)
            || self.has_all(Mods::NoFail | Mods::SuddenDeath)
            || self.has_all(Mods::NoFail | Mods::Perfect)
            || self.has_all(Mods::Halftime | Mods::Doubletime)
            || self.has_all(Mods::Halftime | Mods::Nightcore)
            || self.has_all(Mods::Autopilot | Mods::SpunOut)
            || self.has_all(Mods::FadeIn | Mods::Flashlight)
    }

    pub fn sanitize_for_mode(self, mode: Gamemode) -> Mods {
        match mode {
            Gamemode::Standard => self.intersection(STANDARD_ALLOWED_MODS),
            Gamemode::Taiko => self.intersection(TAIKO_ALLOWED_MODS),
            Gamemode::Catch => self.intersection(CATCH_ALLOWED_MODS),
            Gamemode::Mania => self.intersection(MANIA_ALLOWED_MODS),
        }
    }
}

pub const MODS_SHORT: [&'static str; 31] = [
    "NF", "EZ", "TD", "HD", "HR", "SD", "DT", "RX", "HT", "NC", "FL", "AT", "SO", "AP", "PF", "4K",
    "5K", "6K", "7K", "8K", "FI", "RN", "CN", "TP", "9K", "CO", "1K", "3K", "2K", "V2", "MR",
];

pub const NP_MODS: [&str; 31] = [
    "-NoFail",
    "-Easy",
    "~TouchDevice~",
    "+Hidden",
    "+HardRock",
    "+SuddenDeath",
    "+DoubleTime",
    "~Relax~",
    "-HalfTime",
    "+Nightcore",
    "+Flashlight",
    "|Autoplay|",
    "-SpunOut",
    "~Autopilot~",
    "+Perfect",
    "|4K|",
    "|5K|",
    "|6K|",
    "|7K|",
    "|8K|",
    "+FadeIn",
    "~Random~",
    "|Cinema|",
    "~Target~",
    "|9K|",
    "|Coop|",
    "|1K|",
    "|3K|",
    "|2K|",
    "~ScoreV2~",
    "~Mirror~",
];

impl Mods {
    pub fn from_code<T: AsRef<[u8]>>(code: T) -> Mods {
        MODS_SHORT
            .into_iter()
            .position(|x| x.as_bytes() == code.as_ref())
            .map(|pos| Mods::from_bits_truncate(1 << pos))
            .unwrap_or(Mods::None)
    }

    pub fn from_np(np_mod: &str) -> Mods {
        match NP_MODS.into_iter().position(|mod_str| mod_str == np_mod) {
            None => Mods::None,
            Some(mod_index) => Mods::from_bits_truncate(1 << mod_index),
        }
    }
}

impl FromStr for Mods {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(Error::new(ErrorKind::InvalidInput, "invalid mod input"));
        }

        Ok(s.as_bytes()
            .chunks(2)
            .fold(Mods::None, |mods, mod_str| mods | Mods::from_code(mod_str)))
    }
}

const MODS_DEBUG: [&'static str; 31] = [
    "NoFail",
    "Easy",
    "TouchDevice",
    "Hidden",
    "HardRock",
    "SuddenDeath",
    "Doubletime",
    "Relax",
    "Halftime",
    "Nightcore",
    "Flashlight",
    "Autoplay",
    "SpunOut",
    "Autopilot",
    "Perfect",
    "Keys4",
    "Keys5",
    "Keys6",
    "Keys7",
    "Keys8",
    "FadeIn",
    "Random",
    "Cinema",
    "TargetPractice",
    "Keys9",
    "Coop",
    "Key1",
    "Keys3",
    "Keys2",
    "ScoreV2",
    "Mirror",
];

impl Debug for Mods {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut mods = vec![];

        let v = self.bits();
        for i in 0..MODS_DEBUG.len() {
            let debug_str = MODS_DEBUG[i];
            if (v & (1 << i)) != 0 {
                mods.push(debug_str);
            }
        }

        Debug::fmt(&mods, f)
    }
}

impl Display for Mods {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut mods = vec![];

        let v = self.bits();
        for i in 0..MODS_SHORT.len() {
            let short_str = MODS_SHORT[i];
            if (v & (1 << i)) != 0 {
                mods.push(short_str);
            }
        }

        if mods.is_empty() {
            return Ok(());
        }

        // TODO: adjust logic here a bit for DTNC/SDPF cases
        let mods_joined = mods.join("");
        let mods_format = format!("+{mods_joined}");
        Display::fmt(&mods_format, f)
    }
}
