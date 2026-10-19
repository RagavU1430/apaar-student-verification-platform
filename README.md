# AICTE APAAR - Student Data & Verification Platform

React + TypeScript + Vite frontend on Supabase (PostgreSQL, Supabase Auth, Row Level Security, and Edge Functions), built from [`PRD.md`](./PRD.md) and migrated according to [`MIGRATION_PLAN.md`](./MIGRATION_PLAN.md).

> **Prototype scope.** This is a demonstration build. There is **no** live APAAR gateway integration, the intake
> counting rule is a labelled demo rule (`demo-v1`), all capacity fixtures are synthetic (`DEMO-SEED-v1`), and no record here
> is authoritative. See [`docs/assumptions.md`](./docs/assumptions.md) for every open question and
> the assumption it was locked to.

---

## What It Does

- **Batch CSV Ingestion:** An authorised institution uploads a CSV of admissions and receives **row-level validation results** with zero rows silently discarded.
- **Explainable Findings:** Every finding carries a stable rule code (23 enumerated codes), a plain-language explanation, concrete evidence, and the rule version that produced it.
- **Duplicate Detection:** Duplicate APAAR IDs are detected **within a batch** and **across institutions**, and surfaced as review flags rather than automated fraud verdicts.
- **Intake Monitoring & Overrun Detection:** Admissions are compared against approved capacity under a **versioned, configurable** counting rule (`demo-v1`); capacity overruns trigger reviewable exceptions.
- **Student & Admission Registry:** Authorised users search and filter committed records with privacy-compliant masked APAAR IDs (`APAAR-***-XXXX`).
- **AICTE Central Regulator Dashboard:** Cross-institution aggregates, comparative Recharts visualisations, and a review queue.
- **Purpose-Limited Verification:** Authorised external verifiers check academic identity status with strict purpose policies and a prominent simulated adapter notice.
- **Attributable Audit Trail:** Every upload, transactional commit, issue resolution, and verification attempt is recorded to an append-only audit log.

---

## Architecture

```
[ React + TypeScript + Vite Frontend ]
  ├── Tailwind CSS + Lucide Icons + Recharts
  ├── Supabase JS Client (@supabase/supabase-js)
  └── Screens:
        ├── Institution Dashboard & Capacity Utilisation
        ├── AICTE Central Monitoring & Exception Queue
        ├── CSV Upload Wizard (with quick synthetic fixtures)
        ├── Batch Detail & Row-Level Validation Findings
        ├── Student & Admission Registry (Masked APAAR IDs)
        ├── Validation Issue Review & Resolution Queue
        ├── Authorised Verification Service (Purpose-Limited)
        └── Attributable Audit Log Explorer
     |
     v
[ Supabase PostgreSQL + Auth + RLS ]
  ├── auth.users & public.profiles (RBAC & Institution Scope)
  ├── 12 Relational Tables (UUIDs, Foreign Keys, UTC Timestamps)
  ├── Row Level Security (RLS) on all tables (Institution isolation)
  ├── Database functions for atomic batch commit & intake overruns
  └── Append-only audit_events history
```

---

## Layout

```
frontend/                 React + TypeScript + Vite Application
  src/api/client.ts       Typed API client
  src/lib/types.ts        Domain, DB schema, and API contracts
  src/lib/validation.ts   Pure validation engine & CSV parser
  src/lib/intake.ts       Intake counting & capacity calculations
  src/lib/verification.ts Purpose-limited verification policy
  src/lib/repository.ts   Unified repository (Supabase + local demo mode)
  src/lib/supabase.ts     Supabase client initialization
  src/pages/              All 11 PRD-required application screens
  src/components/         Layout, StatusBadge, reusable UI
  src/test/               Automated Vitest test suite
supabase/
  migrations/             PostgreSQL migrations (schema, RLS, functions, seed data)
backend/sample-data/      Synthetic CSV batches (demo_batch.csv, clean_batch.csv, intake_overrun.csv)
docs/                     Data dictionary, demo script, assumptions register
```

---

## Quick Start (Local Development)

### 1. Prerequisites
- **Node.js 18+**
- (Optional) **Supabase CLI** (`npx supabase`) if connecting to a hosted Supabase project.

### 2. Install & Run Frontend
```powershell
cd frontend
npm install
npm run dev
```

