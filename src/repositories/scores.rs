use crate::api::context::RequestContext;
use crate::entities::accuracy::Accuracy;
use crate::entities::mode::ModdedGamemode;
use crate::entities::mods::Mods;
use crate::entities::scores::{Score, ScoreRankingMetric, ScoreStatus};
use chrono::Utc;
use lapin::BasicProperties;
use lapin::options::BasicPublishOptions;
use lapin::types::ShortString;
use crate::settings::AppSettings;

pub async fn create(
    ctx: &RequestContext,
    user_id: i64,
    beatmap_md5: &str,
    mode: ModdedGamemode,
    score: i32,
    accuracy: Accuracy,
    max_combo: u16,
    perfect_combo: bool,
    mods: Mods,
    performance: f32,
    score_md5: &str,
    status: ScoreStatus,
) -> sqlx::Result<Score> {
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
    if let Some(amqp_channel) = ctx.amqp_channel.as_ref() {
        let payload = args?;
        let settings = AppSettings::get();
        for routing_key in settings.score_submission_routing_keys.iter() {
            amqp_channel.basic_publish(
                ShortString::default(),
                ShortString::from(routing_key.clone()),
                BasicPublishOptions::default(),
                payload.as_bytes(),
                BasicProperties::default(),
            ).await?;
        }
    }

    Ok(())
}