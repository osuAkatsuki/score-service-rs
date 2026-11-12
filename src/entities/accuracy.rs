use crate::entities::mode::Gamemode;
use crate::entities::mods::Mods;
use crate::entities::scores::ScoreGrade;

#[derive(Debug, Copy, Clone)]
pub struct Accuracy {
    pub count_300: u16,
    pub count_100: u16,
    pub count_50: u16,
    pub count_geki: u16,
    pub count_katu: u16,
    pub count_miss: u16,
}

impl Accuracy {
    pub fn total_hits(&self, mode: Gamemode) -> u32 {
        match mode {
            Gamemode::Standard => {
                (self.count_300 as u32)
                    + (self.count_100 as u32)
                    + (self.count_50 as u32)
                    + (self.count_miss as u32)
            }
            Gamemode::Taiko => {
                (self.count_300 as u32) + (self.count_100 as u32) + (self.count_miss as u32)
            }
            Gamemode::Catch => {
                (self.count_300 as u32)
                    + (self.count_100 as u32)
                    + (self.count_50 as u32)
                    + (self.count_katu as u32)
                    + (self.count_miss as u32)
            }
            Gamemode::Mania => {
                (self.count_300 as u32)
                    + (self.count_100 as u32)
                    + (self.count_50 as u32)
                    + (self.count_geki as u32)
                    + (self.count_katu as u32)
                    + (self.count_miss as u32)
            }
        }
    }

    /// The amount of objects that were actually hit, which is what the total hits of a user
    /// are made up of. Unlike [`Accuracy::total_hits`] this does not include misses.
    pub fn successful_hits(&self, mode: Gamemode) -> u32 {
        let hits = (self.count_300 as u32) + (self.count_100 as u32);
        match mode {
            Gamemode::Standard => hits + (self.count_50 as u32),
            // Droplets and their misses are tracked as 50s and katus
            Gamemode::Catch => hits,
            Gamemode::Taiko | Gamemode::Mania => {
                hits + (self.count_50 as u32) + (self.count_geki as u32) + (self.count_katu as u32)
            }
        }
    }

    pub fn as_percent(&self, mode: Gamemode) -> f32 {
        let total_hits = self.total_hits(mode);
        if total_hits == 0 {
            return 0.0;
        }

        match mode {
            Gamemode::Standard => {
                let accuracy_sum = (self.count_300 as u32 * 300)
                    + (self.count_100 as u32 * 100)
                    + (self.count_50 as u32 * 50);
                100.0 * accuracy_sum as f32 / (total_hits * 300) as f32
            }
            Gamemode::Taiko => {
                let accuracy_sum = (self.count_300 as u32 * 2) + (self.count_100 as u32);
                100.0 * accuracy_sum as f32 / (total_hits * 2) as f32
            }
            Gamemode::Catch => {
                let accuracy_sum =
                    (self.count_300 as u32) + (self.count_100 as u32) + (self.count_50 as u32);
                100.0 * accuracy_sum as f32 / total_hits as f32
            }
            Gamemode::Mania => {
                let accuracy_sum = (self.count_300 as u32 + self.count_geki as u32) * 300
                    + (self.count_katu as u32) * 200
                    + (self.count_100 as u32) * 100
                    + (self.count_50 as u32) * 50;
                100.0 * accuracy_sum as f32 / (total_hits * 300) as f32
            }
        }
    }

    fn std_taiko_grade(&self, use_silver_grades: bool) -> ScoreGrade {
        let total_hits = self.total_hits(Gamemode::Standard);
        if total_hits == 0 {
            return ScoreGrade::D;
        }

        if self.count_300 as u32 == total_hits {
            if use_silver_grades {
                ScoreGrade::XH
            } else {
                ScoreGrade::X
            }
        } else {
            let total_hits = total_hits as f32;
            let ratio_300 = self.count_300 as f32 / total_hits;
            let ratio_50 = self.count_50 as f32 / total_hits;

            if ratio_300 > 0.9 && ratio_50 <= 0.01 && self.count_miss == 0 {
                if use_silver_grades {
                    ScoreGrade::SH
                } else {
                    ScoreGrade::S
                }
            } else if ratio_300 > 0.9 || (ratio_300 > 0.8 && self.count_miss == 0) {
                ScoreGrade::A
            } else if ratio_300 > 0.8 || (ratio_300 > 0.7 && self.count_miss == 0) {
                ScoreGrade::B
            } else if ratio_300 > 0.6 {
                ScoreGrade::C
            } else {
                ScoreGrade::D
            }
        }
    }

