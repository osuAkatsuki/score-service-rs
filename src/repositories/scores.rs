use crate::api::context::RequestContext;
use crate::entities::mode::{GamemodeMod, ModdedGamemode};
use crate::entities::scores::{
    CreateScoreArgs, LeaderboardFilter, LeaderboardScore, Score, ScoreRankingMetric,
};
use chrono::Utc;
use sqlx::Arguments;
use sqlx::mysql::MySqlArguments;
use crate::settings::AppSettings;

pub async fn create(ctx: &RequestContext, args: CreateScoreArgs<'_>) -> sqlx::Result<Score> {
    let CreateScoreArgs {
        user_id,
        beatmap_md5,
        mode,
        score,
        accuracy,
        max_combo,
        perfect_combo,
        mods,
        performance,
        score_md5,
        status,
    } = args;
    let query = format!(
        r#"INSERT INTO {} (
        beatmap_md5, userid, score, max_combo, full_combo, mods,
        300_count, 100_count, 50_count, katus_count, gekis_count, misses_count,
        time, play_mode, completed, accuracy, pp, checksum
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        mode.scores_table(),
    );
    let now = Utc::now();
    let vn_mode = mode.as_gamemode();
    let accuracy_percent = accuracy.as_percent(vn_mode);
    let query_result = sqlx::query(&query)
        .bind(beatmap_md5)
        .bind(user_id)
        .bind(score)
        .bind(max_combo)
        .bind(perfect_combo)
        .bind(mods.bits())
        .bind(accuracy.count_300)
        .bind(accuracy.count_100)
        .bind(accuracy.count_50)
        .bind(accuracy.count_katu)
        .bind(accuracy.count_geki)
        .bind(accuracy.count_miss)
        .bind(now.timestamp())
        .bind(mode.as_gamemode() as u8)
        .bind(status as u8)
        .bind(accuracy_percent)
        .bind(performance)
        .bind(score_md5)
        .execute(&ctx.db)
        .await?;

    Ok(Score {
        mode,
        mods,
        score,
        status,
        user_id,
        accuracy,
        performance,
        perfect_combo,
        score_id: query_result.last_insert_id() as _,
        beatmap_md5: beatmap_md5.to_owned(),
        max_combo: max_combo as _,
        score_md5: Some(score_md5.to_owned()),
        created_at: now,
    })
}

/// Builds the part all beatmap leaderboard queries have in common:
/// `user_bests` numbers the matching scores of every user starting with their best one,
/// `leaderboard` ranks those best scores against each other.
/// Restricted users only ever show up on their own leaderboards.
fn leaderboard_query(
    mode: ModdedGamemode,
    display_pp_as_score: bool,
    filter: &LeaderboardFilter<'_>,
    select: &str,
) -> String {
    let sort_column = match display_pp_as_score {
        true => "pp",
        false => "score",
    };
    format!(
        r#"WITH user_bests AS (
        SELECT users.username AS username, c.tag AS clan_tag,
        s.id AS score_id, s.userid AS user_id, s.score AS score, s.max_combo AS max_combo,
        s.full_combo AS full_combo, s.mods AS mods, s.`300_count` AS `300_count`,
        s.`100_count` AS `100_count`, s.`50_count` AS `50_count`, s.gekis_count AS gekis_count,
        s.katus_count AS katus_count, s.misses_count AS misses_count, s.time AS time, s.pp AS pp,
        ROW_NUMBER() OVER (PARTITION BY s.userid ORDER BY s.{sort_column} DESC, s.time ASC) AS user_score_rank
        FROM {} s
        INNER JOIN users ON users.id = s.userid
        LEFT JOIN clans c ON c.id = users.clan_id
        WHERE s.beatmap_md5 = ?
        AND s.play_mode = ?
        AND s.completed {}
        AND (users.privileges & 1 = 1 OR users.id = ?)
        {}
    ), leaderboard AS (
        SELECT *, CAST(ROW_NUMBER() OVER (ORDER BY {sort_column} DESC, time ASC) AS SIGNED) AS score_rank
        FROM user_bests
        WHERE user_score_rank = 1
    )
    {select}"#,
        mode.scores_table(),
        filter.status_query_part(),
        filter.query_part(),
    )
}

