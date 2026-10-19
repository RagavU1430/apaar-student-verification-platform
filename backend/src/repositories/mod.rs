pub mod admissions;
pub mod batches;
pub mod institutions;
pub mod issues;
pub mod staging;
pub mod students;
pub mod users;

use sqlx::PgPool;

use crate::auth::policy::ScopeFilter;
use crate::error::AppResult;

/// Adds an institution scope predicate to a WHERE fragment.
///
/// This is the single place where "which institutions may this query touch" is
/// decided, so that no route can accidentally query across institutions.
pub fn scope_clause<'a>(
    scope: ScopeFilter,
    column: &str,
    args: &'a mut Vec<sqlx::types::Uuid>,
) -> String {
    match scope {
        ScopeFilter::All => String::new(),
        ScopeFilter::Institution(id) => {
            args.push(id);
            format!(" AND {column} = ${}", args.len())
        }
    }
}

/// Runs a ping against the database for the readiness probe.
pub async fn ping(pool: &PgPool) -> AppResult<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| crate::error::AppError::internal(e))?;
    Ok(())
}
