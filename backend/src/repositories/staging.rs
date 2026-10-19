use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

/// A validated row waiting to be committed.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct StagingRow {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub row_number: i32,
    pub apaar_id: String,
    pub institution_student_id: String,
    pub full_name: Option<String>,
    pub programme_id: Uuid,
    pub academic_year: String,
    pub admission_date: NaiveDate,
    pub admission_status: String,
    pub seat_category: String,
    pub row_status: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn insert(
    pool: &PgPool,
    batch_id: Uuid,
    row_number: i32,
    apaar_id: &str,
    institution_student_id: &str,
    full_name: Option<&str>,
    programme_id: Uuid,
    academic_year: &str,
    admission_date: NaiveDate,
    admission_status: &str,
    seat_category: &str,
    row_status: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO staging_rows
            (id, batch_id, row_number, apaar_id, institution_student_id, full_name, programme_id,
             academic_year, admission_date, admission_status, seat_category, row_status)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
    )
    .bind(Uuid::new_v4())
    .bind(batch_id)
    .bind(row_number)
    .bind(apaar_id)
    .bind(institution_student_id)
    .bind(full_name)
    .bind(programme_id)
    .bind(academic_year)
    .bind(admission_date)
    .bind(admission_status)
    .bind(seat_category)
    .bind(row_status)
    .execute(pool)
    .await
    .map_err(crate::error::AppError::internal)?;
    Ok(())
}

pub async fn list_for_batch(pool: &PgPool, batch_id: Uuid) -> AppResult<Vec<StagingRow>> {
    sqlx::query_as::<_, StagingRow>(
        "SELECT id, batch_id, row_number, apaar_id, institution_student_id, full_name, programme_id,
                academic_year, admission_date, admission_status, seat_category, row_status
         FROM staging_rows WHERE batch_id = $1 ORDER BY row_number",
    )
    .bind(batch_id)
    .fetch_all(pool)
    .await
    .map_err(crate::error::AppError::internal)
}

pub async fn delete_for_batch(pool: &PgPool, batch_id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM staging_rows WHERE batch_id = $1")
        .bind(batch_id)
        .execute(pool)
        .await
        .map_err(crate::error::AppError::internal)?;
    Ok(())
}
