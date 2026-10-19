# APAAR Platform Migration Plan: Rust/Axum to React + Supabase

## 1. Executive Summary

This document outlines the architectural migration plan for the **APAAR-Based Student Data Management and Verification Platform for AICTE-Approved Institutions**.

The platform is being transitioned from a dual-backend prototype (Node.js script prototype + Rust Axum/SQLx monolith) to the target production stack:
- **Frontend:** React 18 + TypeScript + Vite + Tailwind CSS + Lucide React + Recharts
- **Backend & Auth:** Supabase Auth + Supabase Client + Supabase Edge Functions (TypeScript/Deno) + PostgreSQL with Row Level Security (RLS)
- **Database:** Supabase PostgreSQL with versioned SQL migrations, RLS policies, constraints, and audit logging.

All functional requirements, validation rule codes, intake calculations, synthetic demo scenarios, and security boundaries specified in `PRD.md` and `BUILD_PLAN.md` will be preserved without regression.

---

## 2. Existing Architecture vs. Target Architecture

### 2.1 Existing Architecture
```
[ Browser ] 
     |
     v
[ Vite Dev Proxy (:5173/api) ]
     |
     v
[ Rust Axum Backend (:8080) ]
  ├── Auth: Argon2id + HS256 JWT (local users table)
  ├── Domain: pure validation, intake calculation, CSV parsing
  ├── Services: Ingestion, Duplicate Detection, Intake Monitoring, Audit
  └── Repositories: Raw SQL via sqlx::PgPool
     |
     v
[ PostgreSQL Database (Local / Supabase pooler) ]
  ├── 12 tables (institutions, programmes, approved_intakes, intake_rules, users, students,
  │              institution_students, submission_batches, admissions, validation_issues,
  │              verification_requests, audit_events, staging_rows)
  └── No RLS enabled (authorization was enforced strictly inside Rust request extractors)
```

### 2.2 Target Architecture
```
[ React + TypeScript + Vite Frontend (:5173) ]
  ├── Tailwind CSS + Lucide Icons + Recharts
  ├── Supabase JS Client (`@supabase/supabase-js`)
  │     ├── Supabase Auth (JWT sessions with auth.users)
  │     ├── Direct RLS-protected queries (read batches, admissions, programmes, issues, dashboards)
  │     └── Edge Function Client / Database RPC for privileged workflows
  └── UI Screens:
        ├── Sign In (Supabase Auth)
        ├── Institution Dashboard (Intake cards, batch status, recent trends)
        ├── AICTE Administrator Dashboard (Cross-institution metrics, exception queue)
        ├── Student & Admission Records Registry (Filterable, masked APAAR view)
        ├── CSV Upload Wizard (Parsing, schema checking, validation report)
        ├── Batch Detail & Row-Level Validation Issues
        ├── Issue Review & Resolution Modal
        ├── Verification Request & Result (Purpose-limited disclosure, simulated badge)
        └── Attributable Audit Log Explorer
     |
     +───────────────────────────────+
     |                               |
     v (Privileged operations)       v (Scoped CRUD via RLS)
[ Supabase Edge Functions / RPC ]    |
  ├── `validate-batch` (CSV engine)  |
  ├── `commit-batch` (Tx commit)     |
  ├── `resolve-issue`                |
  ├── `create-verification`          |
  └── `enrolment-report`             |
     |                               |
     +───────────────+───────────────+
                     |
                     v
[ Supabase PostgreSQL ]
  ├── `auth.users` + `public.profiles` (RBAC: AICTE_ADMIN, INSTITUTION_ADMIN, etc.)
  ├── Core tables with constraints, triggers, and foreign keys
  ├── PostgreSQL Row Level Security (RLS) on all tables:
  │     ├── Institution scope isolation via `auth.uid()` -> `profiles.institution_id`
  │     └── Role check via `profiles.role`
  ├── PostgreSQL functions for atomic transactional batch commits & intake overruns
  └── Append-only `audit_events` trail
```

---

## 3. Current Implementation Status

