use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::policy::AuthenticatedUser;
use crate::error::AppResult;

/// Writes an append-only audit event.
///
/// The summary must never contain raw student data, tokens or APAAR ids.
#[allow(clippy::too_many_arguments)]
pub async fn record(
    pool: &PgPool,
    actor: Option<&AuthenticatedUser>,
    action: &str,
    entity_type: &str,
    entity_id: Option<Uuid>,
    institution_id: Option<Uuid>,
    outcome: &str,
    correlation_id: Option<&str>,
    summary: Option<String>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO audit_events
            (id, actor_id, actor_role, action, entity_type, entity_id, institution_id,
             outcome, correlation_id, summary)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(Uuid::new_v4())
    .bind(actor.map(|a| a.id))
    .bind(actor.map(|a| a.role.as_str()))
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(institution_id)
    .bind(outcome)
    .bind(correlation_id)
    .bind(summary)
    .execute(pool)
    .await
    .map_err(crate::error::AppError::internal)?;
    Ok(())
}

/// Records a denied access attempt. Cross-institution access attempts are
/// logged exactly like successful actions (PRD section 4).
pub async fn record_denied(
    pool: &PgPool,
    actor: &AuthenticatedUser,
    action: &str,
    entity_type: &str,
    entity_id: Option<Uuid>,
    target_institution: Option<Uuid>,
    summary: &str,
) -> AppResult<()> {
    record(
        pool,
        Some(actor),
        action,
        entity_type,
        entity_id,
        target_institution,
        "DENIED",
        None,
        Some(summary.to_string()),
    )
    .await
}
