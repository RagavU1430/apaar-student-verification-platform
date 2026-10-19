# Data dictionary

Scope: the entities the prototype persists. Personal data is deliberately minimal.

## institutions

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key. |
| `official_code` | text | Unique. In the prototype these are `DEMO-*` fixtures, not real AICTE codes. |
| `name` | text | Display name. |
| `status` | text | `ACTIVE`. |
| `created_at` | timestamptz | UTC. |

## programmes

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key. |
| `institution_id` | uuid | FK to institutions. |
| `code` | text | Unique per institution. |
| `name` | text | Programme name. |
| `level` | text | `UG` in the demo data. |
| `status` | text | `ACTIVE`. |

## approved_intakes

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key. |
| `programme_id` | uuid | FK to programmes. |
| `academic_year` | text | `2026-27`. |
| `regular_intake` | int | Regular approved capacity, `>= 0`. |
| `supernumerary_capacity` | int | Separately configured supernumerary capacity, `>= 0`. |
| `source_reference` | text | Where the capacity came from. `DEMO-SEED-v1` in the prototype. |
| `rule_version` | text | Counting rule version used with this capacity. |
| `effective_version` | int | Unique per programme/year, so capacity changes are traceable. |

## intake_rules

Versioned counting policy per seat category.

| Field | Type | Notes |
| --- | --- | --- |
| `rule_version` | text | e.g. `demo-v1`. |
| `seat_category` | text | `REGULAR`, `SUPERNUMERARY`, `NRI`, `OCI`, `FOREIGN_NATIONAL`. |
| `counts_toward_regular` | bool | Whether the category consumes the regular capacity. |
| `uses_supernumerary_pool` | bool | Whether the category consumes the supernumerary pool. |
| `notes` | text | Human explanation. |

## users

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key. |
| `login_identifier` | text | Unique. |
| `display_name` | text | Display name. |
| `role` | text | `INSTITUTION_ADMIN`, `INSTITUTION_DATA_OPERATOR`, `AICTE_ADMIN`, `AUTHORIZED_VERIFIER`, `AUDITOR`. |
| `institution_id` | uuid | Null for global roles. |
| `password_hash` | text | argon2id hash. Plaintext passwords are never stored. |
| `status` | text | `ACTIVE`. |

## students

Minimal central identity record. **No** date of birth, document number, photograph or
biometric data is stored.

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key. |
| `apaar_id` | text | Unique. Normalised: trimmed and upper-cased. Treated as opaque. |
| `apaar_id_last4` | text | Last four characters, used for masked display only. |

## institution_students

The institution's own view of a student.

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key. |
| `student_id` | uuid | FK to students. |
| `institution_id` | uuid | FK to institutions. |
| `institution_student_id` | text | The institution's local student id. Unique per institution. |
| `full_name` | text | Optional. |

## admissions

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key. |
| `student_identity_id` | uuid | FK to institution_students. |
| `institution_id` | uuid | FK to institutions. |
| `programme_id` | uuid | FK to programmes. |
| `academic_year` | text | `2026-27`. |
| `admission_date` | date | `YYYY-MM-DD`. |
| `status` | text | `ACTIVE`, `ENROLLED`, `COMPLETED`, `WITHDRAWN`, `TRANSFERRED`, `ON_LEAVE`. |
| `seat_category` | text | See `intake_rules`. |
| `source_batch_id` | uuid | FK to submission_batches. |
| `created_at`, `updated_at` | timestamptz | UTC. |

**Constraint:** at most one `ACTIVE`/`ENROLLED` admission per student per row
(`admissions_one_active_per_student`). Historical statuses are unlimited, so a legitimate
transfer or completion history is preserved rather than flattened.

## submission_batches

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key. |
| `institution_id` | uuid | FK to institutions. Taken from the token, never from the client. |
| `submitted_by` | uuid | FK to users. |
| `file_name` | text | Original file name. |
| `file_hash` | text | SHA-256 of the upload, used for fingerprinting. |
| `schema_version` | text | `csv-v1`. |
| `status` | text | `VALIDATED` then `COMMITTED`. |
| `row_count`, `accepted_count`, `warning_count`, `duplicate_count`, `rejected_count`, `needs_review_count` | int | Aggregate counts. |
| `idempotency_key` | text | Unique. A repeated key returns the original batch. |
| `submitted_at`, `committed_at` | timestamptz | UTC. |

The raw uploaded file is **not** retained.

## staging_rows

Rows that passed validation but have not been committed yet.

| Field | Type | Notes |
| --- | --- | --- |
| `batch_id` | uuid | FK to submission_batches, cascades on delete. |
| `row_number` | int | 1-based row in the source file, header excluded. |
| `apaar_id`, `institution_student_id`, `full_name` | text | Same meaning as above. |
| `programme_id` | uuid | FK to programmes. |
| `academic_year` | text | |
| `admission_date` | date | |
| `admission_status`, `seat_category` | text | |
| `row_status` | text | `ACCEPTED` or `ACCEPTED_WITH_WARNING`. |

Rows are deleted once the batch is committed.

## validation_issues

| Field | Type | Notes |
| --- | --- | --- |
| `id` | uuid | Primary key. |
| `batch_id` | uuid | FK to submission_batches. |
| `institution_id` | uuid | FK to institutions. |
| `row_reference` | text | e.g. `row 4`, or `CSE-01 / 2026-27` for intake exceptions. |
| `rule_code` | text | Stable reason code. See `docs/openapi.yaml`. |
| `severity` | text | `ERROR`, `REVIEW`, `DUPLICATE`, `WARNING`. |
| `message` | text | Plain-language explanation. |
| `field` | text | The offending column, when applicable. |
| `evidence` | text | The concrete values that triggered the rule. No unnecessary personal data. |
| `rule_version` | text | Which rule version produced the finding. |
| `status` | text | `OPEN`, `RESOLVED`, `DISMISSED`. |
| `resolution_note` | text | Required when an issue is closed. |
| `resolved_by`, `resolved_at` | uuid, timestamptz | Reviewer and time. |

Findings are never deleted when they are closed: the status and the reason are appended to the
history.

## verification_requests

| Field | Type | Notes |
| --- | --- | --- |
| `requester_id` | uuid | FK to users. |
| `purpose` | text | `EMPLOYMENT_CHECK`, `ADMISSIONS_TRANSFER`, `AUDIT_CHECK`. |
| `record_ref` | text | What was looked up. |
| `fields_disclosed` | jsonb | The fields actually returned to the verifier. |
| `outcome` | text | `VERIFIED_AGAINST_PLATFORM_RECORDS`, `NEEDS_REVIEW`, `NOT_FOUND`. |
| `source` | text | `simulated-apaar-adapter` in the prototype. |
| `simulated` | bool | Always true in the prototype. |

## audit_events

Append-only event log.

| Field | Type | Notes |
| --- | --- | --- |
| `actor_id` | uuid | Who acted. Null for system actions. |
| `actor_role` | text | Role at the time of the action. |
| `action` | text | e.g. `batch.create`, `batch.commit`, `issue.resolve`, `verification.request`. |
| `entity_type`, `entity_id` | text, uuid | What was acted on. |
| `institution_id` | uuid | Scope of the action. |
| `outcome` | text | `SUCCESS` or `DENIED`. Denied cross-institution attempts are logged too. |
| `correlation_id` | text | Ties related events together. |
| `summary` | text | Short, safe description. Never contains APAAR ids or tokens. |
| `created_at` | timestamptz | UTC. |