| Feature / Capability | Rust Implementation Status | Frontend Status | Target Supabase Migration Status |
| :--- | :--- | :--- | :--- |
| **Authentication & Session** | Complete (local JWT + argon2id) | Complete (`LoginPage`, custom token) | Migrate to Supabase Auth + `profiles` table |
| **Institution Scoping** | Enforced in Axum extractors | Present in App state & routes | Enforce natively via PostgreSQL RLS |
| **Master Data (Institutions/Programmes/Intakes)** | Complete (migration 0001 & 0002) | Consumed via API | Complete schema + Supabase migrations |
| **CSV Upload & Parsing** | Complete (UTF-8, 8 required columns) | Basic file input on `/upload` | Client-side preview + Server-side validation |
| **Validation Engine (23 Rule Codes)** | Complete (`validation.rs`) | Displays issue list | Pure TypeScript domain module + Edge Function / DB |
| **Duplicate Detection** | Within-batch & cross-institution active | Displayed in batch issues | Preserved in validation engine + DB unique checks |
| **Intake Monitoring & Demo Rules** | Complete (`intake.rs`, `demo-v1`) | Table on institution/AICTE pages | SQL views / functions + Recharts visualization |
| **Transactional Commit** | Complete (`staging_rows` -> `admissions`) | Commit button in `BatchDetailPage` | Transactional SQL Function / Edge Function |
| **Student / Admission Registry** | Complete in Rust (`/admissions`) | Missing separate UI screen | Create dedicated `AdmissionsPage.tsx` |
| **Issue Resolution** | Complete (`/issues/:id/resolve`) | Complete (`IssuesPage.tsx`) | Supabase RPC / Edge Function |
| **Verification Service** | Complete (`/verifications`, simulated) | Missing UI screen | Create dedicated `VerificationPage.tsx` |
| **Audit Log Explorer** | Complete (`/audit`) | Missing UI screen | Create dedicated `AuditPage.tsx` |
| **Enrolment Export** | Complete (`/reports/enrolment` CSV/JSON) | Not exposed in UI | Add export action to dashboards / registry |

---

## 4. Rust-to-Supabase Responsibility Mapping

| Rust Backend Component | Primary Responsibility | Target Supabase / Frontend Architecture |
| :--- | :--- | :--- |
| `backend/src/auth/` | Password hashing, JWT issuance, scope policy | Supabase Auth (`auth.users`), `profiles` table, JWT custom claims, RLS helper functions |
| `backend/src/routes/institutions.rs` | List institutions, intakes, programmes, download template | Direct Supabase client queries with RLS + static CSV template generator |
| `backend/src/routes/batches.rs` | Upload batch, list batches, get batch, batch issues | Supabase Edge Function `validate-batch` / DB insertion + Supabase client query |
| `backend/src/services/ingestion.rs` | Transactional commit from staging into admissions, calculate overruns | Supabase Edge Function `commit-batch` or PostgreSQL stored function `commit_submission_batch` |
| `backend/src/routes/admissions.rs` | Filter & search admissions, mask APAAR ID | Direct Supabase client query with PostgreSQL column masking / RLS view |
| `backend/src/routes/issues.rs` | List open issues, resolve issue with audit record | Direct Supabase client query (read) + `resolve-issue` RPC / Edge function |
| `backend/src/routes/dashboards.rs` | Compute institution & AICTE metric aggregations | PostgreSQL aggregate views / functions (`get_institution_dashboard`, `get_aicte_dashboard`) |
| `backend/src/routes/verifications.rs` | Purpose-limited lookup, simulated external check | Supabase Edge Function `verify-student` + audit log insertion |
| `backend/src/routes/reports.rs` | Enrolment report aggregation and CSV formatting | Supabase client query / Edge Function `export-enrolment` |
| `backend/src/services/audit.rs` | Append-only audit logging | PostgreSQL `audit_events` table + trigger/RPC helper `record_audit_event` |

---

## 5. File Inventory & Action Plan

### 5.1 Files to Preserve
- `PRD.md` — Authoritative specification
- `docs/assumptions.md` — Documented regulatory and demo assumptions
- `docs/data-dictionary.md` — Schema and field definitions
- `docs/demo-script.md` — Demo presentation scenarios
- `backend/sample-data/*.csv` — Synthetic test and demo batches (`demo_batch.csv`, `clean_batch.csv`, `intake_overrun.csv`)
- `backend/migrations/*.sql` — Reference SQL for migrations

### 5.2 Files to Create / Modify
- **Database (`supabase/migrations/`):**
  - `20261020000001_initial_schema.sql` (Tables, constraints, indexes, triggers)
  - `20261020000002_rls_policies.sql` (Row Level Security policies for all roles)
  - `20261020000003_functions_and_rpc.sql` (Batch commit, intake calculation, audit helpers)
  - `20261020000004_seed_data.sql` (Master institutions, programmes, intakes, intake rules, demo profiles)
- **Supabase Functions / Client (`src/lib/` or `frontend/src/`):**
  - `frontend/src/lib/supabase.ts` (Typed Supabase client with graceful local fallback)
  - `frontend/src/lib/validation.ts` (Ported pure TypeScript validation engine matching `RuleCode`)
  - `frontend/src/lib/intake.ts` (Ported intake calculation engine matching `demo-v1`)
  - `frontend/src/lib/verification.ts` (Simulated verifier with purpose-limited fields)
