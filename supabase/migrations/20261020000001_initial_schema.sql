-- ==============================================================================
-- AICTE APAAR Student Verification Platform - Supabase Initial Schema
-- Migration: 20261020000001_initial_schema.sql
-- ==============================================================================

-- 1. INSTITUTIONS
CREATE TABLE IF NOT EXISTS public.institutions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    official_code   TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'SUSPENDED', 'INACTIVE')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now())
);

-- 2. USER PROFILES (tied to Supabase Auth `auth.users`)
CREATE TABLE IF NOT EXISTS public.profiles (
    id                UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
    login_identifier  TEXT NOT NULL UNIQUE,
    display_name      TEXT NOT NULL,
    role              TEXT NOT NULL CHECK (role IN (
                        'AICTE_ADMIN',
                        'INSTITUTION_ADMIN',
                        'INSTITUTION_DATA_OPERATOR',
                        'AUTHORIZED_VERIFIER',
                        'AUDITOR'
                      )),
    institution_id    UUID REFERENCES public.institutions(id) ON DELETE SET NULL,
    status            TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'INACTIVE', 'SUSPENDED')),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now()),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now())
);

-- 3. PROGRAMMES
CREATE TABLE IF NOT EXISTS public.programmes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id  UUID NOT NULL REFERENCES public.institutions(id) ON DELETE CASCADE,
    code            TEXT NOT NULL,
    name            TEXT NOT NULL,
    level           TEXT NOT NULL CHECK (level IN ('UG', 'PG', 'DIPLOMA', 'DOCTORAL')),
    status          TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'DISCONTINUED')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now()),
    UNIQUE (institution_id, code)
);

-- 4. APPROVED INTAKES
CREATE TABLE IF NOT EXISTS public.approved_intakes (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    programme_id           UUID NOT NULL REFERENCES public.programmes(id) ON DELETE CASCADE,
    academic_year          TEXT NOT NULL,
    regular_intake         INTEGER NOT NULL CHECK (regular_intake >= 0),
    supernumerary_capacity INTEGER NOT NULL DEFAULT 0 CHECK (supernumerary_capacity >= 0),
    source_reference       TEXT NOT NULL,
    rule_version           TEXT NOT NULL,
    effective_version      INTEGER NOT NULL DEFAULT 1,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now()),
    UNIQUE (programme_id, academic_year, effective_version)
);

-- 5. INTAKE RULES (Data-driven counting rules)
CREATE TABLE IF NOT EXISTS public.intake_rules (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_version            TEXT NOT NULL,
    seat_category           TEXT NOT NULL,
    counts_toward_regular   BOOLEAN NOT NULL DEFAULT TRUE,
    uses_supernumerary_pool BOOLEAN NOT NULL DEFAULT FALSE,
    notes                   TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now()),
    UNIQUE (rule_version, seat_category)
);

-- 6. STUDENTS (Minimal Central Academic Identity)
CREATE TABLE IF NOT EXISTS public.students (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    apaar_id        TEXT NOT NULL UNIQUE,
    apaar_id_last4  TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now())
);

-- 7. INSTITUTION STUDENTS (Local Institutional Enrollment Identity)
CREATE TABLE IF NOT EXISTS public.institution_students (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id             UUID NOT NULL REFERENCES public.students(id) ON DELETE CASCADE,
    institution_id         UUID NOT NULL REFERENCES public.institutions(id) ON DELETE CASCADE,
    institution_student_id TEXT NOT NULL,
    full_name              TEXT,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now()),
    UNIQUE (institution_id, institution_student_id)
);

-- 8. SUBMISSION BATCHES
CREATE TABLE IF NOT EXISTS public.submission_batches (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id     UUID NOT NULL REFERENCES public.institutions(id) ON DELETE RESTRICT,
    submitted_by       UUID REFERENCES auth.users(id) ON DELETE SET NULL,
    file_name          TEXT NOT NULL,
    file_hash          TEXT NOT NULL,
    schema_version     TEXT NOT NULL,
    status             TEXT NOT NULL CHECK (status IN ('VALIDATED', 'COMMITTED', 'REJECTED', 'FAILED')),
    row_count          INTEGER NOT NULL DEFAULT 0,
    accepted_count     INTEGER NOT NULL DEFAULT 0,
    warning_count      INTEGER NOT NULL DEFAULT 0,
    duplicate_count    INTEGER NOT NULL DEFAULT 0,
    rejected_count     INTEGER NOT NULL DEFAULT 0,
    needs_review_count INTEGER NOT NULL DEFAULT 0,
    idempotency_key    TEXT UNIQUE,
    submitted_at       TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now()),
    committed_at       TIMESTAMPTZ
);

