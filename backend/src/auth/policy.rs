use crate::error::AppError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The roles the platform understands.
///
/// The enum is locked in the data model from day one (BUILD_PLAN.md 3.2) so
/// that audit rows and user records never need a migration when a role is
/// added later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// Manages the institution profile, programmes and approved intakes.
    InstitutionAdmin,
    /// Prepares batches, submits records, fixes validation issues.
    InstitutionDataOperator,
    /// Monitors institutions, reviews exceptions, configures rules, reports.
    AicteAdmin,
    /// Requests a purpose-limited verification result.
    AuthorizedVerifier,
    /// Read-only inspection of submissions and access history.
    Auditor,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::InstitutionAdmin => "INSTITUTION_ADMIN",
            Role::InstitutionDataOperator => "INSTITUTION_DATA_OPERATOR",
            Role::AicteAdmin => "AICTE_ADMIN",
            Role::AuthorizedVerifier => "AUTHORIZED_VERIFIER",
            Role::Auditor => "AUDITOR",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "INSTITUTION_ADMIN" => Some(Role::InstitutionAdmin),
            "INSTITUTION_DATA_OPERATOR" => Some(Role::InstitutionDataOperator),
            "AICTE_ADMIN" => Some(Role::AicteAdmin),
            "AUTHORIZED_VERIFIER" => Some(Role::AuthorizedVerifier),
            "AUDITOR" => Some(Role::Auditor),
            _ => None,
        }
    }

    /// Roles that are not tied to a single institution.
    pub fn is_global(&self) -> bool {
        matches!(self, Role::AicteAdmin)
    }

    /// May upload batches and create batches.
    pub fn can_upload(&self) -> bool {
        matches!(self, Role::InstitutionAdmin | Role::InstitutionDataOperator)
    }

    /// May commit a batch (final write of accepted records).
    pub fn can_commit(&self) -> bool {
        matches!(self, Role::InstitutionAdmin)
    }

    /// May read records across institutions.
    pub fn can_view_all_institutions(&self) -> bool {
        matches!(self, Role::AicteAdmin)
    }

    /// May review/resolve issues.
    pub fn can_review(&self) -> bool {
        matches!(self, Role::AicteAdmin | Role::InstitutionAdmin)
    }

    /// May request a purpose-limited verification.
    pub fn can_verify(&self) -> bool {
        matches!(self, Role::AuthorizedVerifier)
    }
}

/// The authenticated caller, as established by the server from the token.
///
/// The institution scope is taken from here and **never** from a client
/// supplied parameter (PRD section 4).
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub display_name: String,
    pub role: Role,
    /// `None` means global scope (AICTE administrator).
    pub institution_id: Option<Uuid>,
}

impl AuthenticatedUser {
    pub fn is_global(&self) -> bool {
        self.role.is_global()
    }

    /// Returns the institutions this caller may act on.
    pub fn visible_institutions(&self) -> Vec<Uuid> {
        self.institution_id.into_iter().collect()
    }

    /// Deny-by-default institution scope check.
    ///
    /// A caller from one institution may never touch another institution's
    /// data. The caller is responsible for writing the audit event when this
    /// returns an error.
    pub fn require_institution_scope(&self, institution_id: Uuid) -> Result<(), AppError> {
        if self.role.is_global() {
            return Ok(());
        }
        match self.institution_id {
            Some(own) if own == institution_id => Ok(()),
            _ => Err(AppError::forbidden(
                "You do not have access to this institution's records.",
            )),
        }
    }

    /// Restricts an arbitrary query to the caller's scope.
    pub fn scope_filter(&self) -> ScopeFilter {
        match self.institution_id {
            Some(id) if !self.role.is_global() => ScopeFilter::Institution(id),
            _ => ScopeFilter::All,
        }
    }
}

/// Which institutions a query may touch.
#[derive(Clone, Copy, Debug)]
pub enum ScopeFilter {
    All,
    Institution(Uuid),
}

impl ScopeFilter {
    pub fn matches(&self, institution_id: Uuid) -> bool {
        match self {
            ScopeFilter::All => true,
            ScopeFilter::Institution(id) => *id == institution_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(role: Role, institution: Option<Uuid>) -> AuthenticatedUser {
        AuthenticatedUser {
            id: Uuid::nil(),
            display_name: "test".into(),
            role,
            institution_id: institution,
        }
    }

    #[test]
    fn institution_user_cannot_reach_another_institution() {
        let u = user(Role::InstitutionDataOperator, Some(Uuid::new_v4()));
        assert!(u.require_institution_scope(Uuid::new_v4()).is_err());
    }

    #[test]
    fn institution_user_can_reach_own_institution() {
        let own = Uuid::new_v4();
        let u = user(Role::InstitutionAdmin, Some(own));
        assert!(u.require_institution_scope(own).is_ok());
    }

    #[test]
    fn aicte_admin_has_global_scope() {
        let u = user(Role::AicteAdmin, None);
        assert!(u.require_institution_scope(Uuid::new_v4()).is_ok());
        assert!(matches!(u.scope_filter(), ScopeFilter::All));
    }

    #[test]
    fn verifier_is_not_an_institution_operator() {
        assert!(!Role::AuthorizedVerifier.can_upload());
        assert!(Role::AuthorizedVerifier.can_verify());
    }
}
