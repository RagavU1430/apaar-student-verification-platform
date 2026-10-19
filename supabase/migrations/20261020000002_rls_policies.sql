-- ==============================================================================
-- AICTE APAAR Student Verification Platform - Row Level Security (RLS) Policies
-- Migration: 20261020000002_rls_policies.sql
-- ==============================================================================

-- 1. HELPER FUNCTIONS
CREATE OR REPLACE FUNCTION public.get_auth_role()
RETURNS TEXT
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = public
AS $$
  SELECT role FROM public.profiles WHERE id = auth.uid();
$$;

CREATE OR REPLACE FUNCTION public.get_auth_institution_id()
RETURNS UUID
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = public
AS $$
  SELECT institution_id FROM public.profiles WHERE id = auth.uid();
$$;

CREATE OR REPLACE FUNCTION public.is_aicte_admin()
RETURNS BOOLEAN
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = public
AS $$
  SELECT EXISTS (
    SELECT 1 FROM public.profiles
    WHERE id = auth.uid() AND role = 'AICTE_ADMIN'
  );
$$;

CREATE OR REPLACE FUNCTION public.is_auditor()
RETURNS BOOLEAN
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = public
AS $$
  SELECT EXISTS (
    SELECT 1 FROM public.profiles
    WHERE id = auth.uid() AND role IN ('AICTE_ADMIN', 'AUDITOR')
  );
$$;

-- 2. ENABLE RLS ON ALL RELEVANT TABLES
ALTER TABLE public.institutions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.programmes ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.approved_intakes ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.intake_rules ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.students ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.institution_students ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.submission_batches ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.staging_rows ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.admissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.validation_issues ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.verification_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.audit_events ENABLE ROW LEVEL SECURITY;

-- 3. POLICIES: institutions
-- Authenticated users can view institutions; only AICTE admin can insert/update.
CREATE POLICY "Institutions viewable by authenticated users"
    ON public.institutions FOR SELECT
    TO authenticated
    USING (true);

CREATE POLICY "Institutions manageable by AICTE Admin"
    ON public.institutions FOR ALL
    TO authenticated
    USING (public.is_aicte_admin())
    WITH CHECK (public.is_aicte_admin());

-- 4. POLICIES: profiles
-- Users can view their own profile; AICTE admins can view all profiles.
CREATE POLICY "Profiles viewable by self or AICTE Admin"
    ON public.profiles FOR SELECT
    TO authenticated
    USING (id = auth.uid() OR public.is_aicte_admin());

CREATE POLICY "Profiles updatable by self (display_name)"
    ON public.profiles FOR UPDATE
    TO authenticated
    USING (id = auth.uid())
    WITH CHECK (id = auth.uid());

-- 5. POLICIES: programmes
-- Programmes are viewable by authenticated users.
CREATE POLICY "Programmes viewable by authenticated"
    ON public.programmes FOR SELECT
    TO authenticated
    USING (true);

CREATE POLICY "Programmes manageable by AICTE or own institution admin"
    ON public.programmes FOR ALL
    TO authenticated
    USING (public.is_aicte_admin() OR (institution_id = public.get_auth_institution_id() AND public.get_auth_role() = 'INSTITUTION_ADMIN'))
    WITH CHECK (public.is_aicte_admin() OR (institution_id = public.get_auth_institution_id() AND public.get_auth_role() = 'INSTITUTION_ADMIN'));

-- 6. POLICIES: approved_intakes & intake_rules
CREATE POLICY "Approved intakes viewable by authenticated"
    ON public.approved_intakes FOR SELECT
    TO authenticated
    USING (true);

CREATE POLICY "Approved intakes manageable by AICTE admin"
    ON public.approved_intakes FOR ALL
    TO authenticated
    USING (public.is_aicte_admin())
    WITH CHECK (public.is_aicte_admin());

CREATE POLICY "Intake rules viewable by authenticated"
    ON public.intake_rules FOR SELECT
    TO authenticated
    USING (true);

CREATE POLICY "Intake rules manageable by AICTE admin"
    ON public.intake_rules FOR ALL
    TO authenticated
    USING (public.is_aicte_admin())
    WITH CHECK (public.is_aicte_admin());

