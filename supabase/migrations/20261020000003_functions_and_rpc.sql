-- ==============================================================================
-- AICTE APAAR Student Verification Platform - Functions & Stored Procedures
-- Migration: 20261020000003_functions_and_rpc.sql
-- ==============================================================================

-- 1. TRANSACTIONAL BATCH COMMIT
CREATE OR REPLACE FUNCTION public.commit_submission_batch(
    p_batch_id UUID,
    p_rule_version TEXT DEFAULT 'demo-v1'
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    v_batch RECORD;
    v_row RECORD;
    v_user_id UUID := auth.uid();
    v_user_role TEXT;
    v_user_inst UUID;
    v_student_id UUID;
    v_identity_id UUID;
    v_already_active BOOLEAN;
    v_committed INT := 0;
    v_skipped INT := 0;
    v_norm_apaar TEXT;
    v_last4 TEXT;
    v_usage RECORD;
    v_exceptions JSONB := '[]'::jsonb;
    v_regular_count BIGINT;
    v_supernumerary_count BIGINT;
    v_reg_capacity INT;
    v_super_capacity INT;
    v_prog_code TEXT;
    v_excess BIGINT;
    v_issue_msg TEXT;
BEGIN
    -- Fetch batch
    SELECT * INTO v_batch FROM public.submission_batches WHERE id = p_batch_id;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Batch not found' USING ERRCODE = 'P0002';
    END IF;

    IF v_batch.status = 'COMMITTED' THEN
        RAISE EXCEPTION 'Batch is already committed' USING ERRCODE = 'P0001';
    END IF;

    -- Verify authorization
    SELECT role, institution_id INTO v_user_role, v_user_inst FROM public.profiles WHERE id = v_user_id;
    IF v_user_role IS NULL THEN
        -- Allow call if service role or direct execution, fallback to batch institution
        v_user_inst := v_batch.institution_id;
    ELSIF v_user_role != 'AICTE_ADMIN' AND v_user_inst != v_batch.institution_id THEN
        RAISE EXCEPTION 'Unauthorized: cross-institution commit denied' USING ERRCODE = '42501';
    END IF;

    -- Iterate over staging rows
    FOR v_row IN 
        SELECT * FROM public.staging_rows 
        WHERE batch_id = p_batch_id AND row_status IN ('ACCEPTED', 'ACCEPTED_WITH_WARNING')
        ORDER BY row_number ASC
    LOOP
        v_norm_apaar := upper(trim(v_row.apaar_id));
        IF length(v_norm_apaar) >= 4 THEN
            v_last4 := right(v_norm_apaar, 4);
        ELSE
            v_last4 := v_norm_apaar;
        END IF;

        -- 1. Upsert Student Identity
        INSERT INTO public.students (apaar_id, apaar_id_last4)
        VALUES (v_norm_apaar, v_last4)
        ON CONFLICT (apaar_id) DO UPDATE SET apaar_id = EXCLUDED.apaar_id
        RETURNING id INTO v_student_id;

        -- 2. Upsert Institution Student
        INSERT INTO public.institution_students (student_id, institution_id, institution_student_id, full_name)
        VALUES (v_student_id, v_batch.institution_id, v_row.institution_student_id, v_row.full_name)
        ON CONFLICT (institution_id, institution_student_id)
        DO UPDATE SET full_name = COALESCE(EXCLUDED.full_name, institution_students.full_name)
        RETURNING id INTO v_identity_id;

        -- 3. Check for conflicting active admission
        SELECT EXISTS (
            SELECT 1 FROM public.admissions
            WHERE student_identity_id = v_identity_id
              AND programme_id = v_row.programme_id
              AND academic_year = v_row.academic_year
              AND status IN ('ACTIVE', 'ENROLLED')
        ) INTO v_already_active;

        IF v_already_active THEN
            v_skipped := v_skipped + 1;
            INSERT INTO public.validation_issues (
                batch_id, institution_id, row_reference, rule_code, severity,
                message, field, evidence, rule_version, status
            ) VALUES (
                p_batch_id, v_batch.institution_id, format('row %s', v_row.row_number),
                'DUPLICATE_ACTIVE_ADMISSION', 'REVIEW',
                'Skipped at commit: the student already has an active admission for this programme and year.',
                'apaar_id', v_norm_apaar, p_rule_version, 'OPEN'
            );
            CONTINUE;
        END IF;

        -- 4. Insert Admission
        INSERT INTO public.admissions (
            student_identity_id, institution_id, programme_id, academic_year,
            admission_date, status, seat_category, source_batch_id
        ) VALUES (
            v_identity_id, v_batch.institution_id, v_row.programme_id, v_row.academic_year,
            v_row.admission_date, v_row.admission_status, v_row.seat_category, p_batch_id
        );

        v_committed := v_committed + 1;
    END LOOP;

    -- Update batch status
    UPDATE public.submission_batches 
    SET status = 'COMMITTED', committed_at = timezone('utc'::text, now())
    WHERE id = p_batch_id;

    -- Clear staging rows
    DELETE FROM public.staging_rows WHERE batch_id = p_batch_id;

    -- Record Audit Event
    INSERT INTO public.audit_events (
        actor_id, actor_role, action, entity_type, entity_id, institution_id, outcome, summary
    ) VALUES (
        v_user_id, COALESCE(v_user_role, 'SYSTEM'), 'batch.commit', 'submission_batch',
        p_batch_id::text, v_batch.institution_id, 'SUCCESS',
        format('committed %s records, skipped %s', v_committed, v_skipped)
    );

    -- 5. Intake Monitoring & Capacity Exception Detection
    FOR v_usage IN
        SELECT DISTINCT a.programme_id, a.academic_year
        FROM public.admissions a
        WHERE a.source_batch_id = p_batch_id
    LOOP
        SELECT p.code INTO v_prog_code FROM public.programmes p WHERE p.id = v_usage.programme_id;
        
        -- Approved intake
        SELECT regular_intake, supernumerary_capacity 
        INTO v_reg_capacity, v_super_capacity
        FROM public.approved_intakes
        WHERE programme_id = v_usage.programme_id AND academic_year = v_usage.academic_year
        ORDER BY effective_version DESC LIMIT 1;

        IF FOUND THEN
            -- Calculate regular & supernumerary counts
            SELECT 
                COALESCE(SUM(CASE WHEN r.counts_toward_regular THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN NOT r.counts_toward_regular AND r.uses_supernumerary_pool THEN 1 ELSE 0 END), 0)
            INTO v_regular_count, v_supernumerary_count
            FROM public.admissions adm
            LEFT JOIN public.intake_rules r 
                ON r.seat_category = adm.seat_category AND r.rule_version = p_rule_version
            WHERE adm.programme_id = v_usage.programme_id
              AND adm.academic_year = v_usage.academic_year
              AND adm.status IN ('ACTIVE', 'ENROLLED');

            IF v_regular_count > v_reg_capacity THEN
                v_excess := v_regular_count - v_reg_capacity;
                v_issue_msg := format('Active regular admissions (%s) exceed the configured regular intake (%s). Excess: %s.',
                                      v_regular_count, v_reg_capacity, v_excess);

                INSERT INTO public.validation_issues (
                    batch_id, institution_id, row_reference, rule_code, severity,
                    message, evidence, rule_version, status
                ) VALUES (
                    p_batch_id, v_batch.institution_id, format('%s / %s', v_prog_code, v_usage.academic_year),
                    'INTAKE_CAPACITY_EXCEEDED', 'REVIEW',
                    v_issue_msg,
                    format('rule %s | counted %s regular / %s supernumerary', p_rule_version, v_regular_count, v_supernumerary_count),
                    p_rule_version, 'OPEN'
                );

                v_exceptions := v_exceptions || jsonb_build_object(
                    'programme_code', v_prog_code,
                    'academic_year', v_usage.academic_year,
                    'rule_version', p_rule_version,
                    'message', v_issue_msg,
                    'regular_count', v_regular_count,
                    'regular_intake', v_reg_capacity,
                    'excess', v_excess
                );
            END IF;
        END IF;
    END LOOP;

    RETURN jsonb_build_object(
        'batch_id', p_batch_id,
        'status', 'COMMITTED',
        'committed', v_committed,
        'skipped', v_skipped,
        'intake_exceptions', v_exceptions
    );
END;
$$;

-- 2. ISSUE RESOLUTION FUNCTION
CREATE OR REPLACE FUNCTION public.resolve_validation_issue(
    p_issue_id UUID,
    p_status TEXT,
    p_note TEXT
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    v_issue RECORD;
    v_user_id UUID := auth.uid();
    v_role TEXT := public.get_auth_role();
BEGIN
    IF p_status NOT IN ('RESOLVED', 'DISMISSED') THEN
        RAISE EXCEPTION 'Status must be RESOLVED or DISMISSED' USING ERRCODE = '22023';
    END IF;

    SELECT * INTO v_issue FROM public.validation_issues WHERE id = p_issue_id;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Validation issue not found' USING ERRCODE = 'P0002';
    END IF;

    -- Must be AICTE Admin or belonging to that institution
    IF v_role != 'AICTE_ADMIN' AND public.get_auth_institution_id() != v_issue.institution_id THEN
        RAISE EXCEPTION 'Unauthorized to resolve this issue' USING ERRCODE = '42501';
    END IF;

    UPDATE public.validation_issues
    SET status = p_status,
        resolution_note = p_note,
        resolved_by = v_user_id,
        resolved_at = timezone('utc'::text, now())
    WHERE id = p_issue_id;

    -- Audit event
    INSERT INTO public.audit_events (
        actor_id, actor_role, action, entity_type, entity_id, institution_id, outcome, summary
    ) VALUES (
        v_user_id, v_role, 'issue.resolve', 'validation_issue',
        p_issue_id::text, v_issue.institution_id, 'SUCCESS',
        format('%s issue %s with note: %s', p_status, v_issue.rule_code, p_note)
    );

    RETURN jsonb_build_object(
        'id', p_issue_id,
        'status', p_status,
        'resolved_at', timezone('utc'::text, now())
    );
END;
$$;