    pub fn grade(&self, mode: Gamemode, mods: Mods) -> ScoreGrade {
        let use_silver_grades = mods.has_any(Mods::Hidden | Mods::Flashlight | Mods::FadeIn);
        match mode {
            Gamemode::Standard | Gamemode::Taiko => self.std_taiko_grade(use_silver_grades),
            Gamemode::Catch => {
                let accuracy = self.as_percent(mode);
                if accuracy == 100.0 {
                    if use_silver_grades {
                        ScoreGrade::XH
                    } else {
                        ScoreGrade::X
                    }
                } else if accuracy > 98.0 {
                    if use_silver_grades {
                        ScoreGrade::SH
                    } else {
                        ScoreGrade::S
                    }
                } else if accuracy > 94.0 {
                    ScoreGrade::A
                } else if accuracy > 90.0 {
                    ScoreGrade::B
                } else if accuracy > 85.0 {
                    ScoreGrade::C
                } else {
                    ScoreGrade::D
                }
            }
            Gamemode::Mania => {
                let accuracy = self.as_percent(mode);
                if accuracy == 100.0 {
                    if use_silver_grades {
                        ScoreGrade::XH
                    } else {
                        ScoreGrade::X
                    }
                } else if accuracy > 95.0 {
                    if use_silver_grades {
                        ScoreGrade::SH
                    } else {
                        ScoreGrade::S
                    }
                } else if accuracy > 90.0 {
                    ScoreGrade::A
                } else if accuracy > 80.0 {
                    ScoreGrade::B
                } else if accuracy > 70.0 {
                    ScoreGrade::C
                } else {
                    ScoreGrade::D
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn accuracy(
        count_300: u16,
        count_100: u16,
        count_50: u16,
        count_geki: u16,
        count_katu: u16,
        count_miss: u16,
    ) -> Accuracy {
        Accuracy {
            count_300,
            count_100,
            count_50,
            count_geki,
            count_katu,
            count_miss,
        }
    }

    #[test]
    fn calculates_the_accuracy_of_every_mode() {
        let accuracy = accuracy(80, 10, 6, 20, 5, 4);
        let expected = [
            (Gamemode::Standard, 84.3333),
            (Gamemode::Taiko, 90.4255),
            (Gamemode::Catch, 91.4286),
            (Gamemode::Mania, 86.1333),
        ];
        for (mode, expected_percent) in expected {
            let percent = accuracy.as_percent(mode);
            assert!(
                (percent - expected_percent).abs() < 0.001,
                "{mode:?}: {percent} != {expected_percent}"
            );
        }
    }

    #[test]
    fn plays_without_any_hits_have_no_accuracy() {
        let accuracy = accuracy(0, 0, 0, 0, 0, 0);
        assert_eq!(accuracy.as_percent(Gamemode::Standard), 0.0);
        assert_eq!(accuracy.grade(Gamemode::Standard, Mods::None), ScoreGrade::D);
    }

    #[test]
    fn misses_are_not_hits() {
        let accuracy = accuracy(80, 10, 6, 20, 5, 4);
        assert_eq!(accuracy.successful_hits(Gamemode::Standard), 96);
        assert_eq!(accuracy.successful_hits(Gamemode::Taiko), 121);
        assert_eq!(accuracy.successful_hits(Gamemode::Catch), 90);
        assert_eq!(accuracy.successful_hits(Gamemode::Mania), 121);
    }

    #[test]
    fn grades_standard_plays_by_their_hit_ratios() {
        let perfect = accuracy(100, 0, 0, 0, 0, 0);
        assert_eq!(perfect.grade(Gamemode::Standard, Mods::None), ScoreGrade::X);
        assert_eq!(perfect.grade(Gamemode::Standard, Mods::Hidden), ScoreGrade::XH);

        let no_miss = accuracy(95, 5, 0, 0, 0, 0);
        assert_eq!(no_miss.grade(Gamemode::Standard, Mods::None), ScoreGrade::S);
        assert_eq!(no_miss.grade(Gamemode::Standard, Mods::Flashlight), ScoreGrade::SH);

        let one_miss = accuracy(95, 4, 0, 0, 0, 1);
        assert_eq!(one_miss.grade(Gamemode::Standard, Mods::None), ScoreGrade::A);

        let sloppy = accuracy(65, 20, 5, 0, 0, 10);
        assert_eq!(sloppy.grade(Gamemode::Standard, Mods::None), ScoreGrade::C);
    }

    #[test]
    fn grades_mania_and_catch_plays_by_their_accuracy() {
        let mania = accuracy(900, 20, 0, 60, 20, 0);
        assert_eq!(mania.grade(Gamemode::Mania, Mods::None), ScoreGrade::S);
        assert_eq!(mania.grade(Gamemode::Mania, Mods::FadeIn), ScoreGrade::SH);

        let catch = accuracy(300, 40, 50, 0, 3, 7);
        assert_eq!(catch.grade(Gamemode::Catch, Mods::None), ScoreGrade::A);
    }
}