fn leaderboard_arguments(
    user_id: i64,
    beatmap_md5: &str,
    mode: ModdedGamemode,
    filter: &LeaderboardFilter<'_>,
) -> sqlx::Result<MySqlArguments> {
    let mut args = MySqlArguments::default();
    args.add(beatmap_md5).map_err(sqlx::Error::Encode)?;
    args.add(mode.as_gamemode() as u8)
        .map_err(sqlx::Error::Encode)?;
    args.add(user_id).map_err(sqlx::Error::Encode)?;

    match filter {
        LeaderboardFilter::None => {}
        LeaderboardFilter::Friends(friend_ids) => {
            // The user is part of their own friends leaderboard
            args.add(user_id).map_err(sqlx::Error::Encode)?;
            for friend_id in friend_ids {
                args.add(*friend_id).map_err(sqlx::Error::Encode)?;
            }
        }
        LeaderboardFilter::Mods(mods) => args.add(mods.bits()).map_err(sqlx::Error::Encode)?,
        LeaderboardFilter::Country(country_code) => {
            args.add(*country_code).map_err(sqlx::Error::Encode)?
        }
    }

    Ok(args)
}

pub async fn fetch_leaderboard_view(
    ctx: &RequestContext,
    user_id: i64,
    beatmap_md5: &str,
    mode: ModdedGamemode,
    display_pp_as_score: bool,
    filter: &LeaderboardFilter<'_>,
    leaderboard_size: i32,
) -> sqlx::Result<Vec<LeaderboardScore>> {
    let query = leaderboard_query(
        mode,
        display_pp_as_score,
        filter,
        "SELECT * FROM leaderboard ORDER BY score_rank LIMIT ?",
    );
    let mut args = leaderboard_arguments(user_id, beatmap_md5, mode, filter)?;
    args.add(leaderboard_size).map_err(sqlx::Error::Encode)?;
    sqlx::query_as_with(&query, args).fetch_all(&ctx.db).await
}

/// Fetches the score of the user on a leaderboard, ranked among everyone else on it.
pub async fn fetch_leaderboard_personal_best(
    ctx: &RequestContext,
    user_id: i64,
    beatmap_md5: &str,
    mode: ModdedGamemode,
    display_pp_as_score: bool,
    filter: &LeaderboardFilter<'_>,
) -> sqlx::Result<Option<LeaderboardScore>> {
    let query = leaderboard_query(
        mode,
        display_pp_as_score,
        filter,
        "SELECT * FROM leaderboard WHERE user_id = ?",
    );
    let mut args = leaderboard_arguments(user_id, beatmap_md5, mode, filter)?;
    args.add(user_id).map_err(sqlx::Error::Encode)?;
    sqlx::query_as_with(&query, args)
        .fetch_optional(&ctx.db)
        .await
}

pub async fn fetch_leaderboard_scores_count(
    ctx: &RequestContext,
    user_id: i64,
    beatmap_md5: &str,
    mode: ModdedGamemode,
    display_pp_as_score: bool,
    filter: &LeaderboardFilter<'_>,
) -> sqlx::Result<i64> {
    let query = leaderboard_query(
        mode,
        display_pp_as_score,
        filter,
        "SELECT COUNT(*) FROM leaderboard",
    );
    let args = leaderboard_arguments(user_id, beatmap_md5, mode, filter)?;
    sqlx::query_scalar_with(&query, args)
        .fetch_one(&ctx.db)
        .await
}

pub async fn find_rank(
    ctx: &RequestContext,
    user_id: i64,
    beatmap_md5: &str,
    mode: ModdedGamemode,
    ranking_metric: ScoreRankingMetric,
) -> sqlx::Result<i64> {
    let query = format!(
        r#"SELECT COUNT(*) FROM {}
        INNER JOIN users ON users.id = userid
        WHERE beatmap_md5 = ?
        AND play_mode = ?
        AND completed = 3
        AND userid != ?
        AND {} >= ?
        AND (users.privileges & 1) = 1"#,
        mode.scores_table(),
        ranking_metric.column_name(),
    );
    let count: i64 = sqlx::query_scalar(&query)
        .bind(beatmap_md5)
        .bind(mode.as_gamemode() as u8)
        .bind(user_id)
        .bind(ranking_metric)
        .fetch_one(&ctx.db)
        .await?;
    Ok(count + 1)
}

/// Counts the scores of every kind, no matter whether they are still worth anything
pub async fn fetch_total_count(ctx: &RequestContext) -> sqlx::Result<i64> {
    const QUERY: &str = r#"
        SELECT (SELECT COUNT(*) FROM scores)
             + (SELECT COUNT(*) FROM scores_relax)
             + (SELECT COUNT(*) FROM scores_ap)
    "#;

    sqlx::query_scalar(QUERY).fetch_one(&ctx.db).await
}

