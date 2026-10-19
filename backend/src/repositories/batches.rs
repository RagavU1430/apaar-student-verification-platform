use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::policy::ScopeFilter;
use crate::error::AppResult;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct BatchRow {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub submitted_by: Uuid,
    pub file_name: String,
    pub file_hash: String,
    pub schema_version: String,
    pub status: String,
    pub row_count: i32,
    pub accepted_count: i32,
    pub warning_count: i32,
    pub duplicate_count: i32,
    pub rejected_count: i32,
    pub needs_review_count: i32,
    pub idempotency_key: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub committed_at: Option<DateTime<Utc>>,
}

/// Aggregate counts for a batch.
#[derive(Clone, Copy, Debug, Default)]
pub struct BatchCounts {
    pub rows: i32,
    pub accepted: i32,
    pub warning: i32,
    pub duplicate: i32,
    pub rejected: i32,
    pub needs_review: i32,
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &PgPool,
    institution_id: Uuid,
    submitted_by: Uuid,
    file_name: &str,
    file_hash: &str,
    schema_version: &str,
    idempotency_key: Option<&str>,
) -> AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO submission_batches
            (id, institution_id, submitted_by, file_name, file_hash, schema_version, status, idempotency_key)
         VALUES ($1,$2,$3,$4,$5,$6,'VALIDATED',$7)
         RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(institution_id)
    .bind(submitted_by)
    .bind(file_name)
    .bind(file_hash)
    .bind(schema_version)
    .bind(idempotency_key)
    .fetch_one(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))?;
    Ok(id)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<Option<BatchRow>> {
    sqlx::query_as::<_, BatchRow>(
        "SELECT id, institution_id, submitted_by, file_name, file_hash, schema_version, status,
                row_count, accepted_count, warning_count, duplicate_count, rejected_count,
                needs_review_count, idempotency_key, submitted_at, committed_at
         FROM submission_batches WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))
}

pub async fn find_by_idempotency_key(
    pool: &PgPool,
    key: &str,
) -> AppResult<Option<BatchRow>> {
    sqlx::query_as::<_, BatchRow>(
        "SELECT id, institution_id, submitted_by, file_name, file_hash, schema_version, status,
                row_count, accepted_count, warning_count, duplicate_count, rejected_count,
                needs_review_count, idempotency_key, submitted_at, committed_at
         FROM submission_batches WHERE idempotency_key = $1",
    )
    .bind(key)
    .fetch_optional(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))
}

pub async fn list(pool: &PgPool, scope: ScopeFilter) -> AppResult<Vec<BatchRow>> {
    let rows = match scope {
        ScopeFilter::All => {
            sqlx::query_as::<_, BatchRow>(
                "SELECT id, institution_id, submitted_by, file_name, file_hash, schema_version, status,
                        row_count, accepted_count, warning_count, duplicate_count, rejected_count,
                        needs_review_count, idempotency_key, submitted_at, committed_at
                 FROM submission_batches ORDER BY submitted_at DESC LIMIT 200",
            )
            .fetch_all(pool)
            .await
        }
        ScopeFilter::Institution(id) => {
            sqlx::query_as::<_, BatchRow>(
                "SELECT id, institution_id, submitted_by, file_name, file_hash, schema_version, status,
                        row_count, accepted_count, warning_count, duplicate_count, rejected_count,
                        needs_review_count, idempotency_key, submitted_at, committed_at
                 FROM submission_batches WHERE institution_id = $1 ORDER BY submitted_at DESC LIMIT 200",
            )
            .bind(id)
            .fetch_all(pool)
            .await
        }
    };
    rows.map_err(|e| crate::error::AppError::internal(e))
}

pub async fn update_counts(pool: &PgPool, id: Uuid, counts: &BatchCounts) -> AppResult<()> {
    sqlx::query(
        "UPDATE submission_batches
         SET row_count = $2, accepted_count = $3, warning_count = $4, duplicate_count = $5,
             rejected_count = $6, needs_review_count = $7
         WHERE id = $1",
    )
    .bind(id)
    .bind(counts.rows)
    .bind(counts.accepted)
    .bind(counts.warning)
    .bind(counts.duplicate)
    .bind(counts.rejected)
    .bind(counts.needs_review)
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))?;
    Ok(())
}

pub async fn set_status(pool: &PgPool, id: Uuid, status: &str) -> AppResult<()> {
    sqlx::query(
        "UPDATE submission_batches
         SET status = $2,
             committed_at = CASE WHEN $2 = 'COMMITTED' THEN now() ELSE committed_at END
         WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))?;
    Ok(())
}
