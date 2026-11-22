use score_service::entities::accuracy::Accuracy;
use score_service::entities::mode::ModdedGamemode;
use score_service::entities::mods::Mods;
use score_service::entities::scores::{CreateScoreArgs, ScoreStatus};
use score_service::repositories::scores;
use sqlx::MySqlPool;
use std::pin::pin;
use std::time::Duration;

fn play(checksum: &str, score: i32, performance: f32) -> CreateScoreArgs<'_> {
    CreateScoreArgs {
        user_id: 1,
        beatmap_md5: "test-map",
        mode: ModdedGamemode::Standard,
        score,
        accuracy: Accuracy {
            count_300: 1,
            count_100: 0,
            count_50: 0,
            count_geki: 0,
            count_katu: 0,
            count_miss: 0,
        },
        max_combo: 1,
        perfect_combo: true,
        mods: Mods::empty(),
        performance,
        score_md5: checksum,
    }
}

#[sqlx::test(migrations = false, fixtures("personal_best"))]
#[ignore = "requires a disposable MySQL instance"]
async fn concurrent_first_plays_leave_one_personal_best(db: MySqlPool) -> sqlx::Result<()> {
    let mut account_lock = db.begin().await?;
    sqlx::query("SELECT id FROM users WHERE id = 1 FOR UPDATE")
        .fetch_one(&mut *account_lock)
        .await?;

    let mut first = pin!(scores::create_with_personal_best(
        &db,
        play("first", 100, 10.0)
    ));
    let mut second = pin!(scores::create_with_personal_best(
        &db,
        play("second", 200, 10.0)
    ));

    // Both submissions must wait for the account lock, even with no best score yet.
    let finished = tokio::time::timeout(Duration::from_millis(250), async {
        tokio::select! {
            _ = &mut first => {}
            _ = &mut second => {}
        }
    })
    .await;
    assert!(finished.is_err());

    account_lock.commit().await?;
    tokio::try_join!(first, second)?;

    let rows: Vec<(String, i8)> =
        sqlx::query_as("SELECT checksum, completed FROM scores ORDER BY checksum")
            .fetch_all(&db)
            .await?;
    assert_eq!(
        rows,
        [
            ("first".to_owned(), ScoreStatus::Completed as i8),
            ("second".to_owned(), ScoreStatus::Best as i8),
        ]
    );
    Ok(())
}

#[sqlx::test(migrations = false, fixtures("personal_best"))]
#[ignore = "requires a disposable MySQL instance"]
async fn failed_insert_preserves_previous_best(db: MySqlPool) -> sqlx::Result<()> {
    let (original, _) = scores::create_with_personal_best(&db, play("original", 100, 10.0)).await?;
    let result = scores::create_with_personal_best(&db, play("original", 200, 20.0)).await;
    assert!(matches!(result, Err(sqlx::Error::Database(error)) if error.is_unique_violation()));

    let best_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM scores WHERE completed = 3")
        .fetch_all(&db)
        .await?;
    assert_eq!(best_ids, [original.score_id]);
    Ok(())
}

#[sqlx::test(migrations = false, fixtures("personal_best"))]
#[ignore = "requires a disposable MySQL instance"]
async fn a_submission_consolidates_existing_bests(db: MySqlPool) -> sqlx::Result<()> {
    scores::create(&db, play("older", 100, 10.0), ScoreStatus::Best).await?;
    let best = scores::create(&db, play("higher", 200, 10.0), ScoreStatus::Best).await?;

    let (new_score, previous_best) =
        scores::create_with_personal_best(&db, play("new", 999, 5.0)).await?;

    assert_eq!(new_score.status, ScoreStatus::Completed);
    assert_eq!(
        previous_best.map(|score| score.score_id),
        Some(best.score_id)
    );
    let best_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM scores WHERE completed = 3")
        .fetch_all(&db)
        .await?;
    assert_eq!(best_ids, [best.score_id]);
    Ok(())
}
