use crate::api::context::RequestContext;
use crate::entities::mode::ModdedGamemode;
use crate::entities::stats::{Stats, UpdateStats};
use chrono::Utc;
use redis::AsyncCommands;
use sqlx::Arguments;
use sqlx::mysql::MySqlArguments;
use std::ops::DerefMut;

pub async fn fetch_one(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> sqlx::Result<Stats> {
    const QUERY: &str = r#"
    SELECT
    user_id, mode, ranked_score, total_score, playcount, replays_watched,
    total_hits, level, avg_accuracy, pp, playtime, xh_count, x_count, sh_count,
    s_count, a_count, b_count, c_count, d_count, max_combo, latest_pp_awarded
    FROM user_stats
    WHERE user_id = ? AND mode = ?"#;
    sqlx::query_as(QUERY)
        .bind(user_id)
        .bind(mode as i16)
        .fetch_one(&ctx.db)
        .await
}

pub async fn update(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
    update: &UpdateStats,
) -> anyhow::Result<()> {
    let mut update_params = Vec::new();
    let mut args = MySqlArguments::default();

    if update.increment_playcount {
        update_params.push("playcount = playcount+1".to_owned());
    }

    if let Some(incr_total_hits) = update.increment_total_hits {
        update_params.push("total_hits = total_hits+?".to_owned());
        args.add(incr_total_hits)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }

    if let Some(incr_total_score) = update.increment_total_score {
        update_params.push("total_score = total_score+?".to_owned());
        args.add(incr_total_score)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }

    if let Some(incr_playtime) = update.increment_playtime {
        update_params.push("playtime = playtime+?".to_owned());
        args.add(incr_playtime)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }

    if let Some(ranked_score_change) = update.ranked_score_change {
        update_params.push("ranked_score = ranked_score+?".to_owned());
        args.add(ranked_score_change)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }

    if let Some(incr_grade) = update.increment_grade {
        update_params.push(format!("{0} = {0}+1", incr_grade.column_name()));
    }

    if let Some(decr_grade) = update.decrement_grade {
        update_params.push(format!("{0} = {0}-1", decr_grade.column_name()));
    }

    if let Some(max_combo) = update.max_combo {
        update_params.push("max_combo = ?".to_owned());
        args.add(max_combo)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }

    if let Some(updated_pp) = update.update_pp {
        update_params.push("pp = ?".to_owned());
        args.add(updated_pp)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }

    if update.update_latest_pp_awarded {
        let now = Utc::now();
        let timestamp = now.timestamp();
        update_params.push("latest_pp_awarded = ?".to_owned());
        args.add(timestamp)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }

    if let Some(updated_avg_accuracy) = update.update_avg_accuracy {
        update_params.push("avg_accuracy = ?".to_owned());
        args.add(updated_avg_accuracy)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }

    if update_params.is_empty() {
        return Ok(());
    }

    let update_params = update_params.join(",");
    let query = format!("UPDATE user_stats SET {update_params} WHERE user_id = ? AND mode = ?");
    args.add(user_id)
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    args.add(mode as i16)
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    sqlx::query_with(&query, args).execute(&ctx.db).await?;
    Ok(())
}

/// Tells bancho that the stats it has cached for the user are outdated
pub async fn publish_update_event(ctx: &RequestContext, user_id: i64) -> anyhow::Result<()> {
    let mut redis = ctx.redis.get().await?;
    let _: () = redis
        .publish("peppy:update_cached_stats", user_id.to_string())
        .await?;
    Ok(())
}

fn make_key(mode: ModdedGamemode) -> String {
    format!("ripple:{}", mode.redis_board())
}

fn make_country_key(mode: ModdedGamemode, country: &str) -> String {
    let country = country.to_lowercase();
    format!("ripple:{}:{country}", mode.redis_board())
}

/// Takes the user off the global and country leaderboards of all game modes
pub async fn remove_from_leaderboards(
    ctx: &RequestContext,
    user_id: i64,
    user_country: &str,
) -> anyhow::Result<()> {
    let mut pipe = redis::pipe();
    for mode in ModdedGamemode::ALL {
        pipe.zrem(make_key(mode), user_id).ignore();
        if user_country != "XX" {
            pipe.zrem(make_country_key(mode, user_country), user_id)
                .ignore();
        }
    }

    let mut redis = ctx.redis.get().await?;
    let _: () = pipe.query_async(redis.deref_mut()).await?;
    Ok(())
}

pub async fn fetch_global_rank(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> anyhow::Result<Option<usize>> {
    let key = make_key(mode);
    let mut redis = ctx.redis.get().await?;
    let leaderboard_index: Option<usize> = redis.zrevrank(key, user_id).await?;
    Ok(leaderboard_index.map(|x| x + 1))
}

pub async fn update_rank(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
    user_country: &str,
    performance: u32,
) -> anyhow::Result<Option<usize>> {
    let key = make_key(mode);

    let mut redis = ctx.redis.get().await?;
    let mut pipe = redis::pipe();
    pipe.atomic().zadd(&key, user_id, performance).ignore();

    if user_country != "XX" {
        let country_key = make_country_key(mode, user_country);
        pipe.zadd(country_key, user_id, performance).ignore();
    }

    pipe.zrevrank(&key, user_id);
    let leaderboard_index: [Option<usize>; 1] = pipe.query_async(redis.deref_mut()).await?;
    Ok(leaderboard_index[0].map(|x| x + 1))
}
