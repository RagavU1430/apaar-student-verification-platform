use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::policy::ScopeFilter;
use crate::error::AppResult;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct IssueRow {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub institution_id: Uuid,
    pub row_reference: String,
    pub rule_code: String,
    pub severity: String,
    pub message: String,
    pub field: Option<String>,
    pub evidence: Option<String>,
    pub rule_version: String,
    pub status: String,
    pub resolution_note: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub async fn insert(
    pool: &PgPool,
    batch_id: Uuid,
    institution_id: Uuid,
    row_reference: &str,
    rule_code: &str,
    severity: &str,
    message: &str,
    field: Option<&str>,
    evidence: Option<&str>,
    rule_version: &str,
) -> AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO validation_issues
            (id, batch_id, institution_id, row_reference, rule_code, severity, message,
             field, evidence, rule_version, status)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'OPEN')
         RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(batch_id)
    .bind(institution_id)
    .bind(row_reference)
    .bind(rule_code)
    .bind(severity)
    .bind(message)
    .bind(field)
    .bind(evidence)
    .bind(rule_version)
    .fetch_one(pool)
    .await
    .map_err(crate::error::AppError::internal)?;
    Ok(id)
}

pub async fn list_for_batch(pool: &PgPool, batch_id: Uuid) -> AppResult<Vec<IssueRow>> {
    sqlx::query_as::<_, IssueRow>(
        "SELECT id, batch_id, institution_id, row_reference, rule_code, severity, message, field,
                evidence, rule_version, status, resolution_note, resolved_by, resolved_at, created_at
         FROM validation_issues WHERE batch_id = $1
         ORDER BY row_reference, severity DESC, rule_code",
    )
    .bind(batch_id)
    .fetch_all(pool)
    .await
    .map_err(crate::error::AppError::internal)
}

/// Open issues in the caller's scope, most severe first.
pub async fn list_open(pool: &PgPool, scope: ScopeFilter, limit: i64) -> AppResult<Vec<IssueRow>> {
    let rows = match scope {
        ScopeFilter::All => {
            sqlx::query_as::<_, IssueRow>(
                "SELECT id, batch_id, institution_id, row_reference, rule_code, severity, message, field,
                        evidence, rule_version, status, resolution_note, resolved_by, resolved_at, created_at
                 FROM validation_issues WHERE status = 'OPEN'
                 ORDER BY CASE severity WHEN 'ERROR' THEN 0 WHEN 'REVIEW' THEN 1 WHEN 'DUPLICATE' THEN 2 ELSE 3 END,
                          created_at DESC
                 LIMIT $1",
            )
            .bind(limit)
            .fetch_all(pool)
            .await
        }
        ScopeFilter::Institution(id) => {
            sqlx::query_as::<_, IssueRow>(
                "SELECT id, batch_id, institution_id, row_reference, rule_code, severity, message, field,
                        evidence, rule_version, status, resolution_note, resolved_by, resolved_at, created_at
                 FROM validation_issues WHERE status = 'OPEN' AND institution_id = $1
                 ORDER BY CASE severity WHEN 'ERROR' THEN 0 WHEN 'REVIEW' THEN 1 WHEN 'DUPLICATE' THEN 2 ELSE 3 END,
                          created_at DESC
                 LIMIT $2",
            )
            .bind(id)
            .bind(limit)
            .fetch_all(pool)
            .await
        }
    };
    rows.map_err(crate::error::AppError::internal)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<Option<IssueRow>> {
    sqlx::query_as::<_, IssueRow>(
        "SELECT id, batch_id, institution_id, row_reference, rule_code, severity, message, field,
                evidence, rule_version, status, resolution_note, resolved_by, resolved_at, created_at
         FROM validation_issues WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(crate::error::AppError::internal)
}

/// Records a review outcome. The original finding is never deleted.
pub async fn resolve(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    note: &str,
    resolved_by: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE validation_issues
         SET status = $2, resolution_note = $3, resolved_by = $4, resolved_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(note)
    .bind(resolved_by)
    .execute(pool)
    .await
    .map_err(crate::error::AppError::internal)?;
    Ok(())
}

/// Counts of open issues per rule code, used by the AICTE dashboard.
pub async fn open_counts_by_rule(
    pool: &PgPool,
    scope: ScopeFilter,
) -> AppResult<Vec<(String, i64)>> {
    let rows = match scope {
        ScopeFilter::All => {
            sqlx::query_as::<_, (String, i64)>(
                "SELECT rule_code, count(*) FROM validation_issues WHERE status = 'OPEN'
                 GROUP BY rule_code ORDER BY count(*) DESC",
            )
            .fetch_all(pool)
            .await
        }
        ScopeFilter::Institution(id) => {
            sqlx::query_as::<_, (String, i64)>(
                "SELECT rule_code, count(*) FROM validation_issues WHERE status = 'OPEN' AND institution_id = $1
                 GROUP BY rule_code ORDER BY count(*) DESC",
            )
            .bind(id)
            .fetch_all(pool)
            .await
        }
    };
    rows.map_err(crate::error::AppError::internal)
}
