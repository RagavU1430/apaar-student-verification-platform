use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::policy::ScopeFilter;
use crate::error::AppResult;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct AdmissionView {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub programme_code: String,
    pub academic_year: String,
    pub admission_date: NaiveDate,
    pub status: String,
    pub seat_category: String,
    pub institution_student_id: String,
    pub full_name: Option<String>,
    /// Masked APAAR id. The full value is never returned in list views.
    pub apaar_id_masked: String,
    pub apaar_id_last4: String,
}

/// Filters accepted by the admission search endpoint.
#[derive(Clone, Debug, Default)]
pub struct AdmissionFilters {
    pub programme_code: Option<String>,
    pub academic_year: Option<String>,
    pub status: Option<String>,
    pub seat_category: Option<String>,
    pub search: Option<String>,
}

/// Inserts an admission record.
#[allow(clippy::too_many_arguments)]
pub async fn insert(
    pool: &PgPool,
    student_identity_id: Uuid,
    institution_id: Uuid,
    programme_id: Uuid,
    academic_year: &str,
    admission_date: NaiveDate,
    status: &str,
    seat_category: &str,
    source_batch_id: Option<Uuid>,
) -> AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO admissions
            (id, student_identity_id, institution_id, programme_id, academic_year,
             admission_date, status, seat_category, source_batch_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(student_identity_id)
    .bind(institution_id)
    .bind(programme_id)
    .bind(academic_year)
    .bind(admission_date)
    .bind(status)
    .bind(seat_category)
    .bind(source_batch_id)
    .fetch_one(pool)
    .await
    .map_err(crate::error::AppError::internal)?;
    Ok(id)
}

/// True when the student already has an active admission for that programme/year.
pub async fn has_active_admission(
    pool: &PgPool,
    student_identity_id: Uuid,
    programme_id: Uuid,
    academic_year: &str,
) -> AppResult<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM admissions
            WHERE student_identity_id = $1
              AND programme_id = $2
              AND academic_year = $3
              AND status IN ('ACTIVE','ENROLLED')
         )",
    )
    .bind(student_identity_id)
    .bind(programme_id)
    .bind(academic_year)
    .fetch_one(pool)
    .await
    .map_err(crate::error::AppError::internal)?;
    Ok(exists)
}

/// Active admission tallies per programme, year and seat category.
pub async fn active_tallies(
    pool: &PgPool,
    institution_id: Option<Uuid>,
) -> AppResult<Vec<(Uuid, String, String, i64)>> {
    let rows = match institution_id {
        Some(id) => {
            sqlx::query_as(
                "SELECT programme_id, academic_year, seat_category, count(*)
                 FROM admissions
                 WHERE institution_id = $1 AND status IN ('ACTIVE','ENROLLED')
                 GROUP BY programme_id, academic_year, seat_category",
            )
            .bind(id)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as(
                "SELECT programme_id, academic_year, seat_category, count(*)
                 FROM admissions
                 WHERE status IN ('ACTIVE','ENROLLED')
                 GROUP BY programme_id, academic_year, seat_category",
            )
            .fetch_all(pool)
            .await
        }
    };
    rows.map_err(crate::error::AppError::internal)
}

/// Searches admissions within the caller's scope.
pub async fn search(
    pool: &PgPool,
    scope: ScopeFilter,
    filters: &AdmissionFilters,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<AdmissionView>, i64)> {
    // The argument order below is: [scope uuid?] then text filters, then
    // LIMIT/OFFSET. Every predicate is appended in exactly that order.
    let mut predicates: Vec<String> = Vec::new();
    let mut text_values: Vec<String> = Vec::new();
    let mut arg_index: usize = 1;

    let scope_predicate = match scope {
        ScopeFilter::All => String::new(),
        ScopeFilter::Institution(_id) => {
            arg_index += 1;
            format!(" AND a.institution_id = ${}", arg_index - 1)
        }
    };

    for (value, column) in [
        (filters.programme_code.as_deref(), "p.code"),
        (filters.academic_year.as_deref(), "a.academic_year"),
        (filters.status.as_deref(), "a.status"),
        (filters.seat_category.as_deref(), "a.seat_category"),
    ] {
        if let Some(v) = value.filter(|v| !v.is_empty()) {
            predicates.push(format!(" AND {column} = ${arg_index}"));
            text_values.push(v.to_string());
            arg_index += 1;
        }
    }

    if let Some(term) = filters.search.as_ref().filter(|v| !v.is_empty()) {
        predicates.push(format!(
            " AND (s.apaar_id_last4 = ${arg_index} OR ist.institution_student_id = ${arg_index} OR ist.full_name ILIKE ${arg_index})"
        ));
        text_values.push(term.clone());
        arg_index += 1;
    }

    let where_sql = format!("WHERE 1=1{scope_predicate}{}", predicates.join(""));
    let from_sql = "FROM admissions a
         JOIN institution_students ist ON ist.id = a.student_identity_id
         JOIN students s ON s.id = ist.student_id
         JOIN programmes p ON p.id = a.programme_id";

    let list_sql = format!(
        "SELECT a.id, a.institution_id, p.code AS programme_code, a.academic_year, a.admission_date,
                a.status, a.seat_category, ist.institution_student_id, ist.full_name,
                s.apaar_id_last4,
                ('********' || s.apaar_id_last4) AS apaar_id_masked
         {from_sql}
         {where_sql}
         ORDER BY a.created_at DESC, a.id
         LIMIT ${arg_index} OFFSET ${arg_offset}",
        from_sql = from_sql,
        where_sql = where_sql,
        arg_index = arg_index,
        arg_offset = arg_index + 1,
    );

    let count_sql = format!("SELECT count(*) {from_sql} {where_sql}");

    let mut list_query = sqlx::query_as::<_, AdmissionView>(&list_sql);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);

    if let ScopeFilter::Institution(id) = scope {
        list_query = list_query.bind(id);
        count_query = count_query.bind(id);
    }
    for value in &text_values {
        list_query = list_query.bind(value.clone());
        count_query = count_query.bind(value.clone());
    }

    let total = count_query
        .fetch_one(pool)
        .await
        .map_err(crate::error::AppError::internal)?;

    let rows = list_query
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::internal)?;

    Ok((rows, total))
}
