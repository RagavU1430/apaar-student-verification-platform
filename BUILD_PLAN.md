# BUILD PLAN — AICTE APAAR Student Data & Verification Platform

**Status:** Locked for prototype build
**Source spec:** `PRD.md` (v1.0, 19 Oct 2026)
**Window:** 2–7 day prototype, 1–2 developers
**Date locked:** 20 Oct 2026

This document resolves every decision the PRD deferred. It is the authoritative build
contract for the prototype. Where a decision could not be made from available evidence,
it is recorded in the **Assumptions Register** with an explicit label, the demo impact,
and the trigger that forces a revisit. Nothing here is silently decided.

---

## 1. What we are building (one paragraph)

A **Rust modular monolith** (Axum + Tokio + SQLx) backed by **managed PostgreSQL**, with a
**React + TypeScript + Vite** frontend, that lets an authorised institution upload a CSV of
student admissions, get **record-level validation results** (structural, field-level,
uniqueness, referential, capacity, consistency), resolve flagged issues with an auditable
review trail, and lets an AICTE administrator see cross-institution enrolment and intake
exceptions. All intake counting and APAAR checks are **deterministic, versioned,
configurable rules** — not AI, and not live government verification.

---

## 2. Scope lock

### 2.1 In scope (P0 — must build)

| # | Capability | Source |
| --- | --- | --- |
| P0-1 | Auth: sign-in, server-validated session/token | FR-01 |
| P0-2 | Role-based access + server-side institution scoping | FR-02, §4 |
| P0-3 | Institution / programme / academic-year / approved-intake configuration | FR-03 |
| P0-4 | CSV bulk upload against a versioned template (file-level checks, limits) | FR-04, §5.1 |
| P0-5 | Row-level validation report with reason codes | FR-05, §7.1 |
| P0-6 | APAAR field validation (presence + format only, levels 1–2) | FR-06, §7.3 |
| P0-7 | Duplicate detection: in-batch + cross-institution | FR-07, §7.4 |
| P0-8 | Admission vs approved-intake comparison, policy-configurable | FR-08, §7.2 |
| P0-9 | Central student & admission registry with history | FR-09 |
| P0-10 | Auditable trail for every create/update/review action | FR-12 |
| P0-11 | Institution dashboard | FR-13 |
| P0-12 | AICTE dashboard + exception review queue | FR-13, FR-11 |
| P0-13 | Record correction / review workflow (no history destruction) | FR-10 |

### 2.2 Deferred to P1 (build only if P0 is green)

| # | Capability | Source |
| --- | --- | --- |
| P1-1 | Purpose-limited verification endpoint + disclosure policy | FR-14 |
| P1-2 | Report export (CSV) | FR-15 |
| P1-3 | Documented, versioned REST contract (OpenAPI) | FR-16 |

### 2.3 Explicitly out of scope for this prototype

FR-17 (QR verification), FR-18 (anomaly score), FR-19 (certificate display), XLSX support,
microservices, message brokers, background job workers, AI/ML scoring, live APAAR calls,
production deployment, formal security certification. *(PRD §3.2 non-goals, §16.1.)*

> **Correction applied:** PRD §16.1 says "build only two roles initially". We lock **four
> roles in the policy layer from day one** (below) because retrofitting a role enum into
> persisted rows and audit events later is a data migration. Only three are exposed in the
> demo UI. See §3.2.

---

## 3. Architecture lock

### 3.1 Stack

| Concern | Decision | Note |
| --- | --- | --- |
| Language | **Rust, stable, `x86_64-pc-windows-gnu`** | Host has no MSVC toolchain; the GNU host target ships its own MinGW linker, needs no admin |
| HTTP | axum 0.8 + tokio 1 + tower / tower-http | Typed extractors, thin handlers, CORS + trace + body-limit layers |
| DB access | sqlx 0.8 (runtime queries, `postgres` feature) | `query!` macros are **not** used — no compile-time DB needed for `cargo check` |
| Database | **PostgreSQL (managed, Supabase)** | TLS required; connection string from `DATABASE_URL` |
| Migrations | `sqlx migrate!` from `backend/migrations/` | Repeatable, version-controlled, applied on boot |
| Serialization | serde + serde_json | Typed request/response structs at the API boundary |
| Validation | Hand-written pure domain functions | Business rules must be unit-testable without a DB (PRD §21) |
| Auth (prototype) | argon2id password hash + HS256 JWT access token | **Not** production identity management (see A-07) |
| CSV | `csv` crate, UTF-8, strict headers | XLSX deferred (§16.1) |
| Frontend | React 18 + TypeScript + Vite | Consumes the REST API only |
| Observability | tracing + tracing-subscriber, request IDs | No student data in logs |

