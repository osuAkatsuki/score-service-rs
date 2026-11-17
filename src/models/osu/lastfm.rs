use bitflags::bitflags;

bitflags! {
    /// The checks the osu! client runs against its own process, which it reports through
    /// its last.fm integration in place of the id of the beatmap it would scrobble.
    ///
    /// These are not the anticheat flags of score submission: the client sends the value
    /// of these checks as is (`OsuMain.startupValue`), so they start at the lowest bit.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct LastFmFlags: i32 {
        /// osu! was started with an `ld` argument
        const LdArgument = 1 << 0;
        /// The window in the foreground when osu! started contained a console window
        const ConsoleWindow = 1 << 1;
        /// A thread of the osu! process owned a window before osu! created any of its own
        const ForeignWindow = 1 << 2;
        /// An assembly named like hq!osu's was loaded when osu! started
        const HqOsuAssembly = 1 << 3;
        /// A file named like hq!osu's is in the osu! directory
        const HqOsuFile = 1 << 4;
        /// The registry has Image File Execution Options for `osu!.exe`
        const ImageFileExecutionOptions = 1 << 5;
        /// An assembly named `sdl2` was loaded when a play started
        const Sdl2Assembly = 1 << 6;
        /// An assembly named `libeay32` was loaded when a play started
        const Libeay32Assembly = 1 << 7;
        /// osu! loaded the `notify1` sample outside the editor
        const Notify1Sample = 1 << 8;
    }
}

/// Down to earth explanations of the flags, along with how likely they are to mean cheating
const EXPLANATIONS: &[(LastFmFlags, &str)] = &[
    (
        LastFmFlags::LdArgument,
        "[UNKNOWN] osu! was started with an \"ld\" argument. What passes it is unknown.",
    ),
    (
        LastFmFlags::ConsoleWindow,
        "[LIKELY] A console window was open inside the foreground window when osu! started \
         (possibly a debugger or a cheat loader).",
    ),
    (
        LastFmFlags::ForeignWindow,
        "[LIKELY] A thread of the osu! process already owned a window before osu! created its own. \
         This usually means something has been injected into osu!",
    ),
    (
        LastFmFlags::HqOsuAssembly,
        "[CERTAIN] The HQOsu assembly has been loaded into osu!",
    ),
    (
        LastFmFlags::HqOsuFile,
        "[MIXED] HQOsu files are present in the osu! directory.",
    ),
    (
        LastFmFlags::ImageFileExecutionOptions,
        "[MIXED] The registry has Image File Execution Options for osu!.exe. These are traces of \
         the HQOsu Relife multiaccounting tool, meaning the user has used it in the past, \
         but may not be using it now.",
    ),
    (
        LastFmFlags::Sdl2Assembly,
        "[CERTAIN] An sdl2 assembly was loaded into osu! during play, a trace of an ancient AQN version.",
    ),
    (
        LastFmFlags::Libeay32Assembly,
        "[CERTAIN] A libeay32 assembly was loaded into osu! during play, \
         a trace of an ancient AQN version.",
    ),
    (
        LastFmFlags::Notify1Sample,
        "[CERTAIN] osu! loaded the notify1 sample outside the editor, \
         which the menu of an ancient AQN version does.",
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
            Some(LastFmFlags::ForeignWindow | LastFmFlags::Notify1Sample)
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
    fn explains_every_raised_flag_including_the_lowest_and_highest() {
        let flags = LastFmFlags::LdArgument | LastFmFlags::Notify1Sample;
        assert_eq!(
            flags.explain(),
            "[UNKNOWN] osu! was started with an \"ld\" argument. What passes it is unknown.\n\
             [CERTAIN] osu! loaded the notify1 sample outside the editor, \
             which the menu of an ancient AQN version does."
        );
    }

    #[test]
    fn names_flags_that_have_no_explanation() {
        let flags = LastFmFlags::from_bits_retain(1 << 9 | 1 << 30);
        assert_eq!(
            flags.explain(),
            "Undocumented Flag: LastFmFlags(0x200)\n\
             Undocumented Flag: LastFmFlags(0x40000000)"
        );
    }

    #[test]
    fn explains_every_flag_the_client_reports() {
        for flag in LastFmFlags::all().iter() {
            assert!(
                !flag.explain().starts_with("Undocumented"),
                "{flag:?} has no explanation"
            );
        }
    }
}
