use super::admission::{normalise_apaar_id, AdmissionStatus, SeatCategory};
use chrono::NaiveDate;
use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// Stable reason codes. These are part of the API contract and the error
/// report downloaded by institutions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum RuleCode {
    // Structural (file level)
    FileUnsupportedType,
    FileTooLarge,
    MissingColumn,
    RowLimitExceeded,
    // Field level
    ApaarIdMissing,
    ApaarIdFormatInvalid,
    StudentIdMissing,
    FullNameMissing,
    ProgrammeCodeMissing,
    ProgrammeUnknown,
    AcademicYearMissing,
    AcademicYearInvalid,
    AcademicYearNotConfigured,
    AdmissionDateMissing,
    AdmissionDateInvalid,
    AdmissionDateOutsidePeriod,
    SeatCategoryUnknown,
    AdmissionStatusInvalid,
    // Uniqueness
    DuplicateApaarInBatch,
    CrossInstitutionActiveConflict,
    // Referential / capacity
    IntakeNotConfigured,
    IntakeCapacityExceeded,
    // Consistency
    DuplicateActiveAdmission,
}

impl RuleCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleCode::FileUnsupportedType => "FILE_UNSUPPORTED_TYPE",
            RuleCode::FileTooLarge => "FILE_TOO_LARGE",
            RuleCode::MissingColumn => "MISSING_COLUMN",
            RuleCode::RowLimitExceeded => "ROW_LIMIT_EXCEEDED",
            RuleCode::ApaarIdMissing => "APAAR_ID_MISSING",
            RuleCode::ApaarIdFormatInvalid => "APAAR_ID_FORMAT_INVALID",
            RuleCode::StudentIdMissing => "STUDENT_ID_MISSING",
            RuleCode::FullNameMissing => "FULL_NAME_MISSING",
            RuleCode::ProgrammeCodeMissing => "PROGRAMME_CODE_MISSING",
            RuleCode::ProgrammeUnknown => "PROGRAMME_UNKNOWN",
            RuleCode::AcademicYearMissing => "ACADEMIC_YEAR_MISSING",
            RuleCode::AcademicYearInvalid => "ACADEMIC_YEAR_INVALID",
            RuleCode::AcademicYearNotConfigured => "ACADEMIC_YEAR_NOT_CONFIGURED",
            RuleCode::AdmissionDateMissing => "ADMISSION_DATE_MISSING",
            RuleCode::AdmissionDateInvalid => "ADMISSION_DATE_INVALID",
            RuleCode::AdmissionDateOutsidePeriod => "ADMISSION_DATE_OUTSIDE_PERIOD",
            RuleCode::SeatCategoryUnknown => "SEAT_CATEGORY_UNKNOWN",
            RuleCode::AdmissionStatusInvalid => "ADMISSION_STATUS_INVALID",
            RuleCode::DuplicateApaarInBatch => "DUPLICATE_APAAR_IN_BATCH",
            RuleCode::CrossInstitutionActiveConflict => "CROSS_INSTITUTION_ACTIVE_CONFLICT",
            RuleCode::IntakeNotConfigured => "INTAKE_NOT_CONFIGURED",
            RuleCode::IntakeCapacityExceeded => "INTAKE_CAPACITY_EXCEEDED",
            RuleCode::DuplicateActiveAdmission => "DUPLICATE_ACTIVE_ADMISSION",
        }
    }
}

/// Severity drives the row status and the dashboard buckets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Severity {
    Warning,
    Duplicate,
    Review,
    Error,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Warning => "WARNING",
            Severity::Duplicate => "DUPLICATE",
            Severity::Review => "REVIEW",
            Severity::Error => "ERROR",
        }
    }
}

/// One finding against one row.
#[derive(Clone, Debug, Serialize)]
pub struct ValidationFinding {
    pub rule_code: RuleCode,
    pub severity: Severity,
    pub message: String,
    pub field: Option<String>,
    /// The concrete values that triggered the rule (no unnecessary PII).
    pub evidence: Option<String>,
    pub rule_version: String,
}

impl ValidationFinding {
    fn new(
        rule_code: RuleCode,
        severity: Severity,
        message: impl Into<String>,
        field: Option<&str>,
        evidence: Option<String>,
        rule_version: &str,
    ) -> Self {
        Self {
            rule_code,
            severity,
            message: message.into(),
            field: field.map(|f| f.to_string()),
            evidence,
            rule_version: rule_version.to_string(),
        }
    }
}

