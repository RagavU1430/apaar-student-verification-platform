// ==============================================================================
// AICTE APAAR Platform - Pure Validation Engine
// Direct TypeScript port of the domain rules from `backend/src/domain/validation.rs`
// ==============================================================================

import type {
  CandidateRow,
  RowOutcome,
  ValidationFinding,
  ValidationStatus,
  BatchCounts,
  Programme,
  ApprovedIntake,
  Admission
} from './types';

export const SCHEMA_VERSION = 'csv-v1';
export const DEMO_RULE_VERSION = 'demo-v1';

export const REQUIRED_COLUMNS = [
  'apaar_id',
  'institution_student_id',
  'full_name',
  'programme_code',
  'academic_year',
  'admission_date',
  'seat_category',
  'admission_status'
] as const;

export const ALLOWED_STATUSES = new Set([
  'ACTIVE',
  'ENROLLED',
  'WITHDRAWN',
  'COMPLETED',
  'TRANSFERRED',
  'CANCELLED'
]);

export const ACTIVE_STATUSES = new Set(['ACTIVE', 'ENROLLED']);

export const ALLOWED_SEAT_CATEGORIES = new Set([
  'REGULAR',
  'SUPERNUMERARY',
  'NRI',
  'OCI',
  'FOREIGN_NATIONAL'
]);

export const APAAR_PATTERN = /^[A-Z0-9-]{8,20}$/;
export const ACADEMIC_YEAR_PATTERN = /^\d{4}-\d{2}$/;
export const DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/;

export function normaliseApaarId(input: string): string {
  return input.trim().toUpperCase();
}

export function maskApaarId(apaarId: string): string {
  const norm = normaliseApaarId(apaarId);
  if (norm.length <= 4) return '***';
  return `APAAR-***-${norm.slice(-4)}`;
}

export interface ParseResult {
  rows: CandidateRow[];
  headers: string[];
  fileError: string | null;
}

export function parseCsv(text: string, maxRows = 5000): ParseResult {
  if (!text || text.trim().length === 0) {
    return { rows: [], headers: [], fileError: 'The CSV file is empty.' };
  }

  if (text.length > 5 * 1024 * 1024) {
    return { rows: [], headers: [], fileError: 'The file exceeds the 5 MB maximum upload limit.' };
  }

  const lines = text
    .replace(/^\uFEFF/, '') // Strip UTF-8 BOM
    .split(/\r?\n/)
    .filter((line) => line.trim().length > 0);

  if (lines.length < 2) {
    return {
      rows: [],
      headers: [],
      fileError: 'CSV must contain a header row and at least one data row.'
    };
  }

  const headerLine = lines[0];
  const headers = splitCsvLine(headerLine).map((h) => h.trim().toLowerCase());

  const missingColumns = REQUIRED_COLUMNS.filter((col) => !headers.includes(col));
  if (missingColumns.length > 0) {
    return {
      rows: [],
      headers,
      fileError: `Missing required column(s): ${missingColumns.join(', ')}. Expected: ${REQUIRED_COLUMNS.join(', ')}.`
    };
  }

  if (lines.length - 1 > maxRows) {
    return {
      rows: [],
      headers,
      fileError: `The file contains ${lines.length - 1} rows, exceeding the limit of ${maxRows} rows.`
    };
  }

  const rows: CandidateRow[] = [];
  for (let i = 1; i < lines.length; i++) {
    const rawValues = splitCsvLine(lines[i]);
    const rowMap: Record<string, string> = {};
    headers.forEach((header, index) => {
      rowMap[header] = (rawValues[index] ?? '').trim();
    });

    rows.push({
      row_number: i + 1, // 1-indexed, header is row 1
      apaar_id: rowMap['apaar_id'] || '',
      institution_student_id: rowMap['institution_student_id'] || '',
      full_name: rowMap['full_name'] || null,
      programme_code: rowMap['programme_code'] || '',
      academic_year: rowMap['academic_year'] || '',
      admission_date: rowMap['admission_date'] || '',
      seat_category: rowMap['seat_category']?.toUpperCase() || '',
      admission_status: rowMap['admission_status']?.toUpperCase() || ''
    });
  }

  return { rows, headers, fileError: null };
}

