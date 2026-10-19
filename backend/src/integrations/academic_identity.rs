use serde::Serialize;

/// Outcome of an external identity check.
///
/// The `source` field is always surfaced to the user so that a simulated
/// result can never be mistaken for an authoritative one.
#[derive(Clone, Debug, Serialize)]
pub struct ExternalVerification {
    pub matched: bool,
    pub source: String,
    pub simulated: bool,
    pub notes: Vec<String>,
}

/// Boundary for an authorised external APAAR identity check.
///
/// There is **no** live implementation. The production adapter may only be
/// written once the official endpoint, authentication, permitted fields, error
/// model and rate limits are confirmed (BUILD_PLAN.md A-02).
pub trait AcademicIdentityVerifier {
    fn verify(&self, apaar_id: &str) -> ExternalVerification;
}

/// Development adapter. It performs no network call and always self-identifies
/// as simulated.
#[derive(Clone, Copy, Debug, Default)]
pub struct SimulatedAcademicIdentityVerifier;

impl AcademicIdentityVerifier for SimulatedAcademicIdentityVerifier {
    fn verify(&self, _apaar_id: &str) -> ExternalVerification {
        ExternalVerification {
            matched: false,
            source: "simulated-apaar-adapter".to_string(),
            simulated: true,
            notes: vec![
                "No live APAAR source was consulted.".to_string(),
                "This adapter exists to prove the integration boundary, not to verify identity."
                    .to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated_adapter_always_declares_itself() {
        let v = SimulatedAcademicIdentityVerifier.verify("APAAR-2026-0001");
        assert!(v.simulated);
        assert_eq!(v.source, "simulated-apaar-adapter");
        assert!(!v.matched);
    }
}