/// Outcome status of a single row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ValidationStatus {
    Accepted,
    AcceptedWithWarning,
    NeedsReview,
    Rejected,
}

impl ValidationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationStatus::Accepted => "ACCEPTED",
            ValidationStatus::AcceptedWithWarning => "ACCEPTED_WITH_WARNING",
            ValidationStatus::NeedsReview => "NEEDS_REVIEW",
            ValidationStatus::Rejected => "REJECTED",
        }
    }
}

/// Derives the row status from its findings.
///
/// Errors block the commit. Review and duplicate findings hold the row for a
/// human decision. Warnings do not block the commit.
pub fn status_for(findings: &[ValidationFinding]) -> ValidationStatus {
    if findings.iter().any(|f| f.severity == Severity::Error) {
        return ValidationStatus::Rejected;
    }
    if findings
        .iter()
        .any(|f| matches!(f.severity, Severity::Review | Severity::Duplicate))
    {
        return ValidationStatus::NeedsReview;
    }
    if findings.iter().any(|f| f.severity == Severity::Warning) {
        return ValidationStatus::AcceptedWithWarning;
    }
    ValidationStatus::Accepted
}

/// A raw row as parsed from the CSV template. Values are strings because the
/// point of validation is to decide whether they are usable.
#[derive(Clone, Debug)]
pub struct CandidateRow {
    pub row_number: usize,
    pub apaar_id: String,
    pub institution_student_id: String,
    pub full_name: Option<String>,
    pub programme_code: String,
    pub academic_year: String,
    pub admission_date: String,
    pub seat_category: String,
    pub admission_status: String,
}

/// Everything the rules need that is not on the row itself.
#[derive(Clone, Debug)]
pub struct ValidationContext {
    /// Programme codes configured for the uploading institution.
    pub known_programmes: HashSet<String>,
    /// Academic years that have an approved-intake configuration.
    pub configured_years: HashSet<String>,
    /// APAAR ids already active in the uploading institution.
    pub institution_apaar_ids: HashSet<String>,
    /// APAAR ids active in any institution (for cross-institution conflicts).
    pub global_active_apaar_ids: HashSet<String>,
    /// APAAR ids seen more than once inside the current batch -> row numbers.
    pub in_batch_duplicates: HashMap<String, Vec<usize>>,
    /// Accepted admission window for the academic year.
    pub admission_period_start: NaiveDate,
    pub admission_period_end: NaiveDate,
    /// Level 2 shape check. Demo only - never identity verification.
    pub apaar_pattern: Regex,
    pub rule_version: String,
}

