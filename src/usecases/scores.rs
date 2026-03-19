use crate::adapters::performance_service;
use crate::adapters::performance_service::PerformanceRequest;
use crate::api::context::RequestContext;
use crate::common::error::{AppError, ServiceResult};
use crate::entities::mode::ModdedGamemode;
use crate::entities::scores::{Score, ScoreSorting, ScoreStatus};
use crate::entities::stats::{Stats, UpdateStats};
use crate::models::achievements::Achievement;
use crate::models::osu::charts::{Chart, ChartEntry};
use crate::models::osu::score_submission::{ScoreFlags, ScoreSubmissionResult, SubmitScoreArgs};
use crate::repositories::scores;
use crate::usecases::{beatmaps, charts, first_places, performance_caps, stats, users};
use tracing::{info, warn};

pub async fn submit(
    ctx: &RequestContext,
    args: SubmitScoreArgs,
) -> ServiceResult<ScoreSubmissionResult> {
    let user = users::fetch_one_and_auth(ctx, &args.score.username, &args.password_md5).await?;
    if args.anticheat_token.is_none() {
        warn!("Anticheat Token is missing");
        return Err(AppError::ScoresAnticheatTokenMissing);
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
        // TODO: restrict user ()
        warn!("Score is missing Replay");
        return Err(AppError::ScoresMissingReplay);
    }

    let uninstall_md5 = md5::compute(&args.unique_ids.uninstall_md5);
    let uninstall_id_hash = format!("{:x}", uninstall_md5);
    if uninstall_id_hash != args.client_info.uninstall_md5 {
        // TODO: restrict user (uninstall id mismatch)
        warn!("Score has uninstall id mismatch");
        return Err(AppError::ScoresIllegalScore);
    }

    let disk_signature_md5 = md5::compute(&args.unique_ids.disk_signature_md5);
    let disk_signature_hash = format!("{:x}", disk_signature_md5);
    if disk_signature_hash != args.client_info.disk_signature_md5 {
        // TODO: restrict user (disk signature mismatch)
        warn!("Score has disk mismatch");
        return Err(AppError::ScoresIllegalScore);
    }

    if args.beatmap_md5 != args.score.beatmap_md5 {
        // TODO: restrict user (beatmap hash mismatch)
        warn!("Score has beatmap md5 mismatch");
        return Err(AppError::ScoresIllegalScore);
    }

    if args.score_md5() != args.score.score_md5 {
        // TODO: restrict user (score hash mismatch)
        warn!("Score has score hash mismatch");
        return Err(AppError::ScoresIllegalScore);
    }

    // TODO: check for illegal mods (e.g. RXAP)
    if args.score.stats.mods.is_illegal_combination() {
        // TODO: restrict user (illegal mod combination)
        warn!("Score has illegal mod combination");
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
        warn!("Found flags {flags:?} on score from ");
    }

    let task_ctx = ctx.clone();
    tokio::spawn(async move {
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
        warn!("User submitted duplicate score");
        return Ok(ScoreSubmissionResult::Ok);
    }

    let task_ctx = ctx.clone();
    let task_bm_md5 = beatmap.beatmap_md5.clone();
    tokio::spawn(async move {
        _ = users::update_latest_activity(&task_ctx, user.user_id).await;
        _ = users::increment_beatmap_playcount(
            &task_ctx,
            user.user_id,
            &task_bm_md5,
            modded_gamemode,
        )
        .await;
        _ = beatmaps::increment_playcount(&task_ctx, &task_bm_md5, args.score.stats.passed).await;
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

    // TODO: fetch user multiplayer match and insert mp score

    if args.quit || !args.score.stats.passed {
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
            ScoreStatus::Failed,
        )
        .await?;
        stats::update(ctx, user.user_id, modded_gamemode, &update_stats).await?;
        return Ok(ScoreSubmissionResult::Ok);
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

    // Skip stats calculation if performance calc fails
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
            ScoreStatus::Unprocessed,
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
    let status = match &personal_best {
        None => {
            update_stats.adjust_ranked_score(args.score.stats.score);
            update_stats.increment_grade(calculated_grade);
            ScoreStatus::Best
        }
        Some(pb)
            if pb.performance < performance.pp
                || (pb.performance == performance.pp && pb.score < args.score.stats.score) =>
        {
            scores::set_to_completed(ctx, pb.score_id, modded_gamemode).await?;
            let ranked_score_change = args.score.stats.score - pb.score;
            update_stats.adjust_ranked_score(ranked_score_change);

            let pb_calculated_grade = pb.accuracy.grade(pb.mode.as_gamemode(), pb.mods);
            if calculated_grade != pb_calculated_grade {
                update_stats.decrement_grade(pb_calculated_grade);
                update_stats.increment_grade(calculated_grade);
            }
            ScoreStatus::Best
        }
        Some(_) => ScoreStatus::Completed,
    };

    if args.score.stats.max_combo as u32 > old_stats.max_combo {
        update_stats.update_max_combo(args.score.stats.max_combo);
    }

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
        score_id = score.score_id,
        beatmap_md5 = beatmap.beatmap_md5,
        "{} has submitted a score",
        user.username,
    );

    let mut restricted = !user.privileges.is_publicly_visible();
    let performance_autorestrict_cap =
        performance_caps::fetch_one(ctx, modded_gamemode, sanitized_mods).await?;
    if !user.whitelist.is_whitelisted_for(modded_gamemode)
        && score.performance >= performance_autorestrict_cap
    {
        // TODO: restrict user (achieved performance exceeds autorestrict limit)
        restricted = true;
    }

    // TODO: publish score amqp event

    let old_rank = stats::fetch_global_rank(ctx, user.user_id, modded_gamemode).await?;
    let mut new_rank = None;
    let pb_rank = match &personal_best {
        Some(pb) => Some(
            scores::find_rank(
                ctx,
                user.user_id,
                &beatmap.beatmap_md5,
                modded_gamemode,
                ScoreSorting::Performance(pb.performance),
            )
            .await?,
        ),
        None => None,
    };
    let score_rank = scores::find_rank(
        ctx,
        score.score_id,
        &beatmap.beatmap_md5,
        modded_gamemode,
        ScoreSorting::Performance(score.performance),
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
            tokio::spawn(async move {
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
    let achievements =
        unlock_achievements(ctx, &score, &new_stats, performance.stars, new_rank).await?;

    let task_ctx = ctx.clone();
    tokio::spawn(async move {
        _ = stats::update(&task_ctx, user.user_id, modded_gamemode, &update_stats).await;
    });

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
    tokio::spawn(async move {
        for new_achievement in new_achievement_ids {
            _ = users::unlock_achievement(&task_ctx, new_achievement, user_id, mode).await;
        }
    });

    Ok(new_achievements)
}
