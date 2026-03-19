use crate::entities::achievements::Achievement;

pub async fn fetch_all(db: &sqlx::Pool<sqlx::MySql>) -> sqlx::Result<Vec<Achievement>> {
    sqlx::query_as("SELECT id, file, name, `desc`, hidden, mode FROM less_achievements")
        .fetch_all(db)
        .await
}
