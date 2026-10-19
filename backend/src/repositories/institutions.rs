use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::policy::ScopeFilter;
use crate::domain::intake::SeatRule;
use crate::error::AppResult;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Institution {
    pub id: Uuid,
    pub official_code: String,
    pub name: String,
    pub status: String,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Programme {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub code: String,
    pub name: String,
    pub level: String,
    pub status: String,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct ApprovedIntake {
    pub id: Uuid,
    pub programme_id: Uuid,
    pub academic_year: String,
    pub regular_intake: i32,
    pub supernumerary_capacity: i32,
    pub source_reference: String,
    pub rule_version: String,
    pub effective_version: i32,
}

/// Institutions visible to the caller, filtered server-side by scope.
pub async fn list_visible(pool: &PgPool, scope: ScopeFilter) -> AppResult<Vec<Institution>> {
    let rows = match scope {
        ScopeFilter::All => {
            sqlx::query_as::<_, Institution>(
                "SELECT id, official_code, name, status FROM institutions ORDER BY name",
            )
            .fetch_all(pool)
            .await
        }
        ScopeFilter::Institution(id) => sqlx::query_as::<_, Institution>(
            "SELECT id, official_code, name, status FROM institutions WHERE id = $1 ORDER BY name",
        )
        .bind(id)
        .fetch_all(pool)
        .await,
    };
    rows.map_err(crate::error::AppError::internal)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<Option<Institution>> {
    sqlx::query_as::<_, Institution>(
        "SELECT id, official_code, name, status FROM institutions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(crate::error::AppError::internal)
}

pub async fn list_programmes(pool: &PgPool, institution_id: Uuid) -> AppResult<Vec<Programme>> {
    sqlx::query_as::<_, Programme>(
        "SELECT id, institution_id, code, name, level, status
         FROM programmes WHERE institution_id = $1 ORDER BY code",
    )
    .bind(institution_id)
    .fetch_all(pool)
    .await
    .map_err(crate::error::AppError::internal)
}

/// Approved intakes for one institution, joined with the programme code.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct IntakeWithProgramme {
    pub programme_id: Uuid,
    pub institution_id: Uuid,
    pub programme_code: String,
    pub programme_name: String,
    pub academic_year: String,
    pub regular_intake: i32,
    pub supernumerary_capacity: i32,
    pub source_reference: String,
    pub rule_version: String,
    pub effective_version: i32,
}

pub async fn list_intakes(
    pool: &PgPool,
    institution_id: Uuid,
) -> AppResult<Vec<IntakeWithProgramme>> {
    sqlx::query_as::<_, IntakeWithProgramme>(
        "SELECT i.programme_id, p.institution_id, p.code AS programme_code, p.name AS programme_name,
                i.academic_year, i.regular_intake, i.supernumerary_capacity,
                i.source_reference, i.rule_version, i.effective_version
         FROM approved_intakes i
         JOIN programmes p ON p.id = i.programme_id
         WHERE p.institution_id = $1
         ORDER BY p.code, i.academic_year, i.effective_version DESC",
    )
    .bind(institution_id)
    .fetch_all(pool)
    .await
    .map_err(crate::error::AppError::internal)
}

/// Approved intakes across every institution (used by the AICTE dashboard).
pub async fn list_intakes_all(pool: &PgPool) -> AppResult<Vec<IntakeWithProgramme>> {
    sqlx::query_as::<_, IntakeWithProgramme>(
        "SELECT i.programme_id, p.code AS programme_code, p.name AS programme_name,
                p.institution_id, i.academic_year, i.regular_intake, i.supernumerary_capacity,
                i.source_reference, i.rule_version, i.effective_version
         FROM approved_intakes i
         JOIN programmes p ON p.id = i.programme_id
         ORDER BY p.code, i.academic_year, i.effective_version DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(crate::error::AppError::internal)
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct IntakeRuleRow {
    pub seat_category: String,
    pub counts_toward_regular: bool,
    pub uses_supernumerary_pool: bool,
    pub notes: Option<String>,
}

/// Counting rules for a rule version. Policy is data, not code.
pub async fn intake_rules(pool: &PgPool, rule_version: &str) -> AppResult<Vec<SeatRule>> {
    let rows = sqlx::query_as::<_, IntakeRuleRow>(
        "SELECT seat_category, counts_toward_regular, uses_supernumerary_pool, notes
         FROM intake_rules WHERE rule_version = $1 ORDER BY seat_category",
    )
    .bind(rule_version)
    .fetch_all(pool)
    .await
    .map_err(crate::error::AppError::internal)?;

    Ok(rows
        .into_iter()
        .map(|r| SeatRule {
            seat_category: r.seat_category,
            counts_toward_regular: r.counts_toward_regular,
            uses_supernumerary_pool: r.uses_supernumerary_pool,
            notes: r.notes,
        })
        .collect())
}
