// ==============================================================================
// AICTE APAAR Platform - Unified Repository / Service Layer
// Connects to Supabase when configured, or provides high-fidelity local state.
// ==============================================================================

import { supabase, isSupabaseConfigured } from './supabase';
import type {
  UserProfile,
  Institution,
  Programme,
  ApprovedIntake,
  IntakeRule,
  SubmissionBatch,
  ValidationIssue,
  Admission,
  InstitutionDashboardData,
  AicteDashboardData,
  AuditEvent,
  VerificationRequestPayload,
  VerificationResponse
} from './types';
import { parseCsv, validateRows, maskApaarId, SCHEMA_VERSION, DEMO_RULE_VERSION } from './validation';
import { computeIntakeCard } from './intake';
import { executeSimulatedVerification } from './verification';

// --- SEED MASTER DATA ---
const SEED_INSTITUTIONS: Institution[] = [
  {
    id: '11111111-1111-4111-8111-111111111111',
    official_code: 'DEMO-INST-001',
    name: 'Alpha Institute of Technology',
    status: 'ACTIVE',
    created_at: new Date().toISOString()
  },
  {
    id: '22222222-2222-4222-8222-222222222222',
    official_code: 'DEMO-INST-002',
    name: 'Beta Institute of Engineering',
    status: 'ACTIVE',
    created_at: new Date().toISOString()
  }
];

const SEED_PROGRAMMES: Programme[] = [
  {
    id: 'aaaaaaaa-0000-4000-8000-000000000001',
    institution_id: '11111111-1111-4111-8111-111111111111',
    code: 'CSE-01',
    name: 'B.Tech Computer Science and Engineering',
    level: 'UG',
    status: 'ACTIVE',
    created_at: new Date().toISOString()
  },
  {
    id: 'aaaaaaaa-0000-4000-8000-000000000002',
    institution_id: '11111111-1111-4111-8111-111111111111',
    code: 'ECE-01',
    name: 'B.Tech Electronics and Communication',
    level: 'UG',
    status: 'ACTIVE',
    created_at: new Date().toISOString()
  },
  {
    id: 'bbbbbbbb-0000-4000-8000-000000000001',
    institution_id: '22222222-2222-4222-8222-222222222222',
    code: 'MECH-01',
    name: 'B.Tech Mechanical Engineering',
    level: 'UG',
    status: 'ACTIVE',
    created_at: new Date().toISOString()
  }
];

const SEED_INTAKES: ApprovedIntake[] = [
  {
    id: 'cccccccc-0000-4000-8000-000000000001',
    programme_id: 'aaaaaaaa-0000-4000-8000-000000000001',
    academic_year: '2026-27',
    regular_intake: 60,
    supernumerary_capacity: 6,
    source_reference: 'DEMO-SEED-v1',
    rule_version: 'demo-v1',
    effective_version: 1
  },
  {
    id: 'cccccccc-0000-4000-8000-000000000002',
    programme_id: 'aaaaaaaa-0000-4000-8000-000000000002',
    academic_year: '2026-27',
    regular_intake: 40,
    supernumerary_capacity: 4,
    source_reference: 'DEMO-SEED-v1',
    rule_version: 'demo-v1',
    effective_version: 1
  },
  {
    id: 'cccccccc-0000-4000-8000-000000000003',
    programme_id: 'bbbbbbbb-0000-4000-8000-000000000001',
    academic_year: '2026-27',
    regular_intake: 45,
    supernumerary_capacity: 4,
    source_reference: 'DEMO-SEED-v1',
    rule_version: 'demo-v1',
    effective_version: 1
  }
];

