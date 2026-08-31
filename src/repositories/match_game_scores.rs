use crate::api::context::RequestContext;
use crate::entities::match_game_scores::CreateMatchGameScoreArgs;

pub async fn create(ctx: &RequestContext, args: CreateMatchGameScoreArgs) -> sqlx::Result<()> {
    const QUERY: &str = r#"INSERT INTO match_game_scores (
    match_id, game_id, user_id, mode, count_300, count_100,
    count_50, count_miss, count_geki, count_katu, score,
    accuracy, max_combo, mods, passed, team
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#;
    sqlx::query(QUERY)
        .bind(args.match_id)
        .bind(args.game_id)
        .bind(args.user_id)
        .bind(args.mode as i8)
        .bind(args.accuracy.count_300)
        .bind(args.accuracy.count_100)
        .bind(args.accuracy.count_50)
        .bind(args.accuracy.count_miss)
        .bind(args.accuracy.count_geki)
        .bind(args.accuracy.count_katu)
        .bind(args.score)
        .bind(args.accuracy.as_percent(args.mode))
        .bind(args.max_combo)
        .bind(args.mods.bits())
        .bind(args.passed)
        .bind(args.team)
        .execute(&ctx.db)
        .await?;
    Ok(())
}