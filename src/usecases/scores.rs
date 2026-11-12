use crate::adapters::performance_service::PerformanceRequest;
use crate::adapters::{bancho_service, performance_service};
use crate::api::context::RequestContext;
use crate::common::error::{AppError, ServiceResult};
use crate::entities::accuracy::Accuracy;
use crate::entities::match_game_scores::CreateMatchGameScoreArgs;
use crate::entities::mode::{Gamemode, GamemodeMod, ModdedGamemode};
use crate::entities::mods::Mods;
use crate::entities::scores::{LeaderboardFilter, Score, ScoreRankingMetric, ScoreStatus, SubmissionEvent};
use crate::entities::stats::{Stats, UpdateStats};
use crate::models::achievements::Achievement;
use crate::models::osu::score_submission::{ScoreFlags, ScoreSubmissionResult, SubmitScoreArgs};
use crate::repositories::{match_game_scores, scores};
use crate::usecases::{beatmaps, charts, first_places, performance_caps, replays, stats, users};
use tracing::{info, warn};
use crate::models::beatmaps::RankedStatus;
use crate::models::osu::leaderboards::{FetchLeaderboardArgs, LeaderboardType, UserLeaderboardScoresView, UserLeaderboardView};
use crate::settings::AppSettings;

const CURRENT_LBS_VERSION: i32 = 4;
pub async fn fetch_leaderboard_view(
    ctx: &RequestContext,
    args: FetchLeaderboardArgs,
) -> ServiceResult<UserLeaderboardView> {
    let user = users::fetch_one_and_auth(ctx, &args.username, &args.password_md5).await?;
    if args.leaderboard_version != CURRENT_LBS_VERSION {
        let task_ctx = ctx.clone();
        ctx.spawn(async move {
            _ = users::restrict_user(&task_ctx, user.user_id, "osu! stable leaderboard version mismatch.").await;
        });
    }

    let beatmap = match beatmaps::fetch_by_md5(&args.beatmap_md5).await {
        Ok(beatmap) => beatmap,
        Err(AppError::BeatmapsNotFound) => return match beatmaps::file_name_exists(ctx, &args.file_name).await? {
            true => Ok(UserLeaderboardView::Status(RankedStatus::UpdateAvailable)),
            false => Ok(UserLeaderboardView::Status(RankedStatus::NotSubmitted)),
        },
        Err(e) => return Err(e),
    };

    if !beatmap.has_leaderboards() || args.leaderboard_type == LeaderboardType::Local {
        return Ok(UserLeaderboardView::Status(beatmap.status));
    }

    // The editor is only interested in the beatmap itself
    if args.requesting_from_editor_song_select {
        return Ok(UserLeaderboardView::Beatmap(beatmap));
    }

    // Remove any invalid mods for this game mode (e.g. Relax on Mania)
    let sanitized_mods = args.mods.sanitize_for_mode(args.mode);
    let modded_gamemode = ModdedGamemode::from_mode_and_mods(args.mode, sanitized_mods);
    let display_pp_as_score = modded_gamemode.mode_mod() != GamemodeMod::Vanilla || user.vanilla_pp_leaderboards;

    let filter = match args.leaderboard_type {
        LeaderboardType::Global | LeaderboardType::Local => LeaderboardFilter::None,
        LeaderboardType::Mods => LeaderboardFilter::Mods(sanitized_mods),
        LeaderboardType::Friends => {
            let friends = users::fetch_user_friends(ctx, user.user_id).await?;
            LeaderboardFilter::Friends(friends)
        },
        LeaderboardType::Country => LeaderboardFilter::Country(&user.country),
    };

    let leaderboard_size = user.privileges.is_donor()
        .then(|| user.leaderboard_size)
        .flatten()
        .unwrap_or_else(|| AppSettings::get().leaderboard_size);
    let leaderboard_scores = scores::fetch_leaderboard_view(
        ctx,
        user.user_id,
        &beatmap.beatmap_md5,
        modded_gamemode,
        display_pp_as_score,
        &filter,
        leaderboard_size,
    ).await?;

    // Fewer scores than requested means there is nothing left that has not been fetched already
    let is_complete = leaderboard_scores.len() < leaderboard_size as usize;
    let personal_best = leaderboard_scores.iter()
        .find(|score| score.user_id == user.user_id)
        .cloned();

    let personal_best = match personal_best {
        None if !is_complete => scores::fetch_leaderboard_personal_best(
            ctx,
            user.user_id,
            &beatmap.beatmap_md5,
            modded_gamemode,
            display_pp_as_score,
            &filter,
        ).await?,
        personal_best => personal_best,
    };

    let total_scores = match is_complete {
        true => leaderboard_scores.len() as _,
        false => scores::fetch_leaderboard_scores_count(
            ctx,
            user.user_id,
            &beatmap.beatmap_md5,
            modded_gamemode,
            display_pp_as_score,
            &filter,
        ).await?,
    };

    Ok(UserLeaderboardView::ScoresView(UserLeaderboardScoresView {
        beatmap,
        total_scores,
        personal_best,
        display_pp_as_score,
        user_id: user.user_id,
        scores: leaderboard_scores,
    }))
}

