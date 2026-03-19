#[derive(sqlx::FromRow)]
pub struct Achievement {
    pub id: i32,
    pub file: String,
    pub name: String,
    pub desc: String,
    pub hidden: bool,
    pub mode: Option<i8>,
}
