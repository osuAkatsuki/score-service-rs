mod catch;
mod hush_hush;
mod mania;
mod mod_introductions;
mod osu;
mod taiko;

use crate::entities::achievements::Achievement as Entity;
use crate::entities::mode::Gamemode;
use crate::entities::scores::Score;
use crate::entities::stats::Stats;
use crate::models::achievements::Achievement as Model;

pub type ConditionFn = fn(&Score, &Stats, f32, Option<usize>) -> bool;

pub struct Achievement {
    pub id: i32,
    pub icon_file: String,
    pub name: String,
    pub description: String,
    pub hidden: bool,
    pub mode: Option<Gamemode>,
    pub condition: ConditionFn,
}

impl Achievement {
    pub fn map_from(entity: Entity) -> Self {
        Self {
            condition: condition_fn_from(&entity.file),
            id: entity.id,
            icon_file: entity.file,
            name: entity.name,
            description: entity.desc,
            hidden: entity.hidden,
            mode: entity.mode.map(Gamemode::from),
        }
    }

    pub fn as_model(&self) -> Model {
        Model {
            id: self.id,
            icon_file: self.icon_file.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            hidden: self.hidden,
            mode: self.mode,
        }
    }
}

pub fn condition_fn_from(key: &str) -> ConditionFn {
    match key {
        // Catch
        "fruits-skill-pass-1" => catch::catch_skill_pass_1,
        "fruits-skill-pass-2" => catch::catch_skill_pass_2,
        "fruits-skill-pass-3" => catch::catch_skill_pass_3,
        "fruits-skill-pass-4" => catch::catch_skill_pass_4,
        "fruits-skill-pass-5" => catch::catch_skill_pass_5,
        "fruits-skill-pass-6" => catch::catch_skill_pass_6,
        "fruits-skill-pass-7" => catch::catch_skill_pass_7,
        "fruits-skill-pass-8" => catch::catch_skill_pass_8,

        "fruits-skill-fc-1" => catch::catch_skill_fc_1,
        "fruits-skill-fc-2" => catch::catch_skill_fc_2,
        "fruits-skill-fc-3" => catch::catch_skill_fc_3,
        "fruits-skill-fc-4" => catch::catch_skill_fc_4,
        "fruits-skill-fc-5" => catch::catch_skill_fc_5,
        "fruits-skill-fc-6" => catch::catch_skill_fc_6,
        "fruits-skill-fc-7" => catch::catch_skill_fc_7,
        "fruits-skill-fc-8" => catch::catch_skill_fc_8,

        "fruits-hits-20000" => catch::fruits_hits_20000,
        "fruits-hits-200000" => catch::fruits_hits_200000,
        "fruits-hits-2000000" => catch::fruits_hits_2000000,
        "fruits-hits-20000000" => catch::fruits_hits_20000000,

        "fruits-rank-50000" => catch::catch_rank_50k,
        "fruits-rank-10000" => catch::catch_rank_10k,
        "fruits-rank-5000" => catch::catch_rank_5k,
        "fruits-rank-1000" => catch::catch_rank_1k,

        // Hush Hush
        "all-secret-bunny" => hush_hush::dont_let_bunny_distract,
        "all-secret-sranker" => hush_hush::s_ranker,
        "all-secret-improved" => hush_hush::most_improved,
        "osu-secret-dancer" => hush_hush::nonstop_dancer,
        "all-secret-consolation" => hush_hush::consolation_prize,
        "all-secret-challenge" => hush_hush::challenge_accepted,
        "all-secret-stumbler" => hush_hush::stumbler,
        "all-secret-jackpot" => hush_hush::jackpot,
        "all-secret-quickdraw" => hush_hush::quick_draw,
        "all-secret-obsessed" => hush_hush::obsessed,
        "all-secret-nonstop" => hush_hush::nonstop,
        "all-secret-jackofalltrades" => hush_hush::jack_of_all_trades,
        "mania-secret-twin" => hush_hush::twin_perspectives,

        // Mania
        "mania-skill-pass-1" => mania::mania_skill_pass_1,
        "mania-skill-pass-2" => mania::mania_skill_pass_2,
        "mania-skill-pass-3" => mania::mania_skill_pass_3,
        "mania-skill-pass-4" => mania::mania_skill_pass_4,
        "mania-skill-pass-5" => mania::mania_skill_pass_5,
        "mania-skill-pass-6" => mania::mania_skill_pass_6,
        "mania-skill-pass-7" => mania::mania_skill_pass_7,
        "mania-skill-pass-8" => mania::mania_skill_pass_8,

        "mania-skill-fc-1" => mania::mania_skill_fc_1,
        "mania-skill-fc-2" => mania::mania_skill_fc_2,
        "mania-skill-fc-3" => mania::mania_skill_fc_3,
        "mania-skill-fc-4" => mania::mania_skill_fc_4,
        "mania-skill-fc-5" => mania::mania_skill_fc_5,
        "mania-skill-fc-6" => mania::mania_skill_fc_6,
        "mania-skill-fc-7" => mania::mania_skill_fc_7,
        "mania-skill-fc-8" => mania::mania_skill_fc_8,

        "mania-hits-40000" => mania::mania_hits_40000,
        "mania-hits-400000" => mania::mania_hits_400000,
        "mania-hits-4000000" => mania::mania_hits_4000000,
        "mania-hits-40000000" => mania::mania_hits_40000000,

        "mania-rank-50000" => mania::mania_rank_50k,
        "mania-rank-10000" => mania::mania_rank_10k,
        "mania-rank-5000" => mania::mania_rank_5k,
        "mania-rank-1000" => mania::mania_rank_1k,

        // Mod introductions
        "all-intro-suddendeath" => mod_introductions::intro_sudden_death,
        "all-intro-perfect" => mod_introductions::intro_perfect,
        "all-intro-hardrock" => mod_introductions::intro_hardrock,
        "all-intro-doubletime" => mod_introductions::intro_doubletime,
        "all-intro-nightcore" => mod_introductions::intro_nightcore,
        "all-intro-hidden" => mod_introductions::intro_hidden,
        "all-intro-flashlight" => mod_introductions::intro_flashlight,
        "all-intro-easy" => mod_introductions::intro_easy,
        "all-intro-nofail" => mod_introductions::intro_nofail,
        "all-intro-halftime" => mod_introductions::intro_halftime,
        "all-intro-spunout" => mod_introductions::intro_spunout,

        // osu!
        "osu-skill-pass-1" => osu::osu_skill_pass_1,
        "osu-skill-pass-2" => osu::osu_skill_pass_2,
        "osu-skill-pass-3" => osu::osu_skill_pass_3,
        "osu-skill-pass-4" => osu::osu_skill_pass_4,
        "osu-skill-pass-5" => osu::osu_skill_pass_5,
        "osu-skill-pass-6" => osu::osu_skill_pass_6,
        "osu-skill-pass-7" => osu::osu_skill_pass_7,
        "osu-skill-pass-8" => osu::osu_skill_pass_8,
        "osu-skill-pass-9" => osu::osu_skill_pass_9,
        "osu-skill-pass-10" => osu::osu_skill_pass_10,

        "osu-skill-fc-1" => osu::osu_skill_fc_1,
        "osu-skill-fc-2" => osu::osu_skill_fc_2,
        "osu-skill-fc-3" => osu::osu_skill_fc_3,
        "osu-skill-fc-4" => osu::osu_skill_fc_4,
        "osu-skill-fc-5" => osu::osu_skill_fc_5,
        "osu-skill-fc-6" => osu::osu_skill_fc_6,
        "osu-skill-fc-7" => osu::osu_skill_fc_7,
        "osu-skill-fc-8" => osu::osu_skill_fc_8,
        "osu-skill-fc-9" => osu::osu_skill_fc_9,
        "osu-skill-fc-10" => osu::osu_skill_fc_10,

        "osu-combo-500" => osu::osu_combo_500,
        "osu-combo-750" => osu::osu_combo_750,
        "osu-combo-1000" => osu::osu_combo_1000,
        "osu-combo-2000" => osu::osu_combo_2000,

        "osu-plays-5000" => osu::osu_plays_5000,
        "osu-plays-15000" => osu::osu_plays_15000,
        "osu-plays-25000" => osu::osu_plays_25000,
        "osu-plays-50000" => osu::osu_plays_50000,

        "osu-rank-50000" => osu::osu_rank_50k,
        "osu-rank-10000" => osu::osu_rank_10k,
        "osu-rank-5000" => osu::osu_rank_5k,
        "osu-rank-1000" => osu::osu_rank_1k,

        // Taiko
        "taiko-skill-pass-1" => taiko::taiko_skill_pass_1,
        "taiko-skill-pass-2" => taiko::taiko_skill_pass_2,
        "taiko-skill-pass-3" => taiko::taiko_skill_pass_3,
        "taiko-skill-pass-4" => taiko::taiko_skill_pass_4,
        "taiko-skill-pass-5" => taiko::taiko_skill_pass_5,
        "taiko-skill-pass-6" => taiko::taiko_skill_pass_6,
        "taiko-skill-pass-7" => taiko::taiko_skill_pass_7,
        "taiko-skill-pass-8" => taiko::taiko_skill_pass_8,

        "taiko-skill-fc-1" => taiko::taiko_skill_fc_1,
        "taiko-skill-fc-2" => taiko::taiko_skill_fc_2,
        "taiko-skill-fc-3" => taiko::taiko_skill_fc_3,
        "taiko-skill-fc-4" => taiko::taiko_skill_fc_4,
        "taiko-skill-fc-5" => taiko::taiko_skill_fc_5,
        "taiko-skill-fc-6" => taiko::taiko_skill_fc_6,
        "taiko-skill-fc-7" => taiko::taiko_skill_fc_7,
        "taiko-skill-fc-8" => taiko::taiko_skill_fc_8,

        "taiko-hits-30000" => taiko::taiko_hits_30000,
        "taiko-hits-300000" => taiko::taiko_hits_300000,
        "taiko-hits-3000000" => taiko::taiko_hits_3000000,
        "taiko-hits-30000000" => taiko::taiko_hits_30000000,

        "taiko-rank-50000" => taiko::taiko_rank_50k,
        "taiko-rank-10000" => taiko::taiko_rank_10k,
        "taiko-rank-5000" => taiko::taiko_rank_5k,
        "taiko-rank-1000" => taiko::taiko_rank_1k,

        _ => |_, _, _, _| false,
    }
}