pub async fn submit(
    ctx: &RequestContext,
    args: SubmitScoreArgs,
) -> ServiceResult<ScoreSubmissionResult> {
    let user = users::fetch_one_and_auth(ctx, &args.score.username, &args.password_md5).await?;
    if args.anticheat_token.is_none() {
        warn!(user_id = user.user_id, "Anticheat Token is missing");
        return Err(AppError::ScoresAnticheatTokenMissing);
    }

    if ctx.user_agent.as_deref() != Some("osu!") {
        let reason = format!(
            "Submitted score with unexpected user agent: {}.",
            ctx.user_agent.as_deref().unwrap_or("none")
        );
        users::restrict_user(ctx, user.user_id, &reason).await?;
        return Err(AppError::ScoresIllegalScore);
    }

    // Honestly no idea where the 24 comes from
    // Just using it to match https://github.com/osuAkatsuki/score-service/blob/master/app/api/score_sub.py#L348
    if !args.quit
        && args.score.stats.passed
        && args
            .replay_data
            .as_ref()
            .is_none_or(|replay| replay.len() < 24)
    {
        users::restrict_user(ctx, user.user_id, "Submitted score without replay.").await?;
        return Err(AppError::ScoresMissingReplay);
    }

    let uninstall_md5 = md5::compute(&args.unique_ids.uninstall_md5);
    let uninstall_id_hash = format!("{:x}", uninstall_md5);
    if uninstall_id_hash != args.client_info.uninstall_md5 {
        users::restrict_user(
            ctx,
            user.user_id,
            "Submitted score with uninstall id hash mismatch.",
        )
        .await?;
        return Err(AppError::ScoresIllegalScore);
    }

    let disk_signature_md5 = md5::compute(&args.unique_ids.disk_signature_md5);
    let disk_signature_hash = format!("{:x}", disk_signature_md5);
    if disk_signature_hash != args.client_info.disk_signature_md5 {
        users::restrict_user(
            ctx,
            user.user_id,
            "Submitted score with disk signature mismatch.",
        )
        .await?;
        return Err(AppError::ScoresIllegalScore);
    }

    if args.beatmap_md5 != args.score.beatmap_md5 {
        users::restrict_user(
            ctx,
            user.user_id,
            "Submitted score with beatmap hash mismatch (edited .osu file).",
        )
        .await?;
        return Err(AppError::ScoresIllegalScore);
    }

    if args.score_md5() != args.score.score_md5 {
        users::restrict_user(
            ctx,
            user.user_id,
            "Submitted score with score hash mismatch.",
        )
        .await?;
        return Err(AppError::ScoresIllegalScore);
    }

    if args.score.stats.mods.is_illegal_combination() {
        let reason = format!(
            "Submitted score with illegal mod combination: {}.",
            args.score.stats.mods
        );
        users::restrict_user(ctx, user.user_id, &reason).await?;
        return Err(AppError::ScoresIllegalScore);
    }

    // Very old osu! anticheat flags
    if args.score.stats.score_flags != ScoreFlags::Clean {
        let flags: Vec<_> = args
            .score
            .stats
            .score_flags
            .iter_names()
            .map(|(name, _)| name)
            .collect();
        warn!(user_id = user.user_id, "Found flags {flags:?} on score.");
    }

    let task_ctx = ctx.clone();
    ctx.spawn(async move {
        _ = users::handle_pending_username_change(&task_ctx, user.user_id).await;
    });
    let beatmap = beatmaps::fetch_by_md5(&args.beatmap_md5).await?;
    if !beatmap.are_mods_rankable(args.score.stats.mods) || !beatmap.has_leaderboards() {
        return Err(AppError::ScoresNotRanked);
    }

    // Remove any invalid mods for this game mode (e.g. Relax on Mania)
    let sanitized_mods = args
        .score
        .stats
        .mods
        .sanitize_for_mode(args.score.stats.mode);
    let modded_gamemode = ModdedGamemode::from_mode_and_mods(args.score.stats.mode, sanitized_mods);

    let duplicate = scores::checksum_exists(ctx, modded_gamemode, &args.score.score_md5).await?;
    if duplicate {
        warn!(
            user_id = user.user_id,
            score_md5 = args.score.score_md5,
            "User submitted duplicate score"
        );
        return Ok(ScoreSubmissionResult::Ok);
    }

    let task_ctx = ctx.clone();
    let task_bm_md5 = beatmap.beatmap_md5.clone();
    ctx.spawn(async move {
        _ = users::update_latest_activity(&task_ctx, user.user_id).await;
        _ = users::increment_beatmap_playcount(
            &task_ctx,
            user.user_id,
            &task_bm_md5,
            modded_gamemode,
        )
        .await;
        _ = beatmaps::increment_playcount(&task_ctx, &task_bm_md5, args.score.stats.passed).await;
        _ = associate_multiplayer_score(
            &task_ctx,
            user.user_id,
            args.score.stats.mode,
            args.score.stats.accuracy,
            args.score.stats.score,
            args.score.stats.max_combo as _,
            sanitized_mods,
            args.score.stats.passed,
        )
        .await;
    });

    let old_stats = stats::fetch_one(ctx, user.user_id, modded_gamemode).await?;
    let mut update_stats = UpdateStats::default();
    update_stats.increment_play_count();
    update_stats.increment_total_hits(args.score.stats.accuracy.total_hits(args.score.stats.mode));
    update_stats.increment_total_score(args.score.stats.score);
    // TODO: calculate playtime properly
    let score_time_secs = args.score_time / 1000;
    // if quit || failed => (beatmap.hit_length - (beatmap.total_length - score_time_secs)) / time_multiplier
    // else beatmap.hit_length / time_multiplier
    update_stats.increment_playtime(score_time_secs);

    if args.quit || !args.score.stats.passed {
        let status = match args.quit {
            true => ScoreStatus::Quit,
            false => ScoreStatus::Failed,
        };
        scores::create(
            ctx,
            user.user_id,
            &beatmap.beatmap_md5,
            modded_gamemode,
            args.score.stats.score,
            args.score.stats.accuracy,
            args.score.stats.max_combo,
            args.score.stats.perfect_combo,
            sanitized_mods,
            0.0,
            &args.score.score_md5,
            status,
        )
        .await?;
        stats::update(ctx, user.user_id, modded_gamemode, &update_stats).await?;
        return Ok(ScoreSubmissionResult::Ok);
    }

    if args.score.stats.max_combo as u32 > old_stats.max_combo {
        update_stats.update_max_combo(args.score.stats.max_combo);
    }

    let performance = performance_service::calculate(PerformanceRequest {
        beatmap_id: beatmap.beatmap_id,
        beatmap_md5: &beatmap.beatmap_md5,
        mode: args.score.stats.mode as _,
        mods: sanitized_mods.bits(),
        max_combo: args.score.stats.max_combo as _,
        accuracy: args.score.stats.accuracy.as_percent(args.score.stats.mode),
        miss_count: args.score.stats.accuracy.count_miss as _,
    })
    .await;

    // Skip stats calculation if performance calc fails.
    // The score is kept as a pass that is worth nothing until its performance is recalculated.
    let Ok(performance) = performance else {
        scores::create(
            ctx,
            user.user_id,
            &beatmap.beatmap_md5,
            modded_gamemode,
            args.score.stats.score,
            args.score.stats.accuracy,
            args.score.stats.max_combo,
            args.score.stats.perfect_combo,
            sanitized_mods,
            0.0,
            &args.score.score_md5,
            ScoreStatus::Completed,
        )
        .await?;
        stats::update(ctx, user.user_id, modded_gamemode, &update_stats).await?;
        return Ok(ScoreSubmissionResult::Ok);
    };
    let personal_best =
        scores::fetch_user_best(ctx, user.user_id, &beatmap.beatmap_md5, modded_gamemode).await?;

    let calculated_grade = args
        .score
        .stats
        .accuracy
        .grade(args.score.stats.mode, sanitized_mods);
    // Loved and qualified beatmaps have leaderboards, but do not count towards the stats of a user
    let is_ranked = beatmap.is_ranked();
    let status = match &personal_best {
        None => {
            if is_ranked {
                update_stats.adjust_ranked_score(args.score.stats.score);
                update_stats.increment_grade(calculated_grade);
            }
            ScoreStatus::Best
        }
        Some(pb)
            if pb.performance < performance.pp
                || (pb.performance == performance.pp && pb.score < args.score.stats.score) =>
        {
            scores::set_to_completed(ctx, pb.score_id, modded_gamemode).await?;
            let pb_calculated_grade = pb.accuracy.grade(pb.mode.as_gamemode(), pb.mods);
            if is_ranked {
                let ranked_score_change = args.score.stats.score - pb.score;
                update_stats.adjust_ranked_score(ranked_score_change);
            }
            if is_ranked && calculated_grade != pb_calculated_grade {
                update_stats.decrement_grade(pb_calculated_grade);
                update_stats.increment_grade(calculated_grade);
            }
            ScoreStatus::Best
        }
        Some(_) => ScoreStatus::Completed,
    };

    let score = scores::create(
        ctx,
        user.user_id,
        &beatmap.beatmap_md5,
        modded_gamemode,
        args.score.stats.score,
        args.score.stats.accuracy,
        args.score.stats.max_combo,
        args.score.stats.perfect_combo,
        sanitized_mods,
        performance.pp,
        &args.score.score_md5,
        status,
    )
    .await?;
    info!(
        user_id = user.user_id,
        score_id = score.score_id,
        beatmap_md5 = beatmap.beatmap_md5,
        "{} has submitted a score",
        user.username,
    );

    let task_ctx = ctx.clone();
    let event = SubmissionEvent {
        score_data: &args.event_data.score_data,
        exited_out: args.quit,
        fail_time: args.fail_time,
        visual_settings_b64: &args.event_data.visual_settings_b64,
        updated_beatmap_hash: &args.beatmap_md5,
        storyboard_md5: args.storyboard_md5.as_ref().map(|x| x.as_str()),
        iv_b64: &args.event_data.iv_b64,
        unique_ids: &args.event_data.unique_ids,
        score_time: args.score_time,
        osu_version: &args.osu_version,
        client_hash_b64: &args.event_data.client_hash_b64,
        score_id: score.score_id,
        user_id: user.user_id,
        osu_auth_token: args.anticheat_token.as_ref().map(|x| x.as_str()),
        mode_vn: modded_gamemode.as_gamemode() as u8,
        relax: modded_gamemode.mode_mod() as u8,
    };
    let event_payload = serde_json::to_string(&event);
    ctx.spawn(async move {
        _ = replays::save_replay(&task_ctx, score.score_id, args.replay_data.unwrap()).await;
        _ = scores::publish_score_event(&task_ctx, event_payload).await;
    });

    let mut restricted = !user.privileges.is_publicly_visible();
    let performance_autorestrict_cap =
        performance_caps::fetch_one(ctx, modded_gamemode, sanitized_mods).await?;
    if !restricted
        && is_ranked
        && !user.whitelist.is_whitelisted_for(modded_gamemode)
        && score.performance >= performance_autorestrict_cap
    {
        let reason = format!(
            "Achieved performance exceeds autorestrict limit: {:.2}pp with {:?} on {} (score id {}).",
            score.performance, sanitized_mods, beatmap.song_name, score.score_id,
        );
        users::restrict_user(ctx, user.user_id, &reason).await?;
        restricted = true;
    }

    let mut new_rank = None;
    let old_rank = stats::fetch_global_rank(ctx, user.user_id, modded_gamemode).await?;
    let pb_rank = match &personal_best {
        Some(pb) => Some(
            scores::find_rank(
                ctx,
                user.user_id,
                &beatmap.beatmap_md5,
                modded_gamemode,
                ScoreRankingMetric::Performance(pb.performance),
            )
            .await?,
        ),
        None => None,
    };
    let score_rank = scores::find_rank(
        ctx,
        user.user_id,
        &beatmap.beatmap_md5,
        modded_gamemode,
        ScoreRankingMetric::Performance(score.performance),
    )
    .await?;
    if status == ScoreStatus::Best && !restricted {
        let (new_pp, new_avg_acc) =
            stats::recalculate_weighted(ctx, user.user_id, modded_gamemode).await?;
        if old_stats.pp != new_pp {
            update_stats.update_pp(new_pp);
            update_stats.update_latest_pp_awarded();
            update_stats.update_avg_accuracy(new_avg_acc);
            new_rank =
                stats::update_rank(ctx, user.user_id, modded_gamemode, &user.country, new_pp)
                    .await?;
        }
        // The best play on this beatmap
        if score_rank == 1 {
            let task_ctx = ctx.clone();
            let task_bm_md5 = beatmap.beatmap_md5.clone();
            ctx.spawn(async move {
                _ = first_places::handle_first_place(
                    &task_ctx,
                    &task_bm_md5,
                    modded_gamemode,
                    score.score_id,
                    user.user_id,
                )
                .await;
            });
        }
    }

    let new_stats = update_stats.apply(&old_stats);
    let task_ctx = ctx.clone();
    ctx.spawn(async move {
        _ = stats::update(&task_ctx, user.user_id, modded_gamemode, &update_stats).await;
    });

    let achievements =
        unlock_achievements(ctx, &score, &new_stats, performance.stars, new_rank).await?;

    let beatmap_ranking_chart =
        charts::beatmap_ranking(&beatmap, &personal_best, &score, pb_rank, score_rank);
    let overall_ranking_chart =
        charts::overall_ranking(user.user_id, old_stats, new_stats, old_rank, new_rank);
    /*let clan_ranking_chart = Chart {
        chart_id: "clan",
        chart_url: "https://akatsuki.gg/c/1".to_string(),
        chart_name: "Clan Ranking",
        entries: vec![
            ChartEntry::new_int(
                "rank",
                Some(2),
                1,
            ),
            ChartEntry::new_int(
                "rankedScore",
                Some(old_stats.ranked_score as _),
                new_stats.ranked_score as _,
            ),
            ChartEntry::new_int(
                "totalScore",
                Some(old_stats.total_score as _),
                new_stats.total_score as _,
            ),
            ChartEntry::new_int(
                "maxCombo",
                Some(old_stats.max_combo as _),
                new_stats.max_combo as _,
            ),
            ChartEntry::new_float(
                "accuracy",
                Some(old_stats.avg_accuracy as _),
                new_stats.avg_accuracy as _,
            ),
            ChartEntry::new_int("pp", Some(old_stats.pp as _), new_stats.pp as _),
        ],
    };*/
    let charts = vec![
        beatmap_ranking_chart,
        overall_ranking_chart,
        // clan_ranking_chart,
    ];

    Ok(ScoreSubmissionResult::Charts(beatmap, charts, achievements))
}