### 3.2 Roles (locked enum, persisted from day one)

| Role | Institution scope | Prototype UI |
| --- | --- | --- |
| `INSTITUTION_ADMIN` | Own institution only | Yes — also manages programmes & intake |
| `INSTITUTION_DATA_OPERATOR` | Own institution only | Yes — upload & fix issues |
| `AICTE_ADMIN` | All institutions (global) | Yes — dashboards, review queue, rule config |
| `AUTHORIZED_VERIFIER` | Purpose-limited, no bulk search | P1, demo extension |
| `AUDITOR` | Read-only, assigned scope | Deferred (P1); modelled as read-only AICTE variant |

Rules that are **non-negotiable and enforced server-side**:

- Institution scope is derived from the **authenticated server-side identity only**.
  A client-supplied `institution_id` in a body or query is **never** trusted.
- Deny by default. Every route declares its required role + scope.
- Role checks live in `auth/policy.rs` and are applied by middleware/extractor, not in React.
- Cross-institution access attempts are **rejected and logged** as audit events.

### 3.3 Module layout (PRD §20, completed and corrected)

```
.
├── PRD.md                     # repaired spec
├── BUILD_PLAN.md             # this file
├── README.md
├── docker-compose.yml        # optional local Postgres for other devs
├── .env.example
├── backend/
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   ├── migrations/           # sqlx migrations, numbered
│   ├── sample-data/          # synthetic CSVs for the demo
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── state.rs
│   │   ├── error.rs
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── institutions.rs
│   │   │   ├── batches.rs
│   │   │   ├── admissions.rs
│   │   │   ├── issues.rs
│   │   │   ├── dashboards.rs
│   │   │   ├── verifications.rs
│   │   │   └── reports.rs
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── admission.rs
│   │   │   ├── intake.rs
│   │   │   ├── validation.rs
│   │   │   └── verification.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── ingestion.rs
│   │   │   ├── validation.rs
│   │   │   ├── duplicate_detection.rs
│   │   │   ├── intake_monitoring.rs
│   │   │   └── audit.rs
│   │   ├── repositories/
│   │   │   ├── mod.rs
│   │   │   ├── institutions.rs
│   │   │   ├── students.rs
│   │   │   ├── admissions.rs
│   │   │   ├── batches.rs
│   │   │   └── issues.rs
│   │   ├── auth/
│   │   │   ├── mod.rs
│   │   │   └── policy.rs
│   │   └── integrations/
│   │       ├── mod.rs
│   │       └── academic_identity.rs
│   └── tests/
│       ├── api_authz.rs
│       └── api_batches.rs
├── frontend/
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── api/
│       ├── components/
│       ├── pages/
│       └── types/
└── docs/
    ├── openapi.yaml
    ├── data-dictionary.md
    ├── demo-script.md
    └── assumptions.md
```

**Layering rule (enforced by review):** `routes` → `services` → `repositories` → DB.
`domain` is **pure** (no DB, no clock, no I/O) so rules are unit-testable. No module may
call another module's repository directly.

---

## 4. API lock (PRD §10, with auth/scope filled in)

Base path `/api/v1`. All responses carry `X-Request-Id`. All errors use the shared error
envelope in §5.