/// Validates one candidate row against every applicable rule.
///
/// The APAAR id is checked at level 1 (presence) and level 2 (shape). A value
/// that passes level 2 must never be described as "APAAR verified".
pub fn validate_row(row: &CandidateRow, ctx: &ValidationContext) -> Vec<ValidationFinding> {
    let mut findings = Vec::new();
    let rv = ctx.rule_version.as_str();

    // --- APAAR id -------------------------------------------------------
    let normalised = normalise_apaar_id(&row.apaar_id);
    if normalised.is_empty() {
        findings.push(ValidationFinding::new(
            RuleCode::ApaarIdMissing,
            Severity::Error,
            "APAAR id is required. Provide the identifier exactly as issued.",
            Some("apaar_id"),
            None,
            rv,
        ));
    } else if !ctx.apaar_pattern.is_match(&normalised) {
        findings.push(ValidationFinding::new(
            RuleCode::ApaarIdFormatInvalid,
            Severity::Error,
            "APAAR id does not match the expected shape. This is a format check only, not identity verification.",
            Some("apaar_id"),
            Some("level 2 format check (demo rule)"),
            rv,
        ));
    }

    // --- Institution student id ----------------------------------------
    if row.institution_student_id.trim().is_empty() {
        findings.push(ValidationFinding::new(
            RuleCode::StudentIdMissing,
            Severity::Error,
            "Institution student id is required.",
            Some("institution_student_id"),
            None,
            rv,
        ));
    }

    // --- Name (warning only; personal data is minimised) ---------------
    if row.full_name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        findings.push(ValidationFinding::new(
            RuleCode::FullNameMissing,
            Severity::Warning,
            "Student name is missing. Records are accepted but the name may be incomplete.",
            Some("full_name"),
            None,
            rv,
        ));
    }

    // --- Programme ------------------------------------------------------
    let programme_code = row.programme_code.trim().to_ascii_uppercase();
    if programme_code.is_empty() {
        findings.push(ValidationFinding::new(
            RuleCode::ProgrammeCodeMissing,
            Severity::Error,
            "Programme code is required.",
            Some("programme_code"),
            None,
            rv,
        ));
    } else if !ctx.known_programmes.contains(&programme_code) {
        findings.push(ValidationFinding::new(
            RuleCode::ProgrammeUnknown,
            Severity::Error,
            "Programme is not configured for this institution. Ask an administrator to add it.",
            Some("programme_code"),
            Some(programme_code.clone()),
            rv,
        ));
    }

    // --- Academic year --------------------------------------------------
    let year = row.academic_year.trim().to_ascii_uppercase();
    if year.is_empty() {
        findings.push(ValidationFinding::new(
            RuleCode::AcademicYearMissing,
            Severity::Error,
            "Academic year is required.",
            Some("academic_year"),
            None,
            rv,
        ));
    } else if !is_plausible_academic_year(&year) {
        findings.push(ValidationFinding::new(
            RuleCode::AcademicYearInvalid,
            Severity::Error,
            "Academic year must look like 2026-27.",
            Some("academic_year"),
            Some(year.clone()),
            rv,
        ));
    } else if !ctx.configured_years.contains(&year) {
        findings.push(ValidationFinding::new(
            RuleCode::AcademicYearNotConfigured,
            Severity::Error,
            "No approved intake is configured for this programme and academic year.",
            Some("academic_year"),
            Some(year.clone()),
            rv,
        ));
    }

    // --- Admission date --------------------------------------------------
    let date = row.admission_date.trim();
    if date.is_empty() {
        findings.push(ValidationFinding::new(
            RuleCode::AdmissionDateMissing,
            Severity::Error,
            "Admission date is required in YYYY-MM-DD format.",
            Some("admission_date"),
            None,
            rv,
        ));
    } else {
        match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            Err(_) => findings.push(ValidationFinding::new(
                RuleCode::AdmissionDateInvalid,
                Severity::Error,
                "Admission date must be a valid date in YYYY-MM-DD format.",
                Some("admission_date"),
                Some(date.to_string()),
                rv,
            )),
            Ok(parsed) => {
                if parsed < ctx.admission_period_start || parsed > ctx.admission_period_end {
                    findings.push(ValidationFinding::new(
                        RuleCode::AdmissionDateOutsidePeriod,
                        Severity::Error,
                        "Admission date falls outside the configured admission period.",
                        Some("admission_date"),
                        Some(format!(
                            "{} (allowed {} to {})",
                            parsed, ctx.admission_period_start, ctx.admission_period_end
                        )),
                        rv,
                    ));
                }
            }
        }
    }

    // --- Seat category ----------------------------------------------------
    if SeatCategory::parse(&row.seat_category).is_none() {
        findings.push(ValidationFinding::new(
            RuleCode::SeatCategoryUnknown,
            Severity::Error,
            "Seat category is not recognised. Use one of the configured categories.",
            Some("seat_category"),
            Some(row.seat_category.trim().to_string()),
            rv,
        ));
    }

    // --- Admission status --------------------------------------------------
    if AdmissionStatus::parse(&row.admission_status).is_none() {
        findings.push(ValidationFinding::new(
            RuleCode::AdmissionStatusInvalid,
            Severity::Error,
            "Admission status is not recognised.",
            Some("admission_status"),
            Some(row.admission_status.trim().to_string()),
            rv,
        ));
    }

    // --- Duplicates ---------------------------------------------------------
    if !normalised.is_empty() {
        if let Some(rows) = ctx.in_batch_duplicates.get(&normalised) {
            findings.push(ValidationFinding::new(
                RuleCode::DuplicateApaarInBatch,
                Severity::Duplicate,
                "The same APAAR id appears more than once in this batch. A reviewer must decide which row is correct.",
                Some("apaar_id"),
                Some(format!("rows {}", rows.join(", "))),
                rv,
            ));
        }
        if ctx.institution_apaar_ids.contains(&normalised) {
            findings.push(ValidationFinding::new(
                RuleCode::DuplicateActiveAdmission,
                Severity::Review,
                "This APAAR id already has an active admission in this institution.",
                Some("apaar_id"),
                Some("existing active admission"),
                rv,
            ));
        } else if ctx.global_active_apaar_ids.contains(&normalised) {
            findings.push(ValidationFinding::new(
                RuleCode::CrossInstitutionActiveConflict,
                Severity::Review,
                "This APAAR id is linked to an active admission at another institution. This is a review flag, not a fraud verdict.",
                Some("apaar_id"),
                Some("active admission at another institution"),
                rv,
            ));
        }
    }

    findings
}

