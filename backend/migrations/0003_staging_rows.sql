-- Staging area for validated-but-not-yet-committed rows.
--
-- The raw uploaded file is never retained. Rows live here only until the batch
-- is committed, after which they are deleted and the accepted records live in
-- `admissions`. This keeps data minimisation (BUILD_PLAN.md section 6).

CREATE TABLE staging_rows (
    id                     UUID PRIMARY KEY,
    batch_id               UUID NOT NULL REFERENCES submission_batches(id) ON DELETE CASCADE,
    row_number             INTEGER NOT NULL,
    apaar_id               TEXT NOT NULL,
    institution_student_id TEXT NOT NULL,
    full_name              TEXT,
    programme_id           UUID NOT NULL REFERENCES programmes(id),
    academic_year          TEXT NOT NULL,
    admission_date         DATE NOT NULL,
    admission_status       TEXT NOT NULL,
    seat_category          TEXT NOT NULL,
    row_status             TEXT NOT NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (batch_id, row_number)
);

CREATE INDEX staging_rows_batch_idx ON staging_rows (batch_id);
