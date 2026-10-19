// ==============================================================================
// AICTE APAAR Platform - Typed API Client Layer
// Bridges components to Supabase & the repository layer
// ==============================================================================

import { repository } from '../lib/repository';
import type {
  UserProfile,
  SubmissionBatch,
  ValidationIssue,
  InstitutionDashboardData,
  AicteDashboardData,
  Admission,
  VerificationRequestPayload,
  VerificationResponse,
  AuditEvent
} from '../lib/types';

export * from '../lib/types';

export class ApiError extends Error {
  code: string;
  status: number;

  constructor(code: string, message: string, status: number) {
    super(message);
    this.code = code;
    this.status = status;
  }
}

export const api = {
  // Auth
  login: (loginIdentifier: string, password: string): Promise<UserProfile> =>
    repository.login(loginIdentifier, password),

  me: (): Promise<UserProfile | null> => repository.getCurrentUser(),

  logout: (): Promise<void> => repository.logout(),

  // Dashboards
  institutionDashboard: (instId?: string): Promise<InstitutionDashboardData> =>
    repository.getInstitutionDashboard(instId),

  aicteDashboard: (): Promise<AicteDashboardData> => repository.getAicteDashboard(),

  // Batches
  batches: (): Promise<SubmissionBatch[]> => repository.listBatches(),

  batch: (id: string): Promise<SubmissionBatch> => repository.getBatch(id),

  batchIssues: (id: string): Promise<ValidationIssue[]> => repository.getBatchIssues(id),

  uploadBatch: (file: File, idempotencyKey: string): Promise<SubmissionBatch> =>
    repository.uploadBatch(file, idempotencyKey),

  commitBatch: (id: string) => repository.commitBatch(id),

  // Issues
  openIssues: (): Promise<ValidationIssue[]> => repository.listOpenIssues(),

  resolveIssue: (id: string, status: 'RESOLVED' | 'DISMISSED', note: string): Promise<ValidationIssue> =>
    repository.resolveIssue(id, status, note),

  // Admissions Registry
  admissions: (query: {
    search?: string;
    programmeCode?: string;
    academicYear?: string;
    seatCategory?: string;
    status?: string;
  }): Promise<{ items: Admission[]; total: number }> => repository.searchAdmissions(query),

  // Verifications
  verify: (payload: VerificationRequestPayload): Promise<VerificationResponse> =>
    repository.verifyRecord(payload),

  // Audit Log
  auditLogs: (): Promise<AuditEvent[]> => repository.getAuditLogs(),

  // Enrolment Report
  enrolmentReport: (academicYear?: string) => repository.getEnrolmentReport(academicYear),

  // Template
  downloadTemplate: () => repository.downloadTemplateCsv()
};

// Storage token helper for session management
const TOKEN_KEY = 'aicte-apaar.session';
export function getToken(): string | null {
  return sessionStorage.getItem(TOKEN_KEY) || localStorage.getItem(TOKEN_KEY);
}
export function setToken(token: string | null): void {
  if (token) {
    sessionStorage.setItem(TOKEN_KEY, token);
    localStorage.setItem(TOKEN_KEY, token);
  } else {
    sessionStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(TOKEN_KEY);
  }
}