-- 7. POLICIES: submission_batches
-- Institution users see their own batches; AICTE admin sees all.
CREATE POLICY "Batches viewable by owner institution or AICTE"
    ON public.submission_batches FOR SELECT
    TO authenticated
    USING (public.is_aicte_admin() OR institution_id = public.get_auth_institution_id());

CREATE POLICY "Batches creatable by own institution operators/admins"
    ON public.submission_batches FOR INSERT
    TO authenticated
    WITH CHECK (
        institution_id = public.get_auth_institution_id()
        AND public.get_auth_role() IN ('INSTITUTION_ADMIN', 'INSTITUTION_DATA_OPERATOR')
    );

CREATE POLICY "Batches updatable by own institution or AICTE"
    ON public.submission_batches FOR UPDATE
    TO authenticated
    USING (public.is_aicte_admin() OR institution_id = public.get_auth_institution_id())
    WITH CHECK (public.is_aicte_admin() OR institution_id = public.get_auth_institution_id());

-- 8. POLICIES: staging_rows
CREATE POLICY "Staging rows scoped to institution"
    ON public.staging_rows FOR SELECT
    TO authenticated
    USING (
        EXISTS (
            SELECT 1 FROM public.submission_batches b
            WHERE b.id = staging_rows.batch_id
              AND (public.is_aicte_admin() OR b.institution_id = public.get_auth_institution_id())
        )
    );

CREATE POLICY "Staging rows insertable by batch owner"
    ON public.staging_rows FOR INSERT
    TO authenticated
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM public.submission_batches b
            WHERE b.id = staging_rows.batch_id
              AND b.institution_id = public.get_auth_institution_id()
        )
    );

CREATE POLICY "Staging rows deletable by batch owner or AICTE"
    ON public.staging_rows FOR DELETE
    TO authenticated
    USING (
        EXISTS (
            SELECT 1 FROM public.submission_batches b
            WHERE b.id = staging_rows.batch_id
              AND (public.is_aicte_admin() OR b.institution_id = public.get_auth_institution_id())
        )
    );

-- 9. POLICIES: validation_issues
CREATE POLICY "Validation issues viewable by institution or AICTE"
    ON public.validation_issues FOR SELECT
    TO authenticated
    USING (public.is_aicte_admin() OR institution_id = public.get_auth_institution_id());

CREATE POLICY "Validation issues insertable by institution"
    ON public.validation_issues FOR INSERT
    TO authenticated
    WITH CHECK (institution_id = public.get_auth_institution_id() OR public.is_aicte_admin());

CREATE POLICY "Validation issues resolvable by authorized roles"
    ON public.validation_issues FOR UPDATE
    TO authenticated
    USING (public.is_aicte_admin() OR institution_id = public.get_auth_institution_id())
    WITH CHECK (public.is_aicte_admin() OR institution_id = public.get_auth_institution_id());

-- 10. POLICIES: institution_students & admissions
-- Institution isolation: an institution can NEVER view another institution's students or admissions.
CREATE POLICY "Institution students isolated by institution"
    ON public.institution_students FOR SELECT
    TO authenticated
    USING (public.is_aicte_admin() OR institution_id = public.get_auth_institution_id());

CREATE POLICY "Admissions isolated by institution"
    ON public.admissions FOR SELECT
    TO authenticated
    USING (public.is_aicte_admin() OR institution_id = public.get_auth_institution_id());

-- 11. POLICIES: verification_requests
CREATE POLICY "Verification requests viewable by requester or AICTE"
    ON public.verification_requests FOR SELECT
    TO authenticated
    USING (requester_id = auth.uid() OR public.is_aicte_admin());

CREATE POLICY "Verification requests creatable by authorized verifier"
    ON public.verification_requests FOR INSERT
    TO authenticated
    WITH CHECK (
        requester_id = auth.uid()
        AND public.get_auth_role() IN ('AUTHORIZED_VERIFIER', 'AICTE_ADMIN')
    );

-- 12. POLICIES: audit_events
-- Append-only. Readable by AICTE Admin and Auditor.
CREATE POLICY "Audit events readable by AICTE or Auditor"
    ON public.audit_events FOR SELECT
    TO authenticated
    USING (public.is_auditor() OR institution_id = public.get_auth_institution_id());

CREATE POLICY "Audit events insertable by authenticated actions"
    ON public.audit_events FOR INSERT
    TO authenticated
    WITH CHECK (true);