function splitCsvLine(line: string): string[] {
  const values: string[] = [];
  let current = '';
  let inQuotes = false;

  for (let i = 0; i < line.length; i++) {
    const char = line[i];
    if (char === '"') {
      if (inQuotes && line[i + 1] === '"') {
        current += '"';
        i++;
      } else {
        inQuotes = !inQuotes;
      }
    } else if (char === ',' && !inQuotes) {
      values.push(current);
      current = '';
    } else {
      current += char;
    }
  }
  values.push(current);
  return values;
}

export interface ValidationContext {
  institutionId: string;
  programmes: Programme[];
  approvedIntakes: ApprovedIntake[];
  existingAdmissions?: Admission[];
  crossInstitutionActiveApaarIds?: Set<string>;
  ruleVersion?: string;
}

export function validateRows(
  rows: CandidateRow[],
  context: ValidationContext
): { outcomes: RowOutcome[]; counts: BatchCounts } {
  const ruleVersion = context.ruleVersion || DEMO_RULE_VERSION;
  const outcomes: RowOutcome[] = [];
  const seenApaarInBatch = new Map<string, number>();

  for (const row of rows) {
    const findings: ValidationFinding[] = [];
    const normalisedApaar = normaliseApaarId(row.apaar_id);

    // 1. APAAR ID validation
    if (!normalisedApaar) {
      findings.push({
        rule_code: 'APAAR_ID_MISSING',
        severity: 'ERROR',
        message: 'APAAR ID is missing.',
        field: 'apaar_id',
        evidence: null,
        rule_version: ruleVersion
      });
    } else if (!APAAR_PATTERN.test(normalisedApaar)) {
      findings.push({
        rule_code: 'APAAR_ID_FORMAT_INVALID',
        severity: 'ERROR',
        message: 'APAAR ID does not match expected demo format. This is a format check only, not identity verification.',
        field: 'apaar_id',
        evidence: normalisedApaar,
        rule_version: ruleVersion
      });
    }

    // 2. Student ID validation
    if (!row.institution_student_id || row.institution_student_id.trim().length === 0) {
      findings.push({
        rule_code: 'STUDENT_ID_MISSING',
        severity: 'ERROR',
        message: 'Institution student ID is required.',
        field: 'institution_student_id',
        evidence: null,
        rule_version: ruleVersion
      });
    }

    // 3. Full Name
    if (!row.full_name || row.full_name.trim().length === 0) {
      findings.push({
        rule_code: 'FULL_NAME_MISSING',
        severity: 'WARNING',
        message: 'Student full name is missing.',
        field: 'full_name',
        evidence: null,
        rule_version: ruleVersion
      });
    }

    // 4. Programme Validation
    let matchedProgrammeId: string | undefined;
    if (!row.programme_code) {
      findings.push({
        rule_code: 'PROGRAMME_CODE_MISSING',
        severity: 'ERROR',
        message: 'Programme code is required.',
        field: 'programme_code',
        evidence: null,
        rule_version: ruleVersion
      });
    } else {
      const prog = context.programmes.find(
        (p) => p.code.toLowerCase() === row.programme_code.toLowerCase()
      );
      if (!prog) {
        findings.push({
          rule_code: 'PROGRAMME_UNKNOWN',
          severity: 'ERROR',
          message: `Programme '${row.programme_code}' is not configured for this institution.`,
          field: 'programme_code',
          evidence: row.programme_code,
          rule_version: ruleVersion
        });
      } else {
        matchedProgrammeId = prog.id;
      }
    }

    // 5. Academic Year Validation
    if (!row.academic_year) {
      findings.push({
        rule_code: 'ACADEMIC_YEAR_MISSING',
        severity: 'ERROR',
        message: 'Academic year is required.',
        field: 'academic_year',
        evidence: null,
        rule_version: ruleVersion
      });
    } else if (!ACADEMIC_YEAR_PATTERN.test(row.academic_year)) {
      findings.push({
        rule_code: 'ACADEMIC_YEAR_INVALID',
        severity: 'ERROR',
        message: "Academic year must match 'YYYY-YY' (e.g. 2026-27).",
        field: 'academic_year',
        evidence: row.academic_year,
        rule_version: ruleVersion
      });
    } else if (matchedProgrammeId) {
      const intake = context.approvedIntakes.find(
        (i) => i.programme_id === matchedProgrammeId && i.academic_year === row.academic_year
      );
      if (!intake) {
        findings.push({
          rule_code: 'ACADEMIC_YEAR_NOT_CONFIGURED',
          severity: 'WARNING',
          message: `No approved intake configured for programme ${row.programme_code} in year ${row.academic_year}.`,
          field: 'academic_year',
          evidence: `${row.programme_code} / ${row.academic_year}`,
          rule_version: ruleVersion
        });
      }
    }

    // 6. Admission Date Validation
    if (!row.admission_date) {
      findings.push({
        rule_code: 'ADMISSION_DATE_MISSING',
        severity: 'ERROR',
        message: 'Admission date is required.',
        field: 'admission_date',
        evidence: null,
        rule_version: ruleVersion
      });
    } else if (!DATE_PATTERN.test(row.admission_date) || Number.isNaN(Date.parse(row.admission_date))) {
      findings.push({
        rule_code: 'ADMISSION_DATE_INVALID',
        severity: 'ERROR',
        message: "Admission date must use 'YYYY-MM-DD' and represent a valid calendar date.",
        field: 'admission_date',
        evidence: row.admission_date,
        rule_version: ruleVersion
      });
    }

    // 7. Seat Category
    if (!ALLOWED_SEAT_CATEGORIES.has(row.seat_category)) {
      findings.push({
        rule_code: 'SEAT_CATEGORY_UNKNOWN',
        severity: 'ERROR',
        message: `Seat category '${row.seat_category}' is unrecognised. Allowed: ${Array.from(ALLOWED_SEAT_CATEGORIES).join(', ')}.`,
        field: 'seat_category',
        evidence: row.seat_category,
        rule_version: ruleVersion
      });
    }

    // 8. Admission Status
    if (!ALLOWED_STATUSES.has(row.admission_status)) {
      findings.push({
        rule_code: 'ADMISSION_STATUS_INVALID',
        severity: 'ERROR',
        message: `Admission status '${row.admission_status}' is unrecognised. Allowed: ${Array.from(ALLOWED_STATUSES).join(', ')}.`,
        field: 'admission_status',
        evidence: row.admission_status,
        rule_version: ruleVersion
      });
    }

    // 9. Uniqueness: In-batch duplicate APAAR ID
    if (normalisedApaar) {
      const priorRow = seenApaarInBatch.get(normalisedApaar);
      if (priorRow) {
        findings.push({
          rule_code: 'DUPLICATE_APAAR_IN_BATCH',
          severity: 'DUPLICATE',
          message: `Same APAAR ID also appears on row ${priorRow}.`,
          field: 'apaar_id',
          evidence: `Repeated on row ${priorRow}`,
          rule_version: ruleVersion
        });
      } else {
        seenApaarInBatch.set(normalisedApaar, row.row_number);
      }

      // Cross-institution conflict check
      if (context.crossInstitutionActiveApaarIds?.has(normalisedApaar)) {
        findings.push({
          rule_code: 'CROSS_INSTITUTION_ACTIVE_CONFLICT',
          severity: 'REVIEW',
          message: 'APAAR ID is linked to an active admission in another institution; requires regulator review.',
          field: 'apaar_id',
          evidence: maskApaarId(normalisedApaar),
          rule_version: ruleVersion
        });
      }
    }

    // Determine status from findings
    const status = deriveStatus(findings);

    outcomes.push({
      row_number: row.row_number,
      status,
      findings,
      candidate: row,
      programme_id: matchedProgrammeId
    });
  }

  const counts: BatchCounts = {
    rows: outcomes.length,
    accepted: outcomes.filter((o) => o.status === 'ACCEPTED').length,
    accepted_with_warning: outcomes.filter((o) => o.status === 'ACCEPTED_WITH_WARNING').length,
    needs_review: outcomes.filter((o) => o.status === 'NEEDS_REVIEW').length,
    duplicates: outcomes.filter((o) => o.findings.some((f) => f.severity === 'DUPLICATE')).length,
    rejected: outcomes.filter((o) => o.status === 'REJECTED').length
  };

  return { outcomes, counts };
}

function deriveStatus(findings: ValidationFinding[]): ValidationStatus {
  if (findings.some((f) => f.severity === 'ERROR')) {
    return 'REJECTED';
  }
  if (findings.some((f) => f.severity === 'REVIEW' || f.severity === 'DUPLICATE')) {
    return 'NEEDS_REVIEW';
  }
  if (findings.some((f) => f.severity === 'WARNING')) {
    return 'ACCEPTED_WITH_WARNING';
  }
  return 'ACCEPTED';
}