| Method & route | Purpose | Required role | Scope |
| --- | --- | --- | --- |
| `POST /api/v1/auth/login` | Sign in, obtain access token | public (rate-limited) | — |
| `GET /api/v1/me` | Current identity, role, institution scope | any authenticated | self |
| `GET /api/v1/institutions` | Institutions visible to the caller | any authenticated | role-filtered |
| `GET /api/v1/institutions/{id}/intakes` | Approved intake + rule version | any authenticated | scoped to caller's institutions |
| `POST /api/v1/batches` | Create batch + upload CSV | `INSTITUTION_ADMIN`, `INSTITUTION_DATA_OPERATOR` | own institution |
| `GET /api/v1/batches/{id}` | Batch status + aggregate counts | institution roles / AICTE | owner or regulator |
| `GET /api/v1/batches/{id}/issues` | Row-level validation issues | institution roles / AICTE | owner or regulator |
| `POST /api/v1/batches/{id}/commit` | Commit eligible records (idempotent) | `INSTITUTION_ADMIN` | own institution |
| `GET /api/v1/admissions` | Search/filter admissions | institution roles / AICTE | server-filtered by scope |
| `GET /api/v1/dashboard/institution` | Institution metrics | institution roles | own institution |
| `GET /api/v1/dashboard/aicte` | Cross-institution aggregates | `AICTE_ADMIN` | global |
| `POST /api/v1/issues/{id}/resolve` | Resolve/dismiss an issue with reason | assigned reviewer / `AICTE_ADMIN` | scoped |
| `POST /api/v1/verifications` | Purpose-limited verification | `AUTHORIZED_VERIFIER` | purpose-limited | 
| `GET /api/v1/reports/enrolment` | Filtered enrolment report / export | scoped role | scoped |
| `GET /health/live` | Liveness | public | — |
| `GET /health/ready` | Readiness (DB ping) | public, no details | — |

**Contract rules:** page size default 50 / max 200; export volume capped; max upload 5 MB
and 5 000 rows; validation errors are `422`; upload-too-large `413`; all queries validated
and parameterised; login / verification / export endpoints rate-limited.

---

## 5. Error contract

Every failure returns:

```json
{
  "error": {
    "code": "INTAKE_CAPACITY_EXCEEDED",
    "message": "Safe, human-readable explanation.",
    "details": { "field": "apaar_id" },
    "request_id": "..."
  }
}
```

- `400` invalid input · `401` unauthenticated · `403` unauthorised · `404` not found /
  not-disclosed · `409` conflict · `413` payload too large · `422` row-level semantic
  error · `429` rate limited · `500` unexpected.
- Never leak SQL errors, stack traces, secrets, or internal infrastructure detail.
- File-level failures and row-level failures are **distinct** shapes.

---

## 6. Data model lock (PRD §8, expanded)

Tables (all with `id`, `created_at`; timestamps stored UTC):

| Table | Purpose | Key constraints |
| --- | --- | --- |
| `institutions` | Institution master | `official_code` unique |
| `programmes` | Institution-scoped programme catalogue | unique `(institution_id, code)` |
| `approved_intakes` | Capacity + counting rules per programme/year | unique `(programme_id, academic_year, effective_version)`; `source_reference`, `rule_version` |
| `intake_rules` | Seat-category counting rules (versioned) | unique `(rule_version, seat_category)` |
| `users` | Login identity | `login_identifier` unique; argon2id hash; `role`; `institution_id` nullable (null ⇒ global) |
| `students` | Minimal identity record | `apaar_id` unique **per normalised form**; `institution_student_id` unique per institution |
| `admissions` | Admission lifecycle | FK to students/programmes; `status` enum; history preserved (no destructive update) |
| `submission_batches` | Batch + file metadata | `file_hash` + `schema_version`; idempotency via `idempotency_key` |
| `validation_issues` | Row-level findings | `rule_code`, `severity`, `status`, `resolved_by`, `resolved_at`, `evidence` |
| `verification_requests` | Purpose-limited lookups | `purpose`, `fields_disclosed`, `outcome` |
| `audit_events` | Append-only event log | `actor_id`, `action`, `entity_type`, `entity_id`, `correlation_id`; no sensitive payloads |

**Integrity rules:** FKs everywhere; unique constraints for the invariants that must hold
under concurrency; transactions for batch commit and review state changes; normalised
APAAR ID in a dedicated column and indexed; `idempotency_key` on batch creation and commit
to make retries safe; no raw CSV retained after parsing beyond the batch (file metadata
only).

---

## 7. Rules engine lock

### 7.1 Validation classes → reason codes

