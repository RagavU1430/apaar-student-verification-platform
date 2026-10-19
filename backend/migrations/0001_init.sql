-- AICTE APAAR platform - initial schema.
-- All timestamps are stored in UTC (timestamptz) and rendered in local time
-- by the client. See BUILD_PLAN.md section 6.

CREATE TABLE institutions (
    id              UUID PRIMARY KEY,
    official_code   TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'ACTIVE',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE programmes (
    id              UUID PRIMARY KEY,
    institution_id  UUID NOT NULL REFERENCES institutions(id) ON DELETE CASCADE,
    code            TEXT NOT NULL,
    name            TEXT NOT NULL,
    level           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'ACTIVE',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (institution_id, code)
);

-- Approved capacity and its source. Capacity is always traceable to a source
-- reference and an effective version.
CREATE TABLE approved_intakes (
    id                    UUID PRIMARY KEY,
    programme_id          UUID NOT NULL REFERENCES programmes(id) ON DELETE CASCADE,
    academic_year         TEXT NOT NULL,
    regular_intake        INTEGER NOT NULL CHECK (regular_intake >= 0),
    supernumerary_capacity INTEGER NOT NULL DEFAULT 0 CHECK (supernumerary_capacity >= 0),
    source_reference      TEXT NOT NULL,
    rule_version          TEXT NOT NULL,
    effective_version     INTEGER NOT NULL DEFAULT 1,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (programme_id, academic_year, effective_version)
);

-- Versioned counting rules per seat category. Policy data, not code.
CREATE TABLE intake_rules (
    id                    UUID PRIMARY KEY,
    rule_version          TEXT NOT NULL,
    seat_category         TEXT NOT NULL,
    counts_toward_regular BOOLEAN NOT NULL DEFAULT TRUE,
    uses_supernumerary_pool BOOLEAN NOT NULL DEFAULT FALSE,
    notes                 TEXT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (rule_version, seat_category)
);

CREATE TABLE users (
    id                UUID PRIMARY KEY,
    login_identifier  TEXT NOT NULL UNIQUE,
    display_name      TEXT NOT NULL,
    role              TEXT NOT NULL,
    institution_id    UUID REFERENCES institutions(id),
    password_hash     TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'ACTIVE',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Minimal identity record. Deliberately holds no date of birth, no document
-- number and no biometric data (BUILD_PLAN.md A-05).
CREATE TABLE students (
    id              UUID PRIMARY KEY,
    apaar_id        TEXT NOT NULL UNIQUE,
    apaar_id_last4  TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Institution-scoped view of a student: the local student id and name live
-- with the institution that owns them.
CREATE TABLE institution_students (
    id                     UUID PRIMARY KEY,
    student_id             UUID NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    institution_id         UUID NOT NULL REFERENCES institutions(id) ON DELETE CASCADE,
    institution_student_id TEXT NOT NULL,
    full_name              TEXT,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (institution_id, institution_student_id)
);

CREATE TABLE submission_batches (
    id                 UUID PRIMARY KEY,
    institution_id     UUID NOT NULL REFERENCES institutions(id),
    submitted_by       UUID NOT NULL REFERENCES users(id),
    file_name          TEXT NOT NULL,
    file_hash          TEXT NOT NULL,
    schema_version     TEXT NOT NULL,
    status             TEXT NOT NULL,
    row_count          INTEGER NOT NULL DEFAULT 0,
    accepted_count     INTEGER NOT NULL DEFAULT 0,
    warning_count      INTEGER NOT NULL DEFAULT 0,
    duplicate_count    INTEGER NOT NULL DEFAULT 0,
    rejected_count     INTEGER NOT NULL DEFAULT 0,
    needs_review_count INTEGER NOT NULL DEFAULT 0,
    idempotency_key    TEXT UNIQUE,
    submitted_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    committed_at       TIMESTAMPTZ
);

CREATE TABLE admissions (
    id                  UUID PRIMARY KEY,
    student_identity_id UUID NOT NULL REFERENCES institution_students(id),
    institution_id      UUID NOT NULL REFERENCES institutions(id),
    programme_id        UUID NOT NULL REFERENCES programmes(id),
    academic_year       TEXT NOT NULL,
    admission_date      DATE NOT NULL,
    status              TEXT NOT NULL,
    seat_category       TEXT NOT NULL,
    source_batch_id     UUID REFERENCES submission_batches(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- A student may hold several historical admissions, but only one active one.
CREATE UNIQUE INDEX admissions_one_active_per_student
    ON admissions (student_identity_id)
    WHERE status IN ('ACTIVE', 'ENROLLED');

CREATE INDEX admissions_institution_idx ON admissions (institution_id);
CREATE INDEX admissions_programme_year_idx ON admissions (programme_id, academic_year);
CREATE INDEX students_apaar_idx ON students (apaar_id);

CREATE TABLE validation_issues (
    id              UUID PRIMARY KEY,
    batch_id        UUID NOT NULL REFERENCES submission_batches(id) ON DELETE CASCADE,
    institution_id  UUID NOT NULL REFERENCES institutions(id),
    row_reference   TEXT NOT NULL,
    rule_code       TEXT NOT NULL,
    severity        TEXT NOT NULL,
    message         TEXT NOT NULL,
    field           TEXT,
    evidence        TEXT,
    rule_version    TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'OPEN',
    resolution_note TEXT,
    resolved_by     UUID REFERENCES users(id),
    resolved_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX validation_issues_batch_idx ON validation_issues (batch_id);
CREATE INDEX validation_issues_status_idx ON validation_issues (status, institution_id);

CREATE TABLE verification_requests (
    id              UUID PRIMARY KEY,
    requester_id    UUID NOT NULL REFERENCES users(id),
    purpose         TEXT NOT NULL,
    record_ref      TEXT NOT NULL,
    fields_disclosed JSONB NOT NULL DEFAULT '[]'::jsonb,
    outcome         TEXT NOT NULL,
    source          TEXT NOT NULL,
    simulated       BOOLEAN NOT NULL DEFAULT TRUE,
    requested_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Append-only application level event history.
CREATE TABLE audit_events (
    id             UUID PRIMARY KEY,
    actor_id       UUID,
    actor_role     TEXT,
    action         TEXT NOT NULL,
    entity_type    TEXT NOT NULL,
    entity_id      TEXT,
    institution_id UUID,
    outcome        TEXT NOT NULL,
    correlation_id TEXT,
    summary        TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX audit_events_created_idx ON audit_events (created_at DESC);
CREATE INDEX audit_events_actor_idx ON audit_events (actor_id);
