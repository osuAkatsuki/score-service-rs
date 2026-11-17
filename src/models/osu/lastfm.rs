use bitflags::bitflags;

bitflags! {
    /// The checks the osu! client runs against its own process, which it reports through
    /// its last.fm integration in place of the id of the beatmap it would scrobble.
    ///
    /// These are not the anticheat flags of score submission: the client reports these
    /// checks as an unshifted bitmask, so they start at the lowest bit.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct LastFmFlags: i32 {
        /// osu! was started with an `ld` argument
        const LdArgument = 1 << 0;
        /// The window in the foreground when osu! started contained a console window,
        /// possibly a debugger or a cheat loader
        const ConsoleWindow = 1 << 1;
        /// A thread of the osu! process owned a window before osu! created any of its own,
        /// which usually means something has been injected into osu!
        const ForeignWindow = 1 << 2;
        /// An assembly named like hq!osu's was loaded when osu! started
        const HqOsuAssembly = 1 << 3;
        /// A file named like hq!osu's is in the osu! directory
        const HqOsuFile = 1 << 4;
        /// The registry has Image File Execution Options for `osu!.exe`, traces of the
        /// hq!osu Relife multiaccounting tool. The user has used it, but may not be using it now.
        const ImageFileExecutionOptions = 1 << 5;
        /// An assembly named `sdl2` was loaded when a play started, a trace of an ancient AQN
        const Sdl2Assembly = 1 << 6;
        /// An assembly named `libeay32` was loaded when a play started, a trace of an ancient AQN
        const Libeay32Assembly = 1 << 7;
        /// osu! loaded the `notify1` sample outside the editor, which the menu of an ancient
        /// AQN does
        const Notify1Sample = 1 << 8;
        /// A loaded assembly's name contains `hqOsu`, `Harmony`, or `MonoMod` (case-insensitive)
        const PatchingAssemblyLoaded = 1 << 9;
        /// A top-level `ConsoleWindowClass` window is owned by the osu! process, which can be
        /// an injected console or debugging/patching tools
        const ProcessConsoleWindow = 1 << 10;
    }
}

/// Short explanations of the flags to store alongside them, prefixed with how likely
/// they are to mean cheating. The docs of the flags explain them in full.
const EXPLANATIONS: &[(LastFmFlags, &str)] = &[
    (
        LastFmFlags::LdArgument,
        "[UNKNOWN] Started with ld argument",
    ),
    (
        LastFmFlags::ConsoleWindow,
        "[LIKELY] Console window at startup",
    ),
    (
        LastFmFlags::ForeignWindow,
        "[LIKELY] Foreign window in process",
    ),
    (
        LastFmFlags::HqOsuAssembly,
        "[CERTAIN] HQOsu assembly loaded",
    ),
    (LastFmFlags::HqOsuFile, "[MIXED] HQOsu files present"),
    (
        LastFmFlags::ImageFileExecutionOptions,
        "[MIXED] HQOsu Relife registry traces",
    ),
    (
        LastFmFlags::Sdl2Assembly,
        "[CERTAIN] AQN sdl2 assembly loaded",
    ),
    (
        LastFmFlags::Libeay32Assembly,
        "[CERTAIN] AQN libeay32 assembly loaded",
    ),
    (
        LastFmFlags::Notify1Sample,
        "[CERTAIN] AQN notify1 sample loaded",
    ),
    (
        LastFmFlags::PatchingAssemblyLoaded,
        "[MIXED] Patching assembly loaded",
    ),
    (
        LastFmFlags::ProcessConsoleWindow,
        "[MIXED] Console window in process",
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
            None => format!("Undocumented flag {:#x}", self.bits()),
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
    fn reads_assembly_and_process_console_reports() {
        assert_eq!(
            LastFmFlags::from_report("a512"),
            Some(LastFmFlags::PatchingAssemblyLoaded)
        );
        assert_eq!(
            LastFmFlags::from_report("a1024"),
            Some(LastFmFlags::ProcessConsoleWindow)
        );
        assert_eq!(
            LastFmFlags::from_report("a1536"),
            Some(LastFmFlags::PatchingAssemblyLoaded | LastFmFlags::ProcessConsoleWindow)
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
        let flags = LastFmFlags::LdArgument | LastFmFlags::ProcessConsoleWindow;
        assert_eq!(
            flags.explain(),
            "[UNKNOWN] Started with ld argument\n\
             [MIXED] Console window in process"
        );
    }

    #[test]
    fn names_flags_that_have_no_explanation() {
        let flags = LastFmFlags::from_bits_retain(1 << 11 | 1 << 31);
        assert_eq!(
            flags.explain(),
            "Undocumented flag 0x800\n\
             Undocumented flag 0x80000000"
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
