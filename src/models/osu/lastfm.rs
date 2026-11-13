use bitflags::bitflags;

bitflags! {
    /// The flags the anticheat of the osu! client reports through its last.fm integration
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct LastFmFlags: i32 {
        /// The song speed does not match the time that has elapsed
        const Timewarp = 1 << 1;
        const IncorrectModValue = 1 << 2;
        const MultipleOsuClients = 1 << 3;
        const ChecksumFail = 1 << 4;
        const FlashlightChecksumFail = 1 << 5;

        // These two are only ever raised by the server
        const OsuExeChecksumFail = 1 << 6;
        const MissingProcess = 1 << 7;

        const FlashlightRemover = 1 << 8;
        const AutospinHack = 1 << 9;
        const WindowOverlay = 1 << 10;
        const FastPress = 1 << 11;

        // Something alters the input between the raw input and what the game receives
        const MouseDiscrepancy = 1 << 12;
        const KeyboardDiscrepancy = 1 << 13;

        const LfFlagPresent = 1 << 14;
        const OsuDebugged = 1 << 15;
        const ExtraThreads = 1 << 16;

        const HqOsuAssembly = 1 << 17;
        const HqOsuFile = 1 << 18;
        const HqOsuRelife = 1 << 19;

        const AqnSql2Lib = 1 << 20;
        const AqnLibeay32 = 1 << 21;
        const AqnMenuSound = 1 << 22;
    }
}

/// Down to earth explanations of the flags, along with how likely they are to mean cheating
const EXPLANATIONS: &[(LastFmFlags, &str)] = &[
    (
        LastFmFlags::Timewarp,
        "[LIKELY] Timewarp flag triggered (audio is desynced from expected position)! \
         May be caused by lag on the user's end.",
    ),
    (
        LastFmFlags::IncorrectModValue,
        "[MIXED] The score's mod value didn't match enabled mods \
         (possible sign of a mod remover such as Hidden remover).",
    ),
    (
        LastFmFlags::MultipleOsuClients,
        "[MIXED] The user had multiple instances of osu! open.",
    ),
    (
        LastFmFlags::ChecksumFail,
        "[LIKELY] The score related memory has been edited in a weird manner.",
    ),
    (
        LastFmFlags::FlashlightChecksumFail,
        "[UNKNOWN] FL Checksum fail occurence is unknown.",
    ),
    (
        LastFmFlags::FlashlightRemover,
        "[CERTAIN] User is using a flashlight remover.",
    ),
    (
        LastFmFlags::WindowOverlay,
        "[LIKELY] A transparent window is overlaying the osu! client.",
    ),
    (
        LastFmFlags::FastPress,
        "[LIKELY] User is consistently hitting notes with a low latency in mania.",
    ),
    (
        LastFmFlags::MouseDiscrepancy,
        "[LIKELY] Something is altering the mouse position received by the game.",
    ),
    (
        LastFmFlags::KeyboardDiscrepancy,
        "[LIKELY] Something is altering the keyboard presses received by the game.",
    ),
    (
        LastFmFlags::LfFlagPresent,
        "[UNKNOWN] LF flag is present. Occurence of this is unknown.",
    ),
    (
        LastFmFlags::OsuDebugged,
        "[LIKELY] osu! is being debugged. Console attached to the process has been detected.",
    ),
    (
        LastFmFlags::ExtraThreads,
        "[LIKELY] A foreign thread has been detected attached to osu! \
         This is a method usually used by cheats to run.",
    ),
    (
        LastFmFlags::HqOsuAssembly,
        "[CERTAIN] The HQOsu assembly has been detected.",
    ),
    (
        LastFmFlags::HqOsuFile,
        "[MIXED] The presence of HQOsu files has been detected.",
    ),
    (
        LastFmFlags::HqOsuRelife,
        "[MIXED] HQOsu Relife traces found in registry. This means that the user has used \
         the multiaccounting tool in the past, but may not be using it now.",
    ),
    (
        LastFmFlags::AqnSql2Lib,
        "[CERTAIN] Ancient AQN library SQL2Lib detected.",
    ),
    (
        LastFmFlags::AqnLibeay32,
        "[CERTAIN] Use of ancient AQN version detected through library libeay32.dll",
    ),
    (
        LastFmFlags::AqnMenuSound,
        "[CERTAIN] Use of ancient AQN version detected through menu sound.",
    ),
];

impl LastFmFlags {
    /// The client reports its flags in place of the id of the beatmap it would scrobble,
    /// which it tells apart by prefixing them with an `a`.
    pub fn from_report(beatmap_id_or_flags: &str) -> Option<Self> {
        let flags = beatmap_id_or_flags.strip_prefix('a')?;
        let flags = Self::from_bits_retain(flags.parse().ok()?);
        (!flags.is_empty()).then_some(flags)
    }

    /// Explains every flag that has been raised on a line of its own
    pub fn explain(self) -> String {
        let explanations: Vec<String> = (0..i32::BITS)
            .map(|bit| Self::from_bits_retain(1 << bit))
            .filter(|flag| self.contains(*flag))
            .map(Self::explain_one)
            .collect();
        explanations.join("\n")
    }

    fn explain_one(self) -> String {
        match EXPLANATIONS.iter().find(|(flag, _)| *flag == self) {
            Some((_, explanation)) => explanation.to_string(),
            None => format!("Undocumented Flag: {self:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_flags_from_reports_of_the_client() {
        assert_eq!(
            LastFmFlags::from_report("a260"),
            Some(LastFmFlags::IncorrectModValue | LastFmFlags::FlashlightRemover)
        );
    }

    #[test]
    fn does_not_mistake_scrobbled_beatmaps_for_reports() {
        assert_eq!(LastFmFlags::from_report("75"), None);
        assert_eq!(LastFmFlags::from_report(""), None);
    }

    #[test]
    fn ignores_reports_without_flags() {
        assert_eq!(LastFmFlags::from_report("a0"), None);
        assert_eq!(LastFmFlags::from_report("a"), None);
        assert_eq!(LastFmFlags::from_report("abc"), None);
    }

    #[test]
    fn explains_every_raised_flag_including_the_highest() {
        let flags = LastFmFlags::MultipleOsuClients | LastFmFlags::AqnMenuSound;
        assert_eq!(
            flags.explain(),
            "[MIXED] The user had multiple instances of osu! open.\n\
             [CERTAIN] Use of ancient AQN version detected through menu sound."
        );
    }

    #[test]
    fn names_flags_that_have_no_explanation() {
        let flags = LastFmFlags::AutospinHack | LastFmFlags::from_bits_retain(1 << 30);
        assert_eq!(
            flags.explain(),
            "Undocumented Flag: LastFmFlags(AutospinHack)\n\
             Undocumented Flag: LastFmFlags(0x40000000)"
        );
    }
}
