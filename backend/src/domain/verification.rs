use serde::Serialize;

/// Purpose-limited verification (P1).
///
/// The verifier never receives the full record. The outcome carries only what
/// the declared purpose allows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum VerificationOutcome {
    /// A matching record exists and passed the platform's own checks.
    VerifiedAgainstPlatformRecords,
    /// A record exists but has unresolved issues.
    NeedsReview,
    /// No record matched.
    NotFound,
}

impl VerificationOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationOutcome::VerifiedAgainstPlatformRecords => "VERIFIED_AGAINST_PLATFORM_RECORDS",
            VerificationOutcome::NeedsReview => "NEEDS_REVIEW",
            VerificationOutcome::NotFound => "NOT_FOUND",
        }
    }
}

/// A verification response.
///
/// `source` always states where the answer came from. A simulated adapter must
/// say so - see BUILD_PLAN.md A-02.
#[derive(Clone, Debug, Serialize)]
pub struct VerificationResult {
    pub outcome: VerificationOutcome,
    /// The fields actually disclosed. Never more than the purpose allows.
    pub fields_disclosed: Vec<DisclosedField>,
    pub source: String,
    pub verified_at: chrono::DateTime<chrono::Utc>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DisclosedField {
    pub name: String,
    pub value: String,
}

/// Field disclosure policy per verification purpose.
#[derive(Clone, Copy, Debug)]
pub struct PurposePolicy {
    pub purpose: &'static str,
    pub allowed_fields: &'static [&'static str],
}

/// The only purposes the prototype recognises.
pub const PURPOSES: &[PurposePolicy] = &[
    PurposePolicy {
        purpose: "EMPLOYMENT_CHECK",
        allowed_fields: &["record_status", "academic_year", "programme_code"],
    },
    PurposePolicy {
        purpose: "ADMISSIONS_TRANSFER",
        allowed_fields: &["record_status", "academic_year", "programme_code", "admission_status"],
    },
    PurposePolicy {
        purpose: "AUDIT_CHECK",
        allowed_fields: &["record_status", "academic_year", "programme_code", "admission_status"],
    },
];

pub fn policy_for(purpose: &str) -> Option<&'static PurposePolicy> {
    PURPOSES.iter().find(|p| p.purpose == purpose)
}

/// Standard limitation text attached to every verification response.
pub fn default_limitations(source_is_simulated: bool) -> Vec<String> {
    let mut out = vec![
        "This result reflects the platform's own records only.".to_string(),
        "It is not a certification by APAAR, AICTE or any government authority.".to_string(),
    ];
    if source_is_simulated {
        out.push(
            "No live APAAR source was consulted: the external verification adapter is SIMULATED."
                .to_string(),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_purpose_has_no_policy() {
        assert!(policy_for("NOT_A_PURPOSE").is_none());
    }

    #[test]
    fn simulated_source_is_always_disclosed() {
        let limits = default_limitations(true);
        assert!(limits.iter().any(|l| l.contains("SIMULATED")));
        assert!(!default_limitations(false).iter().any(|l| l.contains("SIMULATED")));
    }
}
