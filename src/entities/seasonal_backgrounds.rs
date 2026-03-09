#[derive(sqlx::FromRow)]
pub struct SeasonalBackground {
    pub id: i32,
    pub enabled: bool,
    pub url: String,
}