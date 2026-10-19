use crate::domain::validation::{CandidateRow, RuleCode};
use crate::error::{AppError, AppResult};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// The versioned CSV template contract.
///
/// Column names are part of the API contract: changing them requires a new
/// `schema_version`.
pub const SCHEMA_VERSION: &str = "csv-v1";

pub const TEMPLATE_COLUMNS: &[&str] = &[
    "apaar_id",
    "institution_student_id",
    "full_name",
    "programme_code",
    "academic_year",
    "admission_date",
    "seat_category",
    "admission_status",
];

/// A downloadable template with one clearly synthetic example row.
pub fn template_bytes() -> String {
    let mut out = String::new();
    out.push_str(&TEMPLATE_COLUMNS.join(","));
    out.push('\n');
    out.push_str("APAAR-DEMO-0000001,INST-DEMO-0001,Sample Student,CSE-01,2026-27,2026-07-01,REGULAR,ACTIVE\n");
    out
}

/// Parses an uploaded CSV into candidate rows.
///
/// File level problems (missing column, unsupported file, row limit) are
/// returned as errors. Row level problems are **not** detected here: they are
/// the job of the validation service.
pub fn parse_csv(bytes: &[u8], max_rows: usize) -> AppResult<Vec<CandidateRow>> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        AppError::row(
            "FILE_ENCODING_INVALID",
            "The file must be UTF-8 encoded CSV.",
        )
    })?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(false)
        .from_reader(text.as_bytes());

    let headers = reader
        .headers()
        .map_err(|e| {
            AppError::row(
                "FILE_UNREADABLE",
                format!("Could not read the header row: {e}"),
            )
        })?
        .clone();

    let index: HashMap<String, usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.trim().to_ascii_lowercase(), i))
        .collect();

    for column in TEMPLATE_COLUMNS {
        if !index.contains_key(*column) {
            return Err(AppError::row(
                "MISSING_COLUMN",
                format!(
                    "The template is missing the required column '{column}'. Expected columns: {}.",
                    TEMPLATE_COLUMNS.join(", ")
                ),
            ));
        }
    }

    let mut rows = Vec::new();
    for (offset, record) in reader.records().enumerate() {
        let record = match record {
            Ok(r) => r,
            Err(e) => {
                return Err(AppError::row(
                    "ROW_MALFORMED",
                    format!("Row {} could not be parsed: {e}", offset + 2),
                ))
            }
        };

        if rows.len() >= max_rows {
            return Err(AppError::row(
                "ROW_LIMIT_EXCEEDED",
                format!("The file contains more than {max_rows} rows."),
            ));
        }

        let get = |name: &str| -> String {
            index
                .get(name)
                .and_then(|i| record.get(*i))
                .unwrap_or("")
                .trim()
                .to_string()
        };

        let full_name = get("full_name");
        rows.push(CandidateRow {
            // +2: one for the header, one for 1-based row numbering.
            row_number: offset + 2,
            apaar_id: get("apaar_id"),
            institution_student_id: get("institution_student_id"),
            full_name: if full_name.is_empty() {
                None
            } else {
                Some(full_name)
            },
            programme_code: get("programme_code"),
            academic_year: get("academic_year"),
            admission_date: get("admission_date"),
            seat_category: get("seat_category"),
            admission_status: get("admission_status"),
        });
    }

    if rows.is_empty() {
        return Err(AppError::row(
            "FILE_EMPTY",
            "The file contains a header but no data rows.",
        ));
    }

    Ok(rows)
}