async fn associate_multiplayer_score(
    ctx: &RequestContext,
    user_id: i64,
    mode: Gamemode,
    accuracy: Accuracy,
    score: i32,
    max_combo: i32,
    mods: Mods,
    passed: bool,
) -> ServiceResult<()> {
    let Some(user_match) = bancho_service::fetch_player_match_details(user_id).await? else {
        return Ok(());
    };

    match_game_scores::create(
        ctx,
        CreateMatchGameScoreArgs {
            user_id: user_id as _,
            team: user_match.team as _,
            game_id: user_match.game_id as _,
            match_id: user_match.match_id as _,
            mode,
            mods,
            score,
            passed,
            accuracy,
            max_combo,
        },
    )
    .await?;
    Ok(())
}

async fn unlock_achievements(
    ctx: &RequestContext,
    score: &Score,
    stats: &Stats,
    star_rating: f32,
    global_rank: Option<usize>,
) -> ServiceResult<Vec<Achievement>> {
    let user_achievements = users::fetch_achievements(ctx, score.user_id, score.mode).await?;
    let mut new_achievements = Vec::new();
    for achievement in ctx.achievements.iter() {
        if user_achievements.contains(&achievement.id) {
            continue;
        }

        if achievement
            .mode
            .is_some_and(|mode| mode != score.mode.as_gamemode())
        {
            continue;
        }

        if (achievement.condition)(score, stats, star_rating, global_rank) {
            new_achievements.push(achievement.as_model());
        }
    }

    // Unlock Achievements
    let task_ctx = ctx.clone();
    let user_id = score.user_id;
    let mode = score.mode;
    let new_achievement_ids: Vec<i32> = new_achievements
        .iter()
        .map(|achievement| achievement.id)
        .collect();
    ctx.spawn(async move {
        for new_achievement in new_achievement_ids {
            _ = users::unlock_achievement(&task_ctx, new_achievement, user_id, mode).await;
        }
    });

    Ok(new_achievements)
}