| Class | Reason codes | Default severity |
| --- | --- | --- |
| Structural | `FILE_UNSUPPORTED_TYPE`, `FILE_TOO_LARGE`, `MISSING_COLUMN`, `INVALID_DATE`, `ROW_LIMIT_EXCEEDED` | error (file-level = reject file) |
| Field-level | `APAAR_ID_MISSING`, `APAAR_ID_FORMAT_INVALID`, `STUDENT_ID_MISSING`, `PROGRAMME_CODE_INVALID`, `SEAT_CATEGORY_UNKNOWN` | error / needs-correction |
| Uniqueness | `DUPLICATE_APAAR_IN_BATCH`, `CROSS_INSTITUTION_ACTIVE_CONFLICT` | duplicate = candidate flag; conflict = review |
| Referential | `PROGRAMME_UNKNOWN`, `ACADEMIC_YEAR_NOT_CONFIGURED` | error |
| Capacity | `INTAKE_CAPACITY_EXCEEDED` | review exception |
| Consistency | `ADMISSION_DATE_OUTSIDE_PERIOD`, `CATEGORY_TOTALS_MISMATCH`, `STATUS_DATE_INCONSISTENT` | error / review |

Every issue carries: `rule_code`, human message, severity, row reference, evidence, and the
`rule_version` that produced it.

### 7.2 APAAR validation levels (PRD §7.3)

- **Level 1 — presence**: required, non-empty. Always available.
- **Level 2 — format**: charset/length check **only if** a format spec is confirmed
  (see A-01). In the prototype this is a configurable, clearly-labelled *demo* rule.
- **Level 3 — authoritative**: **not implemented against any live source.** Only a
  simulated adapter exists, and it self-identifies as `simulated`.

A value that passes a format check must **never** be labelled "APAAR verified" in UI or API.

### 7.3 Intake counting rule (demo, versioned)

> **DEMO RULE v1** — count admissions with status in `ACTIVE`/`ENROLLED` grouped by
> `institution + programme + academic_year + seat_category`; compare against the configured
> `regular_intake`; separately configured `supernumerary_capacity` is tracked and reported
> but never silently merged into regular seats. Excess raises `INTAKE_CAPACITY_EXCEEDED`
> with the calculation, the source records, and the rule version.

Rule text lives in config/DB with an `effective_version`, so a real policy can replace it
without code changes. Withdrawal/completion/transfer statuses are **not** counted as active
but their history is retained.

---

## 8. Assumptions Register (PRD §18 resolved)

Every row is an **explicit assumption**, not a fact. Each carries a label that must appear
in the demo and in the UI where relevant.

| ID | Question | Locked decision for the prototype | Label shown in product | Revisit trigger |
| --- | --- | --- | --- | --- |
| A-01 | Authoritative APAAR ID format & live verification | Treat APAAR ID as an **opaque** identifier. Presence check always; format check = configurable demo regex `[A-Z0-9-]{8,20}` after uppercase trim. **No** live verification. | `format-checked, not verified` | Official format spec published |
| A-02 | Authorized API for verification | Not available. Implement `AcademicIdentityVerifier` trait + `SimulatedAcademicIdentityVerifier` that always returns `source: "simulated"`. | `SIMULATED — no live APAAR source` | Approved endpoint + credentials + permitted purpose |
| A-03 | Official source of approved intake | None. Seed master data authored by us, `source_reference = "DEMO-SEED-v1"`. | `Demo seed — not an AICTE approval record` | Official sanction letter / API |
| A-04 | Counting rules for regular / supernumerary / NRI / OCI / foreign | Implement **demo rule v1** (§7.3) with per-seat-category configuration. Category semantics beyond "is counted separately" are **not** invented. | `Demo counting rule v1` | Official category rules per year |
| A-05 | Which student attributes are mandatory/optional/restricted | Minimum set only: `apaar_id`, `institution_student_id`, `full_name`, `programme_code`, `academic_year`, `admission_date`, `seat_category`, `admission_status`. **No** DOB, documents, biometrics, or national ID. | — | Data-sharing policy approval |
| A-06 | Data controller / retention | **Undecided.** Prototype stores no auto-delete policy, holds no raw files after parse, and logs every disclosure. Controller question is a **release gate**, not a code task. | `Retention policy pending` | Named controller + approved schedule |
| A-07 | Authentication / SSO | Prototype: local accounts, argon2id, HS256 JWT (1 h expiry), no refresh-token rotation, no MFA. Written as an adapter boundary so OIDC can replace it. | `Prototype auth — not OIDC` | Named IdP available |
| A-08 | Assurance level for external verifiers | Low assurance, purpose-limited: verifier receives outcome + a **whitelisted** field subset + source + timestamp; no bulk search, no export. | `Low assurance · purpose-limited` | Employer assurance requirement |
| A-09 | Volume / concurrency / uptime | 1 000-row demo target; hard caps: 5 MB file, 5 000 rows, page 50/200, 10 s processing budget, best-effort availability. | — | Production SLOs agreed |
| A-10 | Existing integrations & file formats | CSV only, UTF-8, strict header contract, versioned template. No XLSX in the prototype. | `CSV v1 template` | Institution MIS formats known |
| A-11 | Reporting formats & export permissions | CSV export only; masked identifiers by default; export audited and volume-capped. | — | Required reporting formats confirmed |
| A-12 | Hosting | Managed PostgreSQL (Supabase) with TLS for the prototype; `docker-compose.yml` provided so other devs can run local Postgres; production hosting is a gate. | `Prototype hosting` | Procurement + security review |
| A-13 | Accessibility / security review / production ops | Keyboard-operable forms, labels, text+icon status (never colour alone), structured logs without student data, `cargo fmt`/`clippy`/`test` in the build gate. | — | Formal accessibility & security review |

