use serde::{Deserialize, Serialize};
use std::fmt;

/// Lifecycle state of an admission.
///
/// `Active`/`Enrolled` are the only states counted as active enrolment under the
/// demo counting rule. Historical states are retained and never deleted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionStatus {
    Active,
    Enrolled,
    Completed,
    Withdrawn,
    Transferred,
    OnLeave,
}

impl AdmissionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdmissionStatus::Active => "ACTIVE",
            AdmissionStatus::Enrolled => "ENROLLED",
            AdmissionStatus::Completed => "COMPLETED",
            AdmissionStatus::Withdrawn => "WITHDRAWN",
            AdmissionStatus::Transferred => "TRANSFERRED",
            AdmissionStatus::OnLeave => "ON_LEAVE",
        }
    }

    /// Counts towards active enrolment under the demo counting rule.
    pub fn counts_as_active(&self) -> bool {
        matches!(self, AdmissionStatus::Active | AdmissionStatus::Enrolled)
    }

    /// Parses a status token from the CSV template (case-insensitive).
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "ACTIVE" => Some(AdmissionStatus::Active),
            "ENROLLED" => Some(AdmissionStatus::Enrolled),
            "COMPLETED" => Some(AdmissionStatus::Completed),
            "WITHDRAWN" => Some(AdmissionStatus::Withdrawn),
            "TRANSFERRED" => Some(AdmissionStatus::Transferred),
            "ON_LEAVE" => Some(AdmissionStatus::OnLeave),
            _ => None,
        }
    }

    pub fn all() -> &'static [AdmissionStatus] {
        &[
            AdmissionStatus::Active,
            AdmissionStatus::Enrolled,
            AdmissionStatus::Completed,
            AdmissionStatus::Withdrawn,
            AdmissionStatus::Transferred,
            AdmissionStatus::OnLeave,
        ]
    }
}

impl fmt::Display for AdmissionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Seat category. The prototype treats categories as configured data, never as
/// hard-coded policy. See BUILD_PLAN.md A-04.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeatCategory {
    Regular,
    Supernumerary,
    Nri,
    Oci,
    ForeignNational,
}

impl SeatCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SeatCategory::Regular => "REGULAR",
            SeatCategory::Supernumerary => "SUPERNUMERARY",
            SeatCategory::Nri => "NRI",
            SeatCategory::Oci => "OCI",
            SeatCategory::ForeignNational => "FOREIGN_NATIONAL",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "REGULAR" => Some(SeatCategory::Regular),
            "SUPERNUMERARY" => Some(SeatCategory::Supernumerary),
            "NRI" => Some(SeatCategory::Nri),
            "OCI" => Some(SeatCategory::Oci),
            "FOREIGN_NATIONAL" => Some(SeatCategory::ForeignNational),
            _ => None,
        }
    }

    pub fn all() -> &'static [SeatCategory] {
        &[
            SeatCategory::Regular,
            SeatCategory::Supernumerary,
            SeatCategory::Nri,
            SeatCategory::Oci,
            SeatCategory::ForeignNational,
        ]
    }
}

impl fmt::Display for SeatCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A validated admission record ready to be persisted.
#[derive(Clone, Debug)]
pub struct AdmissionRecord {
    pub apaar_id: String,
    pub apaar_id_last4: String,
    pub institution_student_id: String,
    pub full_name: Option<String>,
    pub programme_code: String,
    pub academic_year: String,
    pub admission_date: chrono::NaiveDate,
    pub status: AdmissionStatus,
    pub seat_category: SeatCategory,
}

/// Normalises an APAAR id for storage and comparison.
///
/// The value is treated as opaque: only whitespace/case normalisation is
/// applied. No format assumption is baked in here.
pub fn normalise_apaar_id(raw: &str) -> String {
    raw.trim().to_ascii_uppercase()
}

/// Masked form used in list views and logs. Never used for lookups.
pub fn mask_apaar_id(normalised: &str) -> String {
    let chars: Vec<char> = normalised.chars().collect();
    if chars.len() <= 4 {
        "*".repeat(chars.len())
    } else {
        let tail: String = chars[chars.len() - 4..].iter().collect();
        format!("{}{}", "*".repeat(chars.len() - 4), tail)
    }
}
