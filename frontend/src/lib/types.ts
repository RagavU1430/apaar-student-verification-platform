// ==============================================================================
// AICTE APAAR Platform - Domain & Schema Types
// ==============================================================================

export type UserRole =
  | 'AICTE_ADMIN'
  | 'INSTITUTION_ADMIN'
  | 'INSTITUTION_DATA_OPERATOR'
  | 'AUTHORIZED_VERIFIER'
  | 'AUDITOR';

export interface UserProfile {
  id: string;
  login_identifier: string;
  display_name: string;
  role: UserRole;
  institution_id: string | null;
  institution_name: string | null;
  permissions: string[];
}

export interface Institution {
  id: string;
  official_code: string;
  name: string;
  status: 'ACTIVE' | 'SUSPENDED' | 'INACTIVE';
  created_at: string;
}

export interface Programme {
  id: string;
  institution_id: string;
  code: string;
  name: string;
  level: string;
  status: string;
  created_at: string;
}

export interface ApprovedIntake {
  id: string;
  programme_id: string;
  academic_year: string;
  regular_intake: number;
  supernumerary_capacity: number;
  source_reference: string;
  rule_version: string;
  effective_version: number;
}

export interface IntakeRule {
  id: string;
  rule_version: string;
  seat_category: string;
  counts_toward_regular: boolean;
  uses_supernumerary_pool: boolean;
  notes?: string;
}

export type Severity = 'ERROR' | 'WARNING' | 'DUPLICATE' | 'REVIEW';
export type ValidationStatus = 'ACCEPTED' | 'ACCEPTED_WITH_WARNING' | 'NEEDS_REVIEW' | 'REJECTED';

export type RuleCode =
  | 'FILE_UNSUPPORTED_TYPE'
  | 'FILE_TOO_LARGE'
  | 'MISSING_COLUMN'
  | 'ROW_LIMIT_EXCEEDED'
  | 'APAAR_ID_MISSING'
  | 'APAAR_ID_FORMAT_INVALID'
  | 'STUDENT_ID_MISSING'
  | 'FULL_NAME_MISSING'
  | 'PROGRAMME_CODE_MISSING'
  | 'PROGRAMME_UNKNOWN'
  | 'ACADEMIC_YEAR_MISSING'
  | 'ACADEMIC_YEAR_INVALID'
  | 'ACADEMIC_YEAR_NOT_CONFIGURED'
  | 'ADMISSION_DATE_MISSING'
  | 'ADMISSION_DATE_INVALID'
  | 'ADMISSION_DATE_OUTSIDE_PERIOD'
  | 'SEAT_CATEGORY_UNKNOWN'
  | 'ADMISSION_STATUS_INVALID'
  | 'DUPLICATE_APAAR_IN_BATCH'
  | 'CROSS_INSTITUTION_ACTIVE_CONFLICT'
  | 'INTAKE_NOT_CONFIGURED'
  | 'INTAKE_CAPACITY_EXCEEDED'
  | 'DUPLICATE_ACTIVE_ADMISSION';

export interface ValidationFinding {
  rule_code: RuleCode;
  severity: Severity;
  message: string;
  field?: string | null;
  evidence?: string | null;
  rule_version: string;
}

export interface CandidateRow {
  row_number: number;
  apaar_id: string;
  institution_student_id: string;
  full_name?: string | null;
  programme_code: string;
  academic_year: string;
  admission_date: string;
  seat_category: string;
  admission_status: string;
}

export interface RowOutcome {
  row_number: number;
  status: ValidationStatus;
  findings: ValidationFinding[];
  candidate?: CandidateRow;
  programme_id?: string;
}

export interface BatchCounts {
  rows: number;
  accepted: number;
  accepted_with_warning: number;
  needs_review: number;
  duplicates: number;
  rejected: number;
}

export interface SubmissionBatch {
  id: string;
  institution_id: string;
  institution_name?: string;
  submitted_by?: string;
  file_name: string;
  file_hash: string;
  schema_version: string;
  status: 'VALIDATED' | 'COMMITTED' | 'REJECTED' | 'FAILED';
  row_count: number;
  accepted_count: number;
  warning_count: number;
  duplicate_count: number;
  rejected_count: number;
  needs_review_count: number;
  submitted_at: string;
  committed_at: string | null;
  rule_version: string;
  external_verification: string;
}

export interface ValidationIssue {
  id: string;
  batch_id: string;
  institution_id: string;
  row_reference: string;
  rule_code: RuleCode;
  severity: Severity;
  message: string;
  field: string | null;
  evidence: string | null;
  rule_version: string;
  status: 'OPEN' | 'RESOLVED' | 'DISMISSED';
  resolution_note: string | null;
  resolved_by?: string | null;
  resolved_at: string | null;
  created_at: string;
}

export interface Admission {
  id: string;
  institution_id: string;
  programme_code: string;
  programme_name?: string;
  academic_year: string;
  admission_date: string;
  status: string;
  seat_category: string;
  institution_student_id: string;
  full_name: string | null;
  apaar_id_masked: string;
}

export interface IntakeCard {
  programme_code: string;
  programme_name: string | null;
  academic_year: string;
  rule_version: string;
  regular_count: number;
  regular_intake: number;
  supernumerary_count: number;
  supernumerary_capacity: number;
  utilisation: number | null;
  over_capacity: boolean;
  source_reference: string | null;
}

export interface InstitutionDashboardData {
  institution_id: string;
  institution_name: string | null;
  rule_version: string;
  data_source: string;
  batches: Array<{
    id: string;
    file_name: string;
    status: string;
    submitted_at: string;
    rows: number;
    accepted: number;
    rejected: number;
    needs_review: number;
  }>;
  intake: IntakeCard[];
  open_issues: number;
  counts: {
    total_batches: number;
    total_rows: number;
    accepted_rows: number;
    rejected_rows: number;
    rows_needing_review: number;
    duplicate_candidates: number;
  };
}

export interface AicteDashboardData {
  rule_version: string;
  data_source: string;
  institutions: Array<{
    id: string;
    name: string;
    official_code: string;
    active_admissions: number;
  }>;
  intake: IntakeCard[];
  exception_queue: Array<{
    issue_id: string;
    institution_id: string;
    institution_name?: string;
    row_reference: string;
    rule_code: string;
    message: string;
    evidence: string | null;
    rule_version: string;
    severity: string;
  }>;
  counts: InstitutionDashboardData['counts'];
}

export interface AuditEvent {
  id: string;
  actor_id: string | null;
  actor_role: string | null;
  action: string;
  entity_type: string;
  entity_id: string | null;
  institution_id: string | null;
  outcome: string;
  summary: string | null;
  created_at: string;
}

export interface VerificationRequestPayload {
  purpose: 'ENROLMENT_STATUS' | 'DEGREE_ELIGIBILITY' | 'EMPLOYMENT_BACKGROUND';
  record_ref: string;
  institution_id?: string;
}

export interface DisclosedField {
  name: string;
  value: string;
  justification: string;
}

export interface VerificationResponse {
  outcome: 'VERIFIED' | 'NEEDS_REVIEW' | 'NOT_FOUND';
  fields_disclosed: DisclosedField[];
  source: string;
  verified_at: string;
  limitations: string[];
  simulated: boolean;
}