/// `2026-27` style academic year.
pub fn is_plausible_academic_year(year: &str) -> bool {
    let bytes = year.as_bytes();
    if bytes.len() != 7 || bytes[4] != b'-' {
        return false;
    }
    let start: &str = &year[0..4];
    let end: &str = &year[5..7];
    start.parse::<u16>().is_ok() && end.parse::<u16>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> ValidationContext {
        ValidationContext {
            known_programmes: ["CSE".to_string()].into_iter().collect(),
            configured_years: ["2026-27".to_string()].into_iter().collect(),
            institution_apaar_ids: HashSet::new(),
            global_active_apaar_ids: HashSet::new(),
            in_batch_duplicates: HashMap::new(),
            admission_period_start: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            admission_period_end: NaiveDate::from_ymd_opt(2026, 9, 30).unwrap(),
            apaar_pattern: Regex::new("^[A-Z0-9-]{8,20}$").unwrap(),
            rule_version: "demo-v1".to_string(),
        }
    }

    fn row() -> CandidateRow {
        CandidateRow {
            row_number: 2,
            apaar_id: "APAAR-2026-0001".into(),
            institution_student_id: "INST-001".into(),
            full_name: Some("Asha Rao".into()),
            programme_code: "CSE".into(),
            academic_year: "2026-27".into(),
            admission_date: "2026-07-01".into(),
            seat_category: "REGULAR".into(),
            admission_status: "ACTIVE".into(),
        }
    }

    #[test]
    fn clean_row_is_accepted() {
        let findings = validate_row(&row(), &ctx());
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
        assert_eq!(status_for(&findings), ValidationStatus::Accepted);
    }

    #[test]
    fn missing_apaar_id_is_an_error() {
        let mut r = row();
        r.apaar_id = "   ".into();
        let findings = validate_row(&r, &ctx());
        assert_eq!(status_for(&findings), ValidationStatus::Rejected);
        assert!(findings
            .iter()
            .any(|f| f.rule_code == RuleCode::ApaarIdMissing));
    }

    #[test]
    fn format_check_is_reported_as_format_only() {
        let mut r = row();
        r.apaar_id = "bad!".into();
        let findings = validate_row(&r, &ctx());
        let f = findings
            .iter()
            .find(|f| f.rule_code == RuleCode::ApaarIdFormatInvalid)
            .expect("format finding");
        assert!(f.message.contains("not identity verification"));
    }

    #[test]
    fn duplicate_in_batch_is_a_review_flag_not_a_rejection() {
        let mut c = ctx();
        c.in_batch_duplicates
            .insert("APAAR-2026-0001".to_string(), vec![2, 5]);
        let findings = validate_row(&row(), &c);
        assert_eq!(status_for(&findings), ValidationStatus::NeedsReview);
        assert!(findings
            .iter()
            .any(|f| f.rule_code == RuleCode::DuplicateApaarInBatch));
    }

    #[test]
    fn cross_institution_conflict_is_a_review_flag() {
        let mut c = ctx();
        c.global_active_apaar_ids.insert("APAAR-2026-0001".to_string());
        let findings = validate_row(&row(), &c);
        assert_eq!(status_for(&findings), ValidationStatus::NeedsReview);
        let f = findings
            .iter()
            .find(|f| f.rule_code == RuleCode::CrossInstitutionActiveConflict)
            .unwrap();
        assert!(f.message.contains("not a fraud verdict"));
    }

    #[test]
    fn missing_name_is_only_a_warning() {
        let mut r = row();
        r.full_name = None;
        let findings = validate_row(&r, &ctx());
        assert_eq!(status_for(&findings), ValidationStatus::AcceptedWithWarning);
    }

    #[test]
    fn unknown_programme_is_rejected() {
        let mut r = row();
        r.programme_code = "UNKNOWN".into();
        let findings = validate_row(&r, &ctx());
        assert_eq!(status_for(&findings), ValidationStatus::Rejected);
    }
}
