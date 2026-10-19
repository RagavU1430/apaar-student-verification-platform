pub mod admission;
pub mod intake;
pub mod validation;
pub mod verification;

pub use admission::{AdmissionRecord, AdmissionStatus, SeatCategory};
pub use intake::{IntakeUsage, SeatRule};
pub use validation::{RuleCode, Severity, ValidationFinding, ValidationStatus};
pub use verification::{VerificationOutcome, VerificationResult};