- **Frontend Enhancements (`frontend/`):**
  - Install Tailwind CSS, `@types/node`, `lucide-react`, `recharts`, `@supabase/supabase-js`
  - Update `index.html` and Tailwind styling for high-aesthetic UI
  - Enhance existing pages (`LoginPage`, `InstitutionDashboardPage`, `AicteDashboardPage`, `UploadPage`, `BatchDetailPage`, `IssuesPage`)
  - Add missing required screens:
    - `AdmissionsPage.tsx` (Student / admission registry search & masked APAAR view)
    - `VerificationPage.tsx` (Verification request wizard & simulated result)
    - `AuditPage.tsx` (Audit log viewer with scope filtering)
  - Update `Layout.tsx` and `App.tsx` routing and navigation
- **Documentation & Environment:**
  - `.env.example` — Update for Supabase configuration (`VITE_SUPABASE_URL`, `VITE_SUPABASE_ANON_KEY`, etc.)
  - `README.md` — Update setup, architecture, and test instructions

### 5.3 Files to Retire / Deprecate
- `backend/` — Will be superseded by Supabase (Rust backend retained in repository as reference until migration verification is complete)
- `src/server.js` & `src/domain.js` — Legacy node prototype, superseded by typed frontend and Supabase functions

---

## 6. Database Migration & RLS Strategy

### 6.1 Schema Refinements for Supabase Auth
- Map `users` table to `public.profiles`, referencing `auth.users(id) ON DELETE CASCADE`.
- Store `role` (`AICTE_ADMIN`, `INSTITUTION_ADMIN`, `INSTITUTION_DATA_OPERATOR`, `AUTHORIZED_VERIFIER`, `AUDITOR`) and `institution_id` in `public.profiles`.
- Create PostgreSQL helper functions:
  ```sql
  CREATE OR REPLACE FUNCTION public.current_user_role() RETURNS text ...
  CREATE OR REPLACE FUNCTION public.current_user_institution() RETURNS uuid ...
  ```

### 6.2 Row Level Security (RLS) Policy Design
- `institutions`:
  - `SELECT`: Public for authenticated users or scoped to assigned institution.
  - `INSERT/UPDATE/DELETE`: AICTE Admin only.
- `programmes` & `approved_intakes` & `intake_rules`:
  - `SELECT`: Authenticated users.
  - `MODIFY`: AICTE Admin only.
- `submission_batches`, `staging_rows`, `validation_issues`:
  - `SELECT`: AICTE Admin (all) or Institution users where `institution_id = current_user_institution()`.
  - `INSERT`: Institution users for their own institution.
  - `UPDATE`: Institution users (own batches) / AICTE Admin (issue resolutions).
- `students` & `institution_students` & `admissions`:
  - `SELECT`: AICTE Admin (all) or Institution users where `institution_id = current_user_institution()`.
  - `INSERT/UPDATE`: Only via trusted transactional batch commit function.
- `audit_events`:
  - `SELECT`: AICTE Admin and Auditor roles.
  - `INSERT`: Allowed for authenticated system operations; append-only (no UPDATE or DELETE).

---

## 7. Risks and Rollback Approach

1. **Risk:** Supabase hosted credentials not configured during local development.
   - **Mitigation:** The application includes a self-contained local repository adapter / mock client mode that preserves 100% of the UI interactivity and domain rules using synthetic data, while seamlessly connecting to any hosted Supabase instance once `.env` credentials are provided.
2. **Risk:** Discrepancy in validation rule codes between Rust and TypeScript.
   - **Mitigation:** Direct port of `backend/src/domain/validation.rs` rule definitions, severity rankings, and evidence formats into typed TypeScript unit tests with 100% test coverage against sample CSV files.
3. **Risk:** RLS policy performance on large queries.
   - **Mitigation:** Comprehensive B-tree indexes on `(institution_id)`, `(batch_id)`, `(programme_id, academic_year)`, and `(apaar_id)`.
4. **Rollback:** The existing `backend/` directory and Git history remain intact and untouched until all verification checks pass.

---

## 8. Ordered Implementation Phases

- [x] **Phase A: Repository Inspection & Migration Plan Creation**
- [ ] **Phase B: Dependencies, Styling & Supabase Foundation Setup**
- [ ] **Phase C: PostgreSQL Schema, Migrations, RLS Policies & Seed Data**
- [ ] **Phase D: TypeScript Domain Logic Porting (Validation, Intake, Verification, Ingestion)**
- [ ] **Phase E: Frontend Supabase Client Integration & Auth Session Management**
- [ ] **Phase F: Complete Frontend Screens (Dashboards, Upload Wizard, Registry, Verification, Audit)**
- [ ] **Phase G: Automated Testing (Vitest Typecheck, Build, Synthetic Validation Scenarios)**
- [ ] **Phase H: Documentation Update, Environment Files & Demo Preparation**
