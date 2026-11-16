use serde::Serialize;

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_md5: String,
    pub privileges: i32,
    pub whitelist: i8,
    pub country: String,
    pub vanilla_pp_leaderboards: bool,
    pub leaderboard_size: Option<i32>,
}

#[derive(Serialize)]
pub struct UsernameChange {
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "newUsername")]
    pub new_username: String,
}