const SEED_INTAKE_RULES: IntakeRule[] = [
  {
    id: 'dddddddd-0000-4000-8000-000000000001',
    rule_version: 'demo-v1',
    seat_category: 'REGULAR',
    counts_toward_regular: true,
    uses_supernumerary_pool: false,
    notes: 'Counts against regular approved intake.'
  },
  {
    id: 'dddddddd-0000-4000-8000-000000000002',
    rule_version: 'demo-v1',
    seat_category: 'SUPERNUMERARY',
    counts_toward_regular: false,
    uses_supernumerary_pool: true,
    notes: 'Counts against supernumerary pool.'
  },
  {
    id: 'dddddddd-0000-4000-8000-000000000003',
    rule_version: 'demo-v1',
    seat_category: 'NRI',
    counts_toward_regular: false,
    uses_supernumerary_pool: true,
    notes: 'Demo rule: supernumerary treatment.'
  },
  {
    id: 'dddddddd-0000-4000-8000-000000000004',
    rule_version: 'demo-v1',
    seat_category: 'OCI',
    counts_toward_regular: false,
    uses_supernumerary_pool: true,
    notes: 'Demo rule: supernumerary treatment.'
  },
  {
    id: 'dddddddd-0000-4000-8000-000000000005',
    rule_version: 'demo-v1',
    seat_category: 'FOREIGN_NATIONAL',
    counts_toward_regular: false,
    uses_supernumerary_pool: true,
    notes: 'Demo rule: supernumerary treatment.'
  }
];

// Pre-configured demo user accounts
export const DEMO_USERS: Record<string, UserProfile> = {
  'aicte.admin': {
    id: '00000000-0000-0000-0000-000000000001',
    login_identifier: 'aicte.admin',
    display_name: 'AICTE Administrator',
    role: 'AICTE_ADMIN',
    institution_id: null,
    institution_name: null,
    permissions: [
      'dashboard:aicte',
      'batches:read_all',
      'issues:resolve',
      'intake:monitor',
      'audit:read',
      'reports:enrolment'
    ]
  },
  'alpha.admin': {
    id: '00000000-0000-0000-0000-000000000002',
    login_identifier: 'alpha.admin',
    display_name: 'Alpha Inst Admin',
    role: 'INSTITUTION_ADMIN',
    institution_id: '11111111-1111-4111-8111-111111111111',
    institution_name: 'Alpha Institute of Technology',
    permissions: [
      'dashboard:institution',
      'batch:upload',
      'batch:commit',
      'issues:read',
      'issues:resolve',
      'admissions:read'
    ]
  },
  'alpha.operator': {
    id: '00000000-0000-0000-0000-000000000003',
    login_identifier: 'alpha.operator',
    display_name: 'Alpha Inst Data Operator',
    role: 'INSTITUTION_DATA_OPERATOR',
    institution_id: '11111111-1111-4111-8111-111111111111',
    institution_name: 'Alpha Institute of Technology',
    permissions: [
      'dashboard:institution',
      'batch:upload',
      'batch:commit',
      'issues:read',
      'admissions:read'
    ]
  },
  'beta.operator': {
    id: '00000000-0000-0000-0000-000000000004',
    login_identifier: 'beta.operator',
    display_name: 'Beta Inst Data Operator',
    role: 'INSTITUTION_DATA_OPERATOR',
    institution_id: '22222222-2222-4222-8222-222222222222',
    institution_name: 'Beta Institute of Engineering',
    permissions: [
      'dashboard:institution',
      'batch:upload',
      'batch:commit',
      'issues:read',
      'admissions:read'
    ]
  },
  verifier: {
    id: '00000000-0000-0000-0000-000000000005',
    login_identifier: 'verifier',
    display_name: 'Authorized Verifier',
    role: 'AUTHORIZED_VERIFIER',
    institution_id: null,
    institution_name: null,
    permissions: ['verification:request']
  },
  auditor: {
    id: '00000000-0000-0000-0000-000000000006',
    login_identifier: 'auditor',
    display_name: 'Auditor',
    role: 'AUDITOR',
    institution_id: null,
    institution_name: null,
    permissions: ['audit:read']
  }
};

// In-memory / localStorage storage key
const STORAGE_PREFIX = 'apaar_local_store_v1_';

function getStored<T>(key: string, defaultValue: T): T {
  try {
    const raw = localStorage.getItem(STORAGE_PREFIX + key);
    return raw ? JSON.parse(raw) : defaultValue;
  } catch {
    return defaultValue;
  }
}

function setStored<T>(key: string, value: T): void {
  try {
    localStorage.setItem(STORAGE_PREFIX + key, JSON.stringify(value));
  } catch {
    // Ignore storage errors in demo
  }
}