-- 9. STAGING ROWS (Pre-commit temporary holding area)
CREATE TABLE IF NOT EXISTS public.staging_rows (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id               UUID NOT NULL REFERENCES public.submission_batches(id) ON DELETE CASCADE,
    row_number             INTEGER NOT NULL,
    apaar_id               TEXT NOT NULL,
    institution_student_id TEXT NOT NULL,
    full_name              TEXT,
    programme_id           UUID NOT NULL REFERENCES public.programmes(id),
    academic_year          TEXT NOT NULL,
    admission_date         DATE NOT NULL,
    admission_status       TEXT NOT NULL,
    seat_category          TEXT NOT NULL,
    row_status             TEXT NOT NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now()),
    UNIQUE (batch_id, row_number)
);

-- 10. ADMISSIONS (Committed Active / Historical Records)
CREATE TABLE IF NOT EXISTS public.admissions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_identity_id UUID NOT NULL REFERENCES public.institution_students(id) ON DELETE RESTRICT,
    institution_id      UUID NOT NULL REFERENCES public.institutions(id) ON DELETE RESTRICT,
    programme_id        UUID NOT NULL REFERENCES public.programmes(id) ON DELETE RESTRICT,
    academic_year       TEXT NOT NULL,
    admission_date      DATE NOT NULL,
    status              TEXT NOT NULL CHECK (status IN ('ACTIVE', 'ENROLLED', 'WITHDRAWN', 'COMPLETED', 'TRANSFERRED', 'CANCELLED')),
    seat_category       TEXT NOT NULL,
    source_batch_id     UUID REFERENCES public.submission_batches(id) ON DELETE SET NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now()),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now())
);

-- Partial index: only one active/enrolled admission per student identity
CREATE UNIQUE INDEX IF NOT EXISTS admissions_one_active_per_student
    ON public.admissions (student_identity_id)
    WHERE status IN ('ACTIVE', 'ENROLLED');

-- 11. VALIDATION ISSUES
CREATE TABLE IF NOT EXISTS public.validation_issues (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id        UUID NOT NULL REFERENCES public.submission_batches(id) ON DELETE CASCADE,
    institution_id  UUID NOT NULL REFERENCES public.institutions(id) ON DELETE CASCADE,
    row_reference   TEXT NOT NULL,
    rule_code       TEXT NOT NULL,
    severity        TEXT NOT NULL CHECK (severity IN ('ERROR', 'WARNING', 'DUPLICATE', 'REVIEW')),
    message         TEXT NOT NULL,
    field           TEXT,
    evidence        TEXT,
    rule_version    TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'RESOLVED', 'DISMISSED')),
    resolution_note TEXT,
    resolved_by     UUID REFERENCES auth.users(id) ON DELETE SET NULL,
    resolved_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now())
);

-- 12. VERIFICATION REQUESTS
CREATE TABLE IF NOT EXISTS public.verification_requests (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requester_id     UUID REFERENCES auth.users(id) ON DELETE SET NULL,
    purpose          TEXT NOT NULL,
    record_ref       TEXT NOT NULL,
    fields_disclosed JSONB NOT NULL DEFAULT '[]'::jsonb,
    outcome          TEXT NOT NULL,
    source           TEXT NOT NULL,
    simulated        BOOLEAN NOT NULL DEFAULT TRUE,
    requested_at     TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now())
);

-- 13. AUDIT EVENTS (Append-only)
CREATE TABLE IF NOT EXISTS public.audit_events (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id       UUID REFERENCES auth.users(id) ON DELETE SET NULL,
    actor_role     TEXT,
    action         TEXT NOT NULL,
    entity_type    TEXT NOT NULL,
    entity_id      TEXT,
    institution_id UUID REFERENCES public.institutions(id) ON DELETE SET NULL,
    outcome        TEXT NOT NULL,
    correlation_id TEXT,
    summary        TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT timezone('utc'::text, now())
);

-- Indexes for fast querying and duplicate detection
CREATE INDEX IF NOT EXISTS idx_admissions_institution ON public.admissions (institution_id);
CREATE INDEX IF NOT EXISTS idx_admissions_programme_year ON public.admissions (programme_id, academic_year);
CREATE INDEX IF NOT EXISTS idx_students_apaar ON public.students (apaar_id);
CREATE INDEX IF NOT EXISTS idx_submission_batches_institution ON public.submission_batches (institution_id, submitted_at DESC);
CREATE INDEX IF NOT EXISTS idx_staging_rows_batch ON public.staging_rows (batch_id);
CREATE INDEX IF NOT EXISTS idx_validation_issues_batch ON public.validation_issues (batch_id);
CREATE INDEX IF NOT EXISTS idx_validation_issues_status ON public.validation_issues (status, institution_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_created ON public.audit_events (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_events_actor ON public.audit_events (actor_id);