pub async fn fetch_one(ctx: &RequestContext, score_id: i64) -> sqlx::Result<Option<Score>> {
    let table_name = GamemodeMod::from_score_id(score_id).scores_table();
    let query = format!(
        r#"
            SELECT id, beatmap_md5, userid, score, max_combo, full_combo, mods,
            `300_count`, `100_count`, `50_count`, katus_count, gekis_count, misses_count,
            time, play_mode, completed, accuracy, pp, checksum FROM {table_name}
            WHERE id = ?
        "#
    );

    sqlx::query_as(&query)
        .bind(score_id)
        .fetch_optional(&ctx.db)
        .await
}

pub async fn fetch_user_best(
    ctx: &RequestContext,
    user_id: i64,
    beatmap_md5: &str,
    mode: ModdedGamemode,
) -> sqlx::Result<Option<Score>> {
    let table_name = mode.scores_table();
    let query = format!(
        r#"
            SELECT id, beatmap_md5, userid, score, max_combo, full_combo, mods,
            300_count, 100_count, 50_count, katus_count, gekis_count, misses_count,
            time, play_mode, completed, accuracy, pp, checksum FROM {table_name}
            WHERE userid = ?
            AND beatmap_md5 = ?
            AND play_mode = ?
            AND completed = 3
            ORDER BY pp DESC
            LIMIT 1
        "#
    );

    sqlx::query_as(&query)
        .bind(user_id)
        .bind(beatmap_md5)
        .bind(mode.as_gamemode() as u8)
        .fetch_optional(&ctx.db)
        .await
}

pub async fn fetch_user_top_plays(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> sqlx::Result<Vec<Score>> {
    let table_name = mode.scores_table();
    let query = format!(
        r#"
            SELECT s.id, s.beatmap_md5, s.userid, s.score, s.max_combo, s.full_combo, s.mods,
            300_count, s.100_count, s.50_count, s.katus_count, s.gekis_count, s.misses_count,
            time, s.play_mode, s.completed, s.accuracy, s.pp, s.checksum
            FROM {table_name} s
            JOIN beatmaps b USING(beatmap_md5)
            WHERE s.completed = 3
            AND s.play_mode = ?
            AND b.ranked IN (3, 2)
            AND s.userid = ?
            ORDER BY s.pp DESC
            LIMIT 100
        "#
    );

    sqlx::query_as(&query)
        .bind(mode.as_gamemode() as u8)
        .bind(user_id)
        .fetch_all(&ctx.db)
        .await
}

pub async fn set_to_completed(
    ctx: &RequestContext,
    score_id: i64,
    mode: ModdedGamemode,
) -> sqlx::Result<()> {
    let query = format!(
        "UPDATE {} SET completed = 2 WHERE id = ?",
        mode.scores_table()
    );
    sqlx::query(&query).bind(score_id).execute(&ctx.db).await?;
    Ok(())
}

pub async fn fetch_user_ranked_scores_count(
    ctx: &RequestContext,
    user_id: i64,
    mode: ModdedGamemode,
) -> sqlx::Result<i64> {
    let query = format!(
        r#"
        SELECT COUNT(*) FROM {} s
        JOIN beatmaps b USING(beatmap_md5)
        WHERE b.ranked IN (2, 3) AND s.completed = 3
        AND s.play_mode = ? AND s.userid = ?
        LIMIT 1000"#,
        mode.scores_table()
    );

    sqlx::query_scalar(&query)
        .bind(mode.as_gamemode() as u8)
        .bind(user_id)
        .fetch_one(&ctx.db)
        .await
}

pub async fn checksum_exists(
    ctx: &RequestContext,
    mode: ModdedGamemode,
    checksum: &str,
) -> sqlx::Result<bool> {
    let query = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE checksum = ?)",
        mode.scores_table()
    );
    sqlx::query_scalar(&query)
        .bind(checksum)
        .fetch_one(&ctx.db)
        .await
}

pub async fn publish_score_event(
    ctx: &RequestContext,
    args: serde_json::Result<String>,
) -> anyhow::Result<()> {
    if let Some(amqp_publisher) = ctx.amqp_publisher.as_ref() {
        let payload = args?;
        let settings = AppSettings::get();
        for routing_key in settings.score_submission_routing_keys.iter() {
            amqp_publisher.publish(routing_key, payload.as_bytes()).await?;
        }
    }

    Ok(())
}