// In-memory state containers with localStorage persistence
class LocalState {
  institutions: Institution[] = getStored('institutions', SEED_INSTITUTIONS);
  programmes: Programme[] = getStored('programmes', SEED_PROGRAMMES);
  intakes: ApprovedIntake[] = getStored('intakes', SEED_INTAKES);
  intakeRules: IntakeRule[] = getStored('rules', SEED_INTAKE_RULES);
  batches: SubmissionBatch[] = getStored('batches', []);
  issues: ValidationIssue[] = getStored('issues', []);
  admissions: Admission[] = getStored('admissions', []);
  stagingRows: Array<any> = getStored('staging_rows', []);
  auditEvents: AuditEvent[] = getStored('audit_events', []);
  currentUser: UserProfile | null = getStored('current_user', DEMO_USERS['alpha.admin']);

  save() {
    setStored('institutions', this.institutions);
    setStored('programmes', this.programmes);
    setStored('intakes', this.intakes);
    setStored('batches', this.batches);
    setStored('issues', this.issues);
    setStored('admissions', this.admissions);
    setStored('staging_rows', this.stagingRows);
    setStored('audit_events', this.auditEvents);
    setStored('current_user', this.currentUser);
  }

  recordAudit(action: string, entityType: string, entityId?: string | null, instId?: string | null, outcome = 'SUCCESS', summary?: string) {
    const event: AuditEvent = {
      id: crypto.randomUUID(),
      actor_id: this.currentUser?.id || null,
      actor_role: this.currentUser?.role || 'SYSTEM',
      action,
      entity_type: entityType,
      entity_id: entityId || null,
      institution_id: instId || this.currentUser?.institution_id || null,
      outcome,
      summary: summary || null,
      created_at: new Date().toISOString()
    };
    this.auditEvents.unshift(event);
    this.save();
  }
}

const local = new LocalState();

// --- SERVICE IMPLEMENTATION ---

