use score_service::api::context::RequestContext;
use score_service::common::axum_ip::IpAddrInfo;
use score_service::common::redis_pool::{RedisPool, RedisPoolManager};
use score_service::entities::accuracy::Accuracy;
use score_service::entities::mode::ModdedGamemode;
use score_service::entities::mods::Mods;
use score_service::entities::scores::{CreateScoreArgs, ScoreStatus};
use score_service::repositories::scores;
use sqlx::MySqlPool;
use std::net::Ipv4Addr;
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
        status: ScoreStatus::Best,
    }
}

#[sqlx::test(migrations = false, fixtures("personal_best"))]
#[ignore = "requires a disposable MySQL instance"]
async fn concurrent_first_plays_leave_one_personal_best(db: MySqlPool) -> sqlx::Result<()> {
    let ctx = RequestContext {
        db,
        // The Redis pool connects lazily and is unused in this test.
        redis: RedisPool::builder(RedisPoolManager::new(
            redis::Client::open("redis://127.0.0.1/").unwrap(),
            Default::default(),
        ))
        .build()
        .unwrap(),
        amqp_publisher: None,
        s3_client: None,
        achievements: &[],
        background_tasks: Default::default(),
        request_ip: IpAddrInfo {
            ip_addr: Ipv4Addr::LOCALHOST.into(),
        },
        user_agent: None,
    };
    let mut account_lock = ctx.db.begin().await?;
    sqlx::query("SELECT id FROM users WHERE id = 1 FOR UPDATE")
        .fetch_one(&mut *account_lock)
        .await?;

    let mut first = pin!(scores::create(&ctx, play("first", 100, 10.0)));
    let mut second = pin!(scores::create(&ctx, play("second", 200, 10.0)));

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

    // A failed replacement must roll back the previous best's demotion.
    let failed = scores::create(&ctx, play("second", 300, 20.0)).await;
    assert!(matches!(failed, Err(sqlx::Error::Database(error)) if error.is_unique_violation()));

    let rows: Vec<(String, i8)> =
        sqlx::query_as("SELECT checksum, completed FROM scores ORDER BY checksum")
            .fetch_all(&ctx.db)
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
