use chrono::NaiveDate;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::config::Config;
use crate::domain::admission::{AdmissionRecord, AdmissionStatus, SeatCategory, normalise_apaar_id};
use crate::domain::intake::CategoryTally;
use crate::domain::validation::{
    CandidateRow, Severity, ValidationContext, ValidationFinding, ValidationStatus, status_for,
    validate_row,
};
use crate::error::AppResult;
use crate::repositories::{admissions, institutions, students};
use crate::repositories::batches::BatchCounts;
use crate::services::duplicate_detection::in_batch_duplicates;
use regex::Regex;

/// The result of validating one row.
#[derive(Clone, Debug)]
pub struct RowOutcome {
    pub row_number: usize,
    pub status: ValidationStatus,
    pub findings: Vec<ValidationFinding>,
    /// Populated when the row is free of error-level findings.
    pub candidate: Option<AdmissionRecord>,
    pub programme_id: Option<Uuid>,
}

/// The result of validating a whole batch.
#[derive(Clone, Debug)]
pub struct BatchValidation {
    pub outcomes: Vec<RowOutcome>,
    pub counts: BatchCounts,
}

/// Builds the validation context for an institution and validates every row.
pub async fn validate_rows(
    pool: &PgPool,
    institution_id: Uuid,
    rows: &[CandidateRow],
    config: &Config,
) -> AppResult<BatchValidation> {
    let programmes = institutions::list_programmes(pool, institution_id).await?;
    let programme_ids: HashMap<String, Uuid> = programmes
        .iter()
        .map(|p| (p.code.to_ascii_uppercase(), p.id))
        .collect();
    let known_programmes: HashSet<String> = programme_ids.keys().cloned().collect();

    let intakes = institutions::list_intakes(pool, institution_id).await?;
    let configured_years: HashSet<String> =
        intakes.iter().map(|i| i.academic_year.clone()).collect();

    let institution_apaar_ids = students::active_apaar_ids_for_institution(pool, institution_id).await?;
    let global_active_apaar_ids = students::active_apaar_ids_global(pool).await?;
    let duplicates = in_batch_duplicates(rows);

    let apaar_pattern = Regex::new(&config.apaar_demo_pattern).map_err(|e| {
        crate::error::AppError::internal(anyhow::anyhow!("invalid APAAR pattern: {e}"))
    })?;

    let mut outcomes = Vec::with_capacity(rows.len());
    let mut counts = BatchCounts::default();

    for row in rows {
        let year = row.academic_year.trim().to_ascii_uppercase();
        let (period_start, period_end) = admission_period(&year);

        let ctx = ValidationContext {
            known_programmes: known_programmes.clone(),
            configured_years: configured_years.clone(),
            institution_apaar_ids: institution_apaar_ids.clone(),
            global_active_apaar_ids: global_active_apaar_ids.clone(),
            in_batch_duplicates: duplicates.clone(),
            admission_period_start: period_start,
            admission_period_end: period_end,
            apaar_pattern: apaar_pattern.clone(),
            rule_version: config.demo_rule_version.clone(),
        };

        let findings = validate_row(row, &ctx);
        let status = status_for(&findings);
        let programme_id = programme_ids.get(&row.programme_code.trim().to_ascii_uppercase()).copied();

        let candidate = if status == ValidationStatus::Rejected {
            None
        } else {
            build_candidate(row)
        };

        counts.rows += 1;
        match status {
            ValidationStatus::Accepted => counts.accepted += 1,
            ValidationStatus::AcceptedWithWarning => {
                counts.accepted += 1;
                counts.warning += 1;
            }
            ValidationStatus::NeedsReview => {
                counts.needs_review += 1;
                if findings.iter().any(|f| f.severity == Severity::Duplicate) {
                    counts.duplicate += 1;
                }
            }
            ValidationStatus::Rejected => counts.rejected += 1,
        }

        outcomes.push(RowOutcome {
            row_number: row.row_number,
            status,
            findings,
            candidate,
            programme_id,
        });
    }

    Ok(BatchValidation { outcomes, counts })
}

/// Admission window derived from the academic year.
///
/// This is demo configuration, not an official AICTE admission schedule.
fn admission_period(year: &str) -> (NaiveDate, NaiveDate) {
    let start_year: i32 = year
        .get(0..4)
        .and_then(|y| y.parse().ok())
        .unwrap_or(2026);
    let start = NaiveDate::from_ymd_opt(start_year, 6, 1).unwrap_or(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
    let end = NaiveDate::from_ymd_opt(start_year + 1, 9, 30)
        .unwrap_or(NaiveDate::from_ymd_opt(2027, 9, 30).unwrap());
    (start, end)
}

/// Converts a candidate row into a persistable record, when the values allow it.
fn build_candidate(row: &CandidateRow) -> Option<AdmissionRecord> {
    let apaar_id = normalise_apaar_id(&row.apaar_id);
    if apaar_id.is_empty() {
        return None;
    }
    let date = NaiveDate::parse_from_str(row.admission_date.trim(), "%Y-%m-%d").ok()?;
    let status = AdmissionStatus::parse(&row.admission_status)?;
    let seat_category = SeatCategory::parse(&row.seat_category)?;
    let last4: String = apaar_id.chars().rev().take(4).collect::<String>().chars().rev().collect();

    Some(AdmissionRecord {
        apaar_id,
        apaar_id_last4: last4,
        institution_student_id: row.institution_student_id.trim().to_string(),
        full_name: row.full_name.clone(),
        programme_code: row.programme_code.trim().to_ascii_uppercase(),
        academic_year: row.academic_year.trim().to_ascii_uppercase(),
        admission_date: date,
        status,
        seat_category,
    })
}

/// Builds the per-category tallies used by the intake rule.
pub fn tallies_from_counts(
    counts: &[(String, i64)],
    rules: &HashMap<String, (bool, bool)>,
) -> Vec<CategoryTally> {
    counts
        .iter()
        .map(|(category, count)| {
            let (regular, supern) = rules
                .get(category)
                .copied()
                .unwrap_or((category == "REGULAR", category != "REGULAR"));
            CategoryTally {
                seat_category: category.clone(),
                count: *count,
                counts_toward_regular: regular,
                uses_supernumerary_pool: supern,
            }
        })
        .collect()
}

/// Active admission counts for a programme/year, grouped by seat category.
pub async fn category_counts(
    pool: &PgPool,
    programme_id: Uuid,
    academic_year: &str,
    institution_id: Option<Uuid>,
) -> AppResult<Vec<(String, i64)>> {
    let rows = admissions::active_tallies(pool, institution_id).await?;
    Ok(rows
        .into_iter()
        .filter(|(pid, year, _, _)| *pid == programme_id && year == academic_year)
        .map(|(_, _, category, count)| (category, count))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_period_spans_two_calendar_years() {
        let (start, end) = admission_period("2026-27");
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2027, 9, 30).unwrap());
    }

    #[test]
    fn unknown_year_falls_back_to_a_safe_window() {
        let (start, end) = admission_period("nonsense");
        assert!(start <= end);
    }

    #[test]
    fn candidate_is_not_built_from_an_unparsable_date() {
        let row = CandidateRow {
            row_number: 2,
            apaar_id: "APAAR-1".into(),
            institution_student_id: "S1".into(),
            full_name: None,
            programme_code: "CSE-01".into(),
            academic_year: "2026-27".into(),
            admission_date: "not-a-date".into(),
            seat_category: "REGULAR".into(),
            admission_status: "ACTIVE".into(),
        };
        assert!(build_candidate(&row).is_none());
    }
}