Open [http://localhost:5173](http://localhost:5173).

> **Zero-Config Demo Mode:** If Supabase credentials are not set in `.env`, the platform operates in a self-contained prototype mode using the exact synthetic seed fixtures from migration `0002`/`0004`. All upload, commit, review, verification, and audit workflows function end-to-end.

---

## Connecting to a Live Supabase Project

1. Create a project at [supabase.com](https://supabase.com).
2. Apply the database migrations in `supabase/migrations/` using the Supabase CLI:
   ```powershell
   npx supabase link --project-ref your-project-ref
   npx supabase db push
   ```
3. Set your frontend environment variables in `frontend/.env`:
   ```env
   VITE_SUPABASE_URL=https://your-project-ref.supabase.co
   VITE_SUPABASE_ANON_KEY=your-anon-publishable-key
   ```
4. Restart the frontend dev server (`npm run dev`).

---

## Demo Accounts

All demo accounts use the password: `DemoPass123!` (or 1-click select on the sign-in page):

| Login | Role | Access Scope |
| :--- | :--- | :--- |
| `alpha.admin` | `INSTITUTION_ADMIN` | Alpha Institute of Technology (Upload, Commit, Issues, Registry) |
| `alpha.operator` | `INSTITUTION_DATA_OPERATOR` | Alpha Institute (Upload, Batch Review, Commit) |
| `beta.operator` | `INSTITUTION_DATA_OPERATOR` | Beta Institute of Engineering |
| `aicte.admin` | `AICTE_ADMIN` | Full Central Regulatory Scope (All institutions, Exception Queue, Enrolment Reports) |
| `verifier` | `AUTHORIZED_VERIFIER` | Purpose-limited verification lookups |
| `auditor` | `AUDITOR` | Read-only audit log inspection |

---

## Guided Walkthrough

1. **Sign In:** Sign in as `alpha.admin`.
2. **Explore Dashboard:** Observe current regular intake utilization (CSE-01, ECE-01) under demo rule `demo-v1`.
3. **Upload CSV Batch:** Navigate to **Upload Wizard**. Click **"demo_batch.csv (Errors, Duplicates & Warnings)"** to load the evaluator fixture. Click **Upload and Validate Batch**.
4. **Inspect Findings:** Review row-level findings. Notice flagged errors (missing APAAR ID, invalid date format `05-07-2026`, unknown programme `UNKNOWN-99`, unknown seat category `SCHOLARSHIP`) and duplicate candidate (`APAAR-DEMO-0001` repeated in batch).
5. **Transactional Commit:** Click **"Commit Accepted Rows"**. Valid records are committed transactionally to `admissions`, staged rows are cleared, and intake usage updates.
6. **Trigger Capacity Overrun:** Go back to **Upload Wizard**, click **"intake_overrun.csv (Capacity Exception 65/60)"**, upload and commit it. Notice the capacity exception alert and automated issue creation in the review queue.
7. **Switch to AICTE Regulator:** Sign out and sign in as `aicte.admin`. Open **AICTE Regulator View** to see cross-institution metric comparison charts, the exception queue, and export the regulatory enrolment report.
8. **Review Issue:** Open **Review Queue**, enter a mandatory reviewer note (e.g., *"Reviewed institutional intake sanction letter; confirmed exception for audit"*), and click **Resolve**.
9. **Verify Student:** Navigate to **Verification Service**, select **Employer Background Screening**, enter student ID `ALPHA-0001`, and verify the purpose-limited disclosed fields and prominent simulated disclaimer.
10. **Audit Log:** Open **Audit Trail** to see the full, attributable record of all actions performed.

---

## Automated Tests

Run all unit tests, typechecks, and builds:

```powershell
# Run frontend vitest suite (validation engine, duplicates, intake rules, verification policy)
npm run test:frontend

# Run TypeScript typecheck
npm run typecheck

# Production build
npm run build
```

---

## Documentation

- [`PRD.md`](./PRD.md) — Product Requirements Document
- [`MIGRATION_PLAN.md`](./MIGRATION_PLAN.md) — Architectural migration plan and RLS strategy
- [`BUILD_PLAN.md`](./BUILD_PLAN.md) — Technical specifications and locked constraints
- [`docs/data-dictionary.md`](./docs/data-dictionary.md) — Entity and attribute definitions
- [`docs/assumptions.md`](./docs/assumptions.md) — Labelled assumptions register (A-01 to A-13)
- [`docs/demo-script.md`](./docs/demo-script.md) — Hackathon demonstration script
