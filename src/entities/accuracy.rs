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