export const repository = {
  isLiveSupabase: () => isSupabaseConfigured,

  // --- AUTHENTICATION ---
  async login(loginIdentifier: string, password: string): Promise<UserProfile> {
    if (isSupabaseConfigured && supabase) {
      const email = loginIdentifier.includes('@') ? loginIdentifier : `${loginIdentifier}@apaar.gov.in`;
      const { data, error } = await supabase.auth.signInWithPassword({ email, password });
      if (error) throw new Error(error.message);

      const { data: profile } = await supabase
        .from('profiles')
        .select('*, institutions(name)')
        .eq('id', data.user.id)
        .single();

      if (!profile) throw new Error('Profile not found.');

      const user: UserProfile = {
        id: profile.id,
        login_identifier: profile.login_identifier,
        display_name: profile.display_name,
        role: profile.role,
        institution_id: profile.institution_id,
        institution_name: profile.institutions?.name || null,
        permissions: getPermissionsForRole(profile.role)
      };
      local.currentUser = user;
      local.save();
      return user;
    }

    // Local / Prototype Authentication
    const demo = DEMO_USERS[loginIdentifier.toLowerCase().trim()];
    if (!demo || password !== 'DemoPass123!') {
      throw new Error("Invalid credentials. Try 'alpha.admin' or 'aicte.admin' with password 'DemoPass123!'");
    }

    local.currentUser = demo;
    local.recordAudit('auth.login', 'user', demo.id, demo.institution_id, 'SUCCESS', `User ${demo.login_identifier} logged in.`);
    local.save();
    return demo;
  },

  async getCurrentUser(): Promise<UserProfile | null> {
    if (isSupabaseConfigured && supabase) {
      const { data } = await supabase.auth.getSession();
      if (!data.session?.user) return null;
      const { data: profile } = await supabase
        .from('profiles')
        .select('*, institutions(name)')
        .eq('id', data.session.user.id)
        .single();
      if (!profile) return null;
      return {
        id: profile.id,
        login_identifier: profile.login_identifier,
        display_name: profile.display_name,
        role: profile.role,
        institution_id: profile.institution_id,
        institution_name: profile.institutions?.name || null,
        permissions: getPermissionsForRole(profile.role)
      };
    }
    return local.currentUser;
  },

  async logout(): Promise<void> {
    if (isSupabaseConfigured && supabase) {
      await supabase.auth.signOut();
    }
    if (local.currentUser) {
      local.recordAudit('auth.logout', 'user', local.currentUser.id, local.currentUser.institution_id, 'SUCCESS');
    }
    local.currentUser = null;
    local.save();
  },

  // --- INSTITUTION DASHBOARD ---
  async getInstitutionDashboard(institutionId?: string): Promise<InstitutionDashboardData> {
    const user = local.currentUser;
    const instId = institutionId || user?.institution_id || SEED_INSTITUTIONS[0].id;
    const inst = local.institutions.find((i) => i.id === instId) || SEED_INSTITUTIONS[0];

    const instBatches = local.batches.filter((b) => b.institution_id === instId);
    const instIssues = local.issues.filter((i) => i.institution_id === instId && i.status === 'OPEN');
    const instAdmissions = local.admissions.filter((a) => a.institution_id === instId && (a.status === 'ACTIVE' || a.status === 'ENROLLED'));
    const instProgrammes = local.programmes.filter((p) => p.institution_id === instId);

    // Compute intake cards
    const intakeCards = instProgrammes.map((prog) => {
      const intake = local.intakes.find((i) => i.programme_id === prog.id && i.academic_year === '2026-27');
      const tallies = [
        {
          seat_category: 'REGULAR',
          count: instAdmissions.filter((a) => a.programme_code === prog.code && a.seat_category === 'REGULAR').length
        },
        {
          seat_category: 'SUPERNUMERARY',
          count: instAdmissions.filter((a) => a.programme_code === prog.code && a.seat_category === 'SUPERNUMERARY').length
        },
        {
          seat_category: 'NRI',
          count: instAdmissions.filter((a) => a.programme_code === prog.code && a.seat_category === 'NRI').length
        },
        {
          seat_category: 'OCI',
          count: instAdmissions.filter((a) => a.programme_code === prog.code && a.seat_category === 'OCI').length
        },
        {
          seat_category: 'FOREIGN_NATIONAL',
          count: instAdmissions.filter((a) => a.programme_code === prog.code && a.seat_category === 'FOREIGN_NATIONAL').length
        }
      ];

      return computeIntakeCard(
        prog.code,
        prog.name,
        '2026-27',
        intake?.regular_intake || 0,
        intake?.supernumerary_capacity || 0,
        intake?.source_reference || 'DEMO-SEED-v1',
        DEMO_RULE_VERSION,
        tallies,
        local.intakeRules
      );
    });

    const totalRows = instBatches.reduce((acc, b) => acc + b.row_count, 0);
    const acceptedRows = instBatches.reduce((acc, b) => acc + b.accepted_count, 0);
    const rejectedRows = instBatches.reduce((acc, b) => acc + b.rejected_count, 0);
    const rowsNeedingReview = instBatches.reduce((acc, b) => acc + b.needs_review_count, 0);
    const duplicateCandidates = instBatches.reduce((acc, b) => acc + b.duplicate_count, 0);

    return {
      institution_id: instId,
      institution_name: inst.name,
      rule_version: DEMO_RULE_VERSION,
      data_source: isSupabaseConfigured ? 'Supabase PostgreSQL (Live)' : 'Synthetic Platform Records (Demo)',
      batches: instBatches.slice(0, 10).map((b) => ({
        id: b.id,
        file_name: b.file_name,
        status: b.status,
        submitted_at: b.submitted_at,
        rows: b.row_count,
        accepted: b.accepted_count,
        rejected: b.rejected_count,
        needs_review: b.needs_review_count
      })),
      intake: intakeCards,
      open_issues: instIssues.length,
      counts: {
        total_batches: instBatches.length,
        total_rows: totalRows,
        accepted_rows: acceptedRows,
        rejected_rows: rejectedRows,
        rows_needing_review: rowsNeedingReview,
        duplicate_candidates: duplicateCandidates
      }
    };
  },

  // --- AICTE DASHBOARD ---
  async getAicteDashboard(): Promise<AicteDashboardData> {
    const instList = local.institutions.map((inst) => {
      const activeAdmissions = local.admissions.filter(
        (a) => a.institution_id === inst.id && (a.status === 'ACTIVE' || a.status === 'ENROLLED')
      ).length;
      return {
        id: inst.id,
        name: inst.name,
        official_code: inst.official_code,
        active_admissions: activeAdmissions
      };
    });

    const openIssues = local.issues
      .filter((i) => i.status === 'OPEN')
      .slice(0, 50)
      .map((i) => {
        const inst = local.institutions.find((x) => x.id === i.institution_id);
        return {
          issue_id: i.id,
          institution_id: i.institution_id,
          institution_name: inst?.name,
          row_reference: i.row_reference,
          rule_code: i.rule_code,
          message: i.message,
          evidence: i.evidence,
          rule_version: i.rule_version,
          severity: i.severity
        };
      });

    // Cross-institution intake cards
    const intakeCards = local.programmes.map((prog) => {
      const intake = local.intakes.find((i) => i.programme_id === prog.id && i.academic_year === '2026-27');
      const tallies = [
        {
          seat_category: 'REGULAR',
          count: local.admissions.filter((a) => a.programme_code === prog.code && a.seat_category === 'REGULAR' && (a.status === 'ACTIVE' || a.status === 'ENROLLED')).length
        },
        {
          seat_category: 'SUPERNUMERARY',
          count: local.admissions.filter((a) => a.programme_code === prog.code && a.seat_category === 'SUPERNUMERARY' && (a.status === 'ACTIVE' || a.status === 'ENROLLED')).length
        }
      ];

      return computeIntakeCard(
        prog.code,
        prog.name,
        '2026-27',
        intake?.regular_intake || 0,
        intake?.supernumerary_capacity || 0,
        intake?.source_reference || 'DEMO-SEED-v1',
        DEMO_RULE_VERSION,
        tallies,
        local.intakeRules
      );
    });

    const totalRows = local.batches.reduce((acc, b) => acc + b.row_count, 0);
    const acceptedRows = local.batches.reduce((acc, b) => acc + b.accepted_count, 0);
    const rejectedRows = local.batches.reduce((acc, b) => acc + b.rejected_count, 0);
    const rowsNeedingReview = local.batches.reduce((acc, b) => acc + b.needs_review_count, 0);
    const duplicateCandidates = local.batches.reduce((acc, b) => acc + b.duplicate_count, 0);

    return {
      rule_version: DEMO_RULE_VERSION,
      data_source: isSupabaseConfigured ? 'Supabase PostgreSQL (Live)' : 'Synthetic Platform Records (Demo)',
      institutions: instList,
      intake: intakeCards,
      exception_queue: openIssues,
      counts: {
        total_batches: local.batches.length,
        total_rows: totalRows,
        accepted_rows: acceptedRows,
        rejected_rows: rejectedRows,
        rows_needing_review: rowsNeedingReview,
        duplicate_candidates: duplicateCandidates
      }
    };
  },

  // --- BATCH MANAGEMENT ---
  async listBatches(): Promise<SubmissionBatch[]> {
    const user = local.currentUser;
    if (user?.role === 'AICTE_ADMIN') {
      return [...local.batches];
    }
    return local.batches.filter((b) => b.institution_id === user?.institution_id);
  },

  async getBatch(id: string): Promise<SubmissionBatch> {
    const batch = local.batches.find((b) => b.id === id);
    if (!batch) throw new Error('Batch not found.');
    return batch;
  },

  async getBatchIssues(batchId: string): Promise<ValidationIssue[]> {
    return local.issues.filter((i) => i.batch_id === batchId);
  },

  async uploadBatch(file: File, _idempotencyKey: string): Promise<SubmissionBatch> {
    const user = local.currentUser;
    const institutionId = user?.institution_id || SEED_INSTITUTIONS[0].id;
    const inst = local.institutions.find((i) => i.id === institutionId);

    const text = await file.text();
    const { rows, fileError } = parseCsv(text);
    if (fileError) {
      throw new Error(fileError);
    }

    const availableProgrammes = local.programmes.filter((p) => p.institution_id === institutionId);
    const availableIntakes = local.intakes.filter((i) =>
      availableProgrammes.some((p) => p.id === i.programme_id)
    );

    // Cross-institution active APAAR lookup
    const otherInstAdmissions = local.admissions.filter(
      (a) => a.institution_id !== institutionId && (a.status === 'ACTIVE' || a.status === 'ENROLLED')
    );
    const crossInstApaar = new Set(otherInstAdmissions.map((a) => a.apaar_id_masked));

    const { outcomes, counts } = validateRows(rows, {
      institutionId,
      programmes: availableProgrammes,
      approvedIntakes: availableIntakes,
      crossInstitutionActiveApaarIds: crossInstApaar,
      ruleVersion: DEMO_RULE_VERSION
    });

    const batchId = crypto.randomUUID();
    const newBatch: SubmissionBatch = {
      id: batchId,
      institution_id: institutionId,
      institution_name: inst?.name,
      submitted_by: user?.id,
      file_name: file.name,
      file_hash: `sha256-${Date.now()}`,
      schema_version: SCHEMA_VERSION,
      status: 'VALIDATED',
      row_count: counts.rows,
      accepted_count: counts.accepted + counts.accepted_with_warning,
      warning_count: counts.accepted_with_warning,
      duplicate_count: counts.duplicates,
      rejected_count: counts.rejected,
      needs_review_count: counts.needs_review,
      submitted_at: new Date().toISOString(),
      committed_at: null,
      rule_version: DEMO_RULE_VERSION,
      external_verification: 'simulated'
    };

    local.batches.unshift(newBatch);

    // Persist issues
    for (const outcome of outcomes) {
      for (const finding of outcome.findings) {
        local.issues.push({
          id: crypto.randomUUID(),
          batch_id: batchId,
          institution_id: institutionId,
          row_reference: `row ${outcome.row_number}`,
          rule_code: finding.rule_code,
          severity: finding.severity,
          message: finding.message,
          field: finding.field || null,
          evidence: finding.evidence || null,
          rule_version: finding.rule_version,
          status: 'OPEN',
          resolution_note: null,
          resolved_at: null,
          created_at: new Date().toISOString()
        });
      }

      // Store in staging if acceptable
      if (outcome.status === 'ACCEPTED' || outcome.status === 'ACCEPTED_WITH_WARNING') {
        local.stagingRows.push({
          id: crypto.randomUUID(),
          batch_id: batchId,
          ...outcome.candidate,
          programme_id: outcome.programme_id
        });
      }
    }

    local.recordAudit(
      'batch.upload',
      'submission_batch',
      batchId,
      institutionId,
      'SUCCESS',
      `Uploaded ${file.name} with ${counts.rows} rows: ${counts.accepted} accepted, ${counts.rejected} rejected.`
    );
    local.save();

    return newBatch;
  },

  async commitBatch(batchId: string): Promise<{
    batch_id: string;
    status: string;
    committed: number;
    skipped: number;
    intake_exceptions: Array<{ programme_code: string; academic_year: string; message: string }>;
  }> {
    const batch = local.batches.find((b) => b.id === batchId);
    if (!batch) throw new Error('Batch not found.');
    if (batch.status === 'COMMITTED') throw new Error('Batch is already committed.');

    const rowsToCommit = local.stagingRows.filter((r) => r.batch_id === batchId);
    let committed = 0;
    let skipped = 0;

    for (const row of rowsToCommit) {
      // Check duplicate active admission
      const alreadyActive = local.admissions.some(
        (a) =>
          a.institution_id === batch.institution_id &&
          a.programme_code === row.programme_code &&
          a.academic_year === row.academic_year &&
          a.institution_student_id === row.institution_student_id &&
          (a.status === 'ACTIVE' || a.status === 'ENROLLED')
      );

      if (alreadyActive) {
        skipped++;
        local.issues.push({
          id: crypto.randomUUID(),
          batch_id: batchId,
          institution_id: batch.institution_id,
          row_reference: `row ${row.row_number}`,
          rule_code: 'DUPLICATE_ACTIVE_ADMISSION',
          severity: 'REVIEW',
          message: 'Skipped at commit: the student already has an active admission for this programme and year.',
          field: 'apaar_id',
          evidence: maskApaarId(row.apaar_id),
          rule_version: DEMO_RULE_VERSION,
          status: 'OPEN',
          resolution_note: null,
          resolved_at: null,
          created_at: new Date().toISOString()
        });
        continue;
      }

      local.admissions.push({
        id: crypto.randomUUID(),
        institution_id: batch.institution_id,
        programme_code: row.programme_code,
        academic_year: row.academic_year,
        admission_date: row.admission_date,
        status: row.admission_status,
        seat_category: row.seat_category,
        institution_student_id: row.institution_student_id,
        full_name: row.full_name || null,
        apaar_id_masked: maskApaarId(row.apaar_id)
      });
      committed++;
    }

    batch.status = 'COMMITTED';
    batch.committed_at = new Date().toISOString();
    local.stagingRows = local.stagingRows.filter((r) => r.batch_id !== batchId);

    // Compute intake overruns
    const exceptions: Array<{ programme_code: string; academic_year: string; message: string }> = [];
    const progsInBatch = Array.from(new Set(rowsToCommit.map((r) => r.programme_code)));

    for (const pCode of progsInBatch) {
      const prog = local.programmes.find((p) => p.code === pCode && p.institution_id === batch.institution_id);
      if (!prog) continue;

      const intake = local.intakes.find((i) => i.programme_id === prog.id && i.academic_year === '2026-27');
      if (intake) {
        const regularCount = local.admissions.filter(
          (a) => a.institution_id === batch.institution_id && a.programme_code === pCode && a.seat_category === 'REGULAR' && (a.status === 'ACTIVE' || a.status === 'ENROLLED')
        ).length;

        if (regularCount > intake.regular_intake) {
          const excess = regularCount - intake.regular_intake;
          const msg = `Active regular admissions (${regularCount}) exceed configured regular intake (${intake.regular_intake}). Excess: ${excess}.`;
          exceptions.push({ programme_code: pCode, academic_year: '2026-27', message: msg });

          local.issues.push({
            id: crypto.randomUUID(),
            batch_id: batchId,
            institution_id: batch.institution_id,
            row_reference: `${pCode} / 2026-27`,
            rule_code: 'INTAKE_CAPACITY_EXCEEDED',
            severity: 'REVIEW',
            message: msg,
            field: null,
            evidence: `rule demo-v1 | counted ${regularCount} regular seats against capacity ${intake.regular_intake}`,
            rule_version: DEMO_RULE_VERSION,
            status: 'OPEN',
            resolution_note: null,
            resolved_at: null,
            created_at: new Date().toISOString()
          });
        }
      }
    }

    local.recordAudit(
      'batch.commit',
      'submission_batch',
      batchId,
      batch.institution_id,
      'SUCCESS',
      `Committed ${committed} records, skipped ${skipped}. Exceptions: ${exceptions.length}`
    );
    local.save();

    return {
      batch_id: batchId,
      status: 'COMMITTED',
      committed,
      skipped,
      intake_exceptions: exceptions
    };
  },

  // --- ISSUES ---
  async listOpenIssues(): Promise<ValidationIssue[]> {
    const user = local.currentUser;
    if (user?.role === 'AICTE_ADMIN') {
      return local.issues.filter((i) => i.status === 'OPEN');
    }
    return local.issues.filter((i) => i.institution_id === user?.institution_id && i.status === 'OPEN');
  },

  async resolveIssue(id: string, status: 'RESOLVED' | 'DISMISSED', note: string): Promise<ValidationIssue> {
    const issue = local.issues.find((i) => i.id === id);
    if (!issue) throw new Error('Issue not found.');

    issue.status = status;
    issue.resolution_note = note;
    issue.resolved_by = local.currentUser?.id;
    issue.resolved_at = new Date().toISOString();

    local.recordAudit('issue.resolve', 'validation_issue', id, issue.institution_id, 'SUCCESS', `Issue marked as ${status}: ${note}`);
    local.save();
    return issue;
  },

  // --- ADMISSIONS / REGISTRY SEARCH ---
  async searchAdmissions(query: {
    search?: string;
    programmeCode?: string;
    academicYear?: string;
    seatCategory?: string;
    status?: string;
  }): Promise<{ items: Admission[]; total: number }> {
    const user = local.currentUser;
    let list = local.admissions;

    if (user?.role !== 'AICTE_ADMIN') {
      list = list.filter((a) => a.institution_id === user?.institution_id);
    }

    if (query.programmeCode) {
      list = list.filter((a) => a.programme_code.toLowerCase() === query.programmeCode?.toLowerCase());
    }
    if (query.academicYear) {
      list = list.filter((a) => a.academic_year === query.academicYear);
    }
    if (query.seatCategory) {
      list = list.filter((a) => a.seat_category === query.seatCategory);
    }
    if (query.status) {
      list = list.filter((a) => a.status === query.status);
    }
    if (query.search) {
      const q = query.search.toLowerCase().trim();
      list = list.filter(
        (a) =>
          a.institution_student_id.toLowerCase().includes(q) ||
          (a.full_name && a.full_name.toLowerCase().includes(q)) ||
          a.apaar_id_masked.toLowerCase().includes(q)
      );
    }

    return { items: list, total: list.length };
  },

  // --- VERIFICATION SERVICE ---
  async verifyRecord(payload: VerificationRequestPayload): Promise<VerificationResponse> {
    const cleanRef = payload.record_ref.trim();
    // Look up in admissions
    const matched = local.admissions.find(
      (a) =>
        a.institution_student_id.toLowerCase() === cleanRef.toLowerCase() ||
        a.apaar_id_masked.toLowerCase() === cleanRef.toLowerCase()
    );

    const inst = matched ? local.institutions.find((i) => i.id === matched.institution_id) : undefined;

    const response = executeSimulatedVerification(
      payload.purpose,
      cleanRef,
      matched
        ? {
            institution_code: inst?.official_code || 'DEMO-INST',
            programme_code: matched.programme_code,
            academic_year: matched.academic_year,
            admission_date: matched.admission_date,
            admission_status: matched.status
          }
        : undefined
    );

    local.recordAudit(
      'verification.request',
      'verification_request',
      null,
      matched?.institution_id || null,
      response.outcome === 'NOT_FOUND' ? 'DENIED' : 'SUCCESS',
      `Verification for purpose '${payload.purpose}', result: ${response.outcome}`
    );

    return response;
  },

  // --- AUDIT TRAIL ---
  async getAuditLogs(): Promise<AuditEvent[]> {
    const user = local.currentUser;
    if (user?.role === 'AICTE_ADMIN' || user?.role === 'AUDITOR') {
      return [...local.auditEvents];
    }
    return local.auditEvents.filter((a) => a.institution_id === user?.institution_id);
  },

  // --- ENROLMENT REPORT ---
  async getEnrolmentReport(academicYear = '2026-27'): Promise<Array<{
    institution_id: string;
    institution_name: string;
    programme_code: string;
    academic_year: string;
    seat_category: string;
    count: number;
  }>> {
    const report: Array<any> = [];
    for (const prog of local.programmes) {
      const inst = local.institutions.find((i) => i.id === prog.institution_id);
      const categories = ['REGULAR', 'SUPERNUMERARY', 'NRI', 'OCI', 'FOREIGN_NATIONAL'];

      for (const cat of categories) {
        const count = local.admissions.filter(
          (a) =>
            a.institution_id === prog.institution_id &&
            a.programme_code === prog.code &&
            a.academic_year === academicYear &&
            a.seat_category === cat &&
            (a.status === 'ACTIVE' || a.status === 'ENROLLED')
        ).length;

        if (count > 0) {
          report.push({
            institution_id: prog.institution_id,
            institution_name: inst?.name || 'Institution',
            programme_code: prog.code,
            academic_year: academicYear,
            seat_category: cat,
            count
          });
        }
      }
    }
    return report;
  },

  // Download template
  downloadTemplateCsv(): string {
    return (
      'apaar_id,institution_student_id,full_name,programme_code,academic_year,admission_date,seat_category,admission_status\n' +
      'APAAR-DEMO-0000001,ALPHA-0001,Asha Rao,CSE-01,2026-27,2026-07-01,REGULAR,ACTIVE\n' +
      'APAAR-DEMO-0000002,ALPHA-0002,Bejoy Iyer,CSE-01,2026-27,2026-07-03,REGULAR,ACTIVE\n'
    );
  }
};

function getPermissionsForRole(role: string): string[] {
  switch (role) {
    case 'AICTE_ADMIN':
      return [
        'dashboard:aicte',
        'batches:read_all',
        'issues:resolve',
        'intake:monitor',
        'audit:read',
        'reports:enrolment'
      ];
    case 'INSTITUTION_ADMIN':
      return [
        'dashboard:institution',
        'batch:upload',
        'batch:commit',
        'issues:read',
        'issues:resolve',
        'admissions:read'
      ];
    case 'INSTITUTION_DATA_OPERATOR':
      return ['dashboard:institution', 'batch:upload', 'batch:commit', 'issues:read', 'admissions:read'];
    case 'AUTHORIZED_VERIFIER':
      return ['verification:request'];
    case 'AUDITOR':
      return ['audit:read'];
    default:
      return [];
  }
}