/// SHA-256 of the uploaded bytes, used for batch fingerprinting and
/// idempotency.
pub fn file_hash(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// The rule code reported when the file itself cannot be processed.
pub fn file_rule_code(code: &str) -> RuleCode {
    match code {
        "FILE_TOO_LARGE" => RuleCode::FileTooLarge,
        "FILE_UNSUPPORTED_TYPE" => RuleCode::FileUnsupportedType,
        _ => RuleCode::MissingColumn,
    }
}

/// Summary of a commit attempt.
#[derive(Clone, Debug, Default)]
pub struct CommitSummary {
    pub committed: i64,
    pub skipped: i64,
}

/// Commits the accepted rows of a batch inside a single transaction.
///
/// Either every accepted row is persisted or none is: a failed commit must not
/// leave a partial, ambiguous state (NFR-01). Rows that would create a second
/// active admission for the same student are skipped and reported as a new
/// validation issue; the original findings are never removed.
pub async fn commit_batch(
    pool: &PgPool,
    batch_id: Uuid,
    institution_id: Uuid,
    actor_id: Uuid,
    rule_version: &str,
) -> AppResult<CommitSummary> {
    use crate::domain::admission::normalise_apaar_id;

    let mut tx = pool
        .begin()
        .await
        .map_err(crate::error::AppError::internal)?;

    let batch_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM submission_batches WHERE id = $1 FOR UPDATE")
            .bind(batch_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(crate::error::AppError::internal)?;

    let Some(batch_status) = batch_status else {
        return Err(crate::error::AppError::not_found("Batch not found."));
    };

    if batch_status == "COMMITTED" {
        return Err(crate::error::AppError::conflict(
            "This batch has already been committed.",
        ));
    }

    let rows = crate::repositories::staging::list_for_batch(pool, batch_id).await?;
    let mut summary = CommitSummary::default();

    for row in rows {
        if !matches!(
            row.row_status.as_str(),
            "ACCEPTED" | "ACCEPTED_WITH_WARNING"
        ) {
            summary.skipped += 1;
            continue;
        }

        let normalised = normalise_apaar_id(&row.apaar_id);
        let last4: String = normalised
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        let student_id: Uuid = sqlx::query_scalar(
            "INSERT INTO students (id, apaar_id, apaar_id_last4) VALUES ($1,$2,$3)
             ON CONFLICT (apaar_id) DO UPDATE SET apaar_id = EXCLUDED.apaar_id RETURNING id",
        )
        .bind(Uuid::new_v4())
        .bind(&normalised)
        .bind(&last4)
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::error::AppError::internal)?;

        let identity_id: Uuid = sqlx::query_scalar(
            "INSERT INTO institution_students
                (id, student_id, institution_id, institution_student_id, full_name)
             VALUES ($1,$2,$3,$4,$5)
             ON CONFLICT (institution_id, institution_student_id)
             DO UPDATE SET full_name = COALESCE(EXCLUDED.full_name, institution_students.full_name)
             RETURNING id",
        )
        .bind(Uuid::new_v4())
        .bind(student_id)
        .bind(institution_id)
        .bind(&row.institution_student_id)
        .bind(&row.full_name)
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::error::AppError::internal)?;

        let already_active: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM admissions
                WHERE student_identity_id = $1
                  AND programme_id = $2
                  AND academic_year = $3
                  AND status IN ('ACTIVE','ENROLLED')
             )",
        )
        .bind(identity_id)
        .bind(row.programme_id)
        .bind(&row.academic_year)
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::error::AppError::internal)?;

        if already_active {
            summary.skipped += 1;
            sqlx::query(
                "INSERT INTO validation_issues
                    (id, batch_id, institution_id, row_reference, rule_code, severity, message,
                     field, evidence, rule_version, status)
                 VALUES ($1,$2,$3,$4,'DUPLICATE_ACTIVE_ADMISSION','REVIEW',
                         'Skipped at commit: the student already has an active admission for this programme and year.',
                         'apaar_id', NULL, $5, 'OPEN')",
            )
            .bind(Uuid::new_v4())
            .bind(batch_id)
            .bind(institution_id)
            .bind(format!("row {}", row.row_number))
            .bind(rule_version)
            .execute(&mut *tx)
            .await
            .map_err(crate::error::AppError::internal)?;
            continue;
        }

        sqlx::query(
            "INSERT INTO admissions
                (id, student_identity_id, institution_id, programme_id, academic_year,
                 admission_date, status, seat_category, source_batch_id)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(Uuid::new_v4())
        .bind(identity_id)
        .bind(institution_id)
        .bind(row.programme_id)
        .bind(&row.academic_year)
        .bind(row.admission_date)
        .bind(&row.admission_status)
        .bind(&row.seat_category)
        .bind(batch_id)
        .execute(&mut *tx)
        .await
        .map_err(crate::error::AppError::internal)?;

        summary.committed += 1;
    }

    sqlx::query(
        "UPDATE submission_batches SET status = 'COMMITTED', committed_at = now() WHERE id = $1",
    )
    .bind(batch_id)
    .execute(&mut *tx)
    .await
    .map_err(crate::error::AppError::internal)?;

    // The staging area is cleared once the records are persisted.
    sqlx::query("DELETE FROM staging_rows WHERE batch_id = $1")
        .bind(batch_id)
        .execute(&mut *tx)
        .await
        .map_err(crate::error::AppError::internal)?;

    sqlx::query(
        "INSERT INTO audit_events
            (id, actor_id, actor_role, action, entity_type, entity_id, institution_id, outcome, summary)
         VALUES ($1,$2,'SYSTEM','batch.commit','submission_batch',$3,$4,'SUCCESS',$5)",
    )
    .bind(Uuid::new_v4())
    .bind(actor_id)
    .bind(batch_id)
    .bind(institution_id)
    .bind(format!("committed {} records, skipped {}", summary.committed, summary.skipped))
    .execute(&mut *tx)
    .await
    .map_err(crate::error::AppError::internal)?;

    tx.commit()
        .await
        .map_err(crate::error::AppError::internal)?;

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_template() {
        let rows = parse_csv(template_bytes().as_bytes(), 100).expect("parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].row_number, 2);
        assert_eq!(rows[0].apaar_id, "APAAR-DEMO-0000001");
        assert_eq!(rows[0].seat_category, "REGULAR");
    }

    #[test]
    fn missing_column_is_a_file_level_error() {
        let csv = "apaar_id,institution_student_id\nX,Y\n";
        let err = parse_csv(csv.as_bytes(), 100).unwrap_err();
        assert!(matches!(err, AppError::Row { ref code, .. } if code == "MISSING_COLUMN"));
    }

    #[test]
    fn row_limit_is_enforced() {
        let mut csv = TEMPLATE_COLUMNS.join(",");
        csv.push('\n');
        for i in 0..5 {
            csv.push_str(&format!(
                "APAAR-0000000{i},S{i},N,CSE-01,2026-27,2026-07-01,REGULAR,ACTIVE\n"
            ));
        }
        let err = parse_csv(csv.as_bytes(), 3).unwrap_err();
        assert!(matches!(err, AppError::Row { ref code, .. } if code == "ROW_LIMIT_EXCEEDED"));
    }

    #[test]
    fn empty_file_is_rejected() {
        let csv = TEMPLATE_COLUMNS.join(",");
        let err = parse_csv(csv.as_bytes(), 10).unwrap_err();
        assert!(matches!(err, AppError::Row { ref code, .. } if code == "FILE_EMPTY"));
    }
}
