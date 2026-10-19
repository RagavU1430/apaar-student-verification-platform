use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub login_identifier: String,
    pub display_name: String,
    pub role: String,
    pub institution_id: Option<Uuid>,
    pub password_hash: String,
    pub status: String,
}

pub async fn find_by_login(pool: &PgPool, login: &str) -> AppResult<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, login_identifier, display_name, role, institution_id, password_hash, status
         FROM users WHERE lower(login_identifier) = lower($1)",
    )
    .bind(login)
    .fetch_optional(pool)
    .await
    .map_err(crate::error::AppError::internal)?;
    Ok(row)
}

pub async fn list(pool: &PgPool) -> AppResult<Vec<UserRow>> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, login_identifier, display_name, role, institution_id, password_hash, status
         FROM users ORDER BY display_name",
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::AppError::internal)
}

pub async fn count(pool: &PgPool) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(crate::error::AppError::internal)
}
