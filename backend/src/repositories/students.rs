use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

use crate::error::AppResult;

/// APAAR ids with an active admission in the given institution.
pub async fn active_apaar_ids_for_institution(
    pool: &PgPool,
    institution_id: Uuid,
) -> AppResult<HashSet<String>> {
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT s.apaar_id
         FROM admissions a
         JOIN institution_students ist ON ist.id = a.student_identity_id
         JOIN students s ON s.id = ist.student_id
         WHERE a.institution_id = $1 AND a.status IN ('ACTIVE','ENROLLED')",
    )
    .bind(institution_id)
    .fetch_all(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))?;
    Ok(rows.into_iter().collect())
}

/// APAAR ids with an active admission anywhere.
///
/// Used for cross-institution conflict detection. The result is only used to
/// raise a review flag; it never discloses another institution's record.
pub async fn active_apaar_ids_global(pool: &PgPool) -> AppResult<HashSet<String>> {
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT s.apaar_id
         FROM admissions a
         JOIN institution_students ist ON ist.id = a.student_identity_id
         JOIN students s ON s.id = ist.student_id
         WHERE a.status IN ('ACTIVE','ENROLLED')",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))?;
    Ok(rows.into_iter().collect())
}

/// Finds or creates the central student record for an APAAR id.
pub async fn upsert_student(pool: &PgPool, apaar_id: &str, last4: &str) -> AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO students (id, apaar_id, apaar_id_last4)
         VALUES ($1, $2, $3)
         ON CONFLICT (apaar_id) DO UPDATE SET apaar_id = EXCLUDED.apaar_id
         RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(apaar_id)
    .bind(last4)
    .fetch_one(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))?;
    Ok(id)
}

/// Finds or creates the institution-scoped student record.
pub async fn upsert_institution_student(
    pool: &PgPool,
    student_id: Uuid,
    institution_id: Uuid,
    institution_student_id: &str,
    full_name: Option<&str>,
) -> AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO institution_students
            (id, student_id, institution_id, institution_student_id, full_name)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (institution_id, institution_student_id)
         DO UPDATE SET full_name = COALESCE(EXCLUDED.full_name, institution_students.full_name)
         RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(student_id)
    .bind(institution_id)
    .bind(institution_student_id)
    .bind(full_name)
    .fetch_one(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))?;
    Ok(id)
}

/// The institution-scoped student id behind an APAAR id, if it exists.
pub async fn find_institution_student(
    pool: &PgPool,
    apaar_id: &str,
    institution_id: Uuid,
) -> AppResult<Option<Uuid>> {
    let id: Option<Uuid> = sqlx::query_scalar(
        "SELECT ist.id
         FROM institution_students ist
         JOIN students s ON s.id = ist.student_id
         WHERE s.apaar_id = $1 AND ist.institution_id = $2",
    )
    .bind(apaar_id)
    .bind(institution_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| crate::error::AppError::internal(e))?;
    Ok(id)
}