---

## 9. Build phases & exit criteria

| Phase | Deliverable | Exit criteria |
| --- | --- | --- |
| **P0.1** Foundations | Repo scaffold, config, error contract, migrations, health endpoints, auth (seeded users) | `cargo check` + `cargo test` green; `/health/ready` returns DB status |
| **P0.2** Ingestion & validation | CSV template, upload, batch lifecycle, row-level issues, domain rule unit tests | Seed CSV produces exactly the expected issue set; no row silently dropped |
| **P0.3** Registry & intake | Commit transaction, duplicate detection, intake computation, exception creation | Counts derived from persisted data; transaction rollback leaves zero partial commits |
| **P0.4** Authz & audit | Role/scope enforcement, negative tests, audit events on every mutation | Cross-institution access attempt returns 403 **and** writes an audit event |
| **P0.5** Frontend | Sign-in, institution dashboard, upload wizard, batch detail, AICTE dashboard, issue review | Demo path walks the §10 script end to end |
| **P0.6** Hardening & docs | README, OpenAPI, data dictionary, demo script, known limitations | `cargo fmt --check`, `cargo clippy`, `cargo test`, frontend build all clean |

---

## 10. MVP sign-off checklist (from PRD §19, made testable)

- [ ] Institution user authenticates and **cannot** access another institution's records (negative test).
- [ ] Authorised user uploads the sample CSV and receives batch status + row-level issues.
- [ ] Missing ID, duplicate candidate, inconsistent field and intake overrun are all detected.
- [ ] Dashboard values are computed from persisted data — no hardcoded numbers.
- [ ] Historical/withdrawn admissions are retained but not counted as active.
- [ ] Every committed batch and review decision has an attributable audit event.
- [ ] AICTE role sees cross-institution aggregates and a review queue.
- [ ] (P1) Verification returns only allowed fields and records the request; mock labelled simulated.
- [ ] API errors are stable and safe; no APAAR IDs, tokens or raw rows in logs.
- [ ] Tests cover core validation logic, transaction behaviour and authorisation failures.
- [ ] README covers run / migrate / seed / test / frontend.
- [ ] Known limitations and production dependencies documented.

---

## 11. Definition of "simulated" (required wording)

The demo may claim: *format validation*, *duplicate candidate detection*, *intake rule check*,
*authorised external verification adapter (simulated)*.

The demo may **not** claim: "AI verifies APAAR", "APAAR-verified students", "fraud detected",
"production-ready", "government-certified data". *(PRD §22.)*

---

## 12. Open gates before production (carried forward, not silently closed)

Live APAAR integration · official intake rules & counting formulas · data-controller and
retention policy · named IdP/SSO · production hosting & security review · reporting format
mandates · volume/concurrency/SLO targets. *(PRD §18, §23.)*
