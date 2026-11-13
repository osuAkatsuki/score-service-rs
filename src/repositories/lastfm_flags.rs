use crate::api::context::RequestContext;

/// The explanation of the flags is cut off at the size of its column
const FLAG_TEXT_MAX_LENGTH: usize = 512;

pub async fn create(
    ctx: &RequestContext,
    user_id: i64,
    flags: i32,
    explanation: &str,
) -> sqlx::Result<()> {
    const QUERY: &str = r#"
    INSERT INTO lastfm_flags (user_id, timestamp, flag_enum, flag_text)
    VALUES (?, UNIX_TIMESTAMP(), ?, ?)"#;

    let flag_text: String = explanation.chars().take(FLAG_TEXT_MAX_LENGTH).collect();
    sqlx::query(QUERY)
        .bind(user_id)
        .bind(flags)
        .bind(flag_text)
        .execute(&ctx.db)
        .await?;
    Ok(())
}
