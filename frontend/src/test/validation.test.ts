import { describe, it, expect } from 'vitest';
import { parseCsv, validateRows, maskApaarId, normaliseApaarId } from '../lib/validation';
import { computeIntakeCard } from '../lib/intake';
import { executeSimulatedVerification } from '../lib/verification';
import type { Programme, ApprovedIntake, IntakeRule } from '../lib/types';

describe('CSV Parsing & Ingestion Limits', () => {
  it('parses valid CSV data into candidate rows', () => {
    const csv =
      'apaar_id,institution_student_id,full_name,programme_code,academic_year,admission_date,seat_category,admission_status\n' +
      'APAAR-DEMO-0001,ALPHA-0001,Asha Rao,CSE-01,2026-27,2026-07-01,REGULAR,ACTIVE\n';

    const { rows, fileError } = parseCsv(csv);
    expect(fileError).toBeNull();
    expect(rows).toHaveLength(1);
    expect(rows[0].apaar_id).toBe('APAAR-DEMO-0001');
    expect(rows[0].seat_category).toBe('REGULAR');
  });

  it('rejects files missing required columns', () => {
    const csv = 'apaar_id,institution_student_id\nAPAAR-01,ALPHA-01\n';
    const { rows, fileError } = parseCsv(csv);
    expect(rows).toHaveLength(0);
    expect(fileError).toContain('Missing required column(s)');
  });

  it('rejects empty CSV text', () => {
    const { rows, fileError } = parseCsv('');
    expect(rows).toHaveLength(0);
    expect(fileError).toBe('The CSV file is empty.');
  });
});

describe('Validation Engine & Rule Codes', () => {
  const dummyProgrammes: Programme[] = [
    {
      id: 'p1',
      institution_id: 'i1',
      code: 'CSE-01',
      name: 'Computer Science',
      level: 'UG',
      status: 'ACTIVE',
      created_at: new Date().toISOString()
    }
  ];

  const dummyIntakes: ApprovedIntake[] = [
    {
      id: 'int1',
      programme_id: 'p1',
      academic_year: '2026-27',
      regular_intake: 60,
      supernumerary_capacity: 6,
      source_reference: 'DEMO-SEED-v1',
      rule_version: 'demo-v1',
      effective_version: 1
    }
  ];

  it('identifies missing APAAR ID and invalid date format', () => {
    const rows = [
      {
        row_number: 2,
        apaar_id: '',
        institution_student_id: 'S01',
        full_name: 'Test Student',
        programme_code: 'CSE-01',
        academic_year: '2026-27',
        admission_date: '01-07-2026', // invalid date format DD-MM-YYYY
        seat_category: 'REGULAR',
        admission_status: 'ACTIVE'
      }
    ];

    const { outcomes, counts } = validateRows(rows, {
      institutionId: 'i1',
      programmes: dummyProgrammes,
      approvedIntakes: dummyIntakes
    });

    expect(counts.rejected).toBe(1);
    const codes = outcomes[0].findings.map((f) => f.rule_code);
    expect(codes).toContain('APAAR_ID_MISSING');
    expect(codes).toContain('ADMISSION_DATE_INVALID');
  });

  it('detects duplicate APAAR IDs within the same batch', () => {
    const rows = [
      {
        row_number: 2,
        apaar_id: 'APAAR-DEMO-0001',
        institution_student_id: 'S01',
        full_name: 'Asha Rao',
        programme_code: 'CSE-01',
        academic_year: '2026-27',
        admission_date: '2026-07-01',
        seat_category: 'REGULAR',
        admission_status: 'ACTIVE'
      },
      {
        row_number: 3,
        apaar_id: 'APAAR-DEMO-0001', // Duplicate in batch
        institution_student_id: 'S02',
        full_name: 'Chitra Menon',
        programme_code: 'CSE-01',
        academic_year: '2026-27',
        admission_date: '2026-07-02',
        seat_category: 'REGULAR',
        admission_status: 'ACTIVE'
      }
    ];

    const { outcomes, counts } = validateRows(rows, {
      institutionId: 'i1',
      programmes: dummyProgrammes,
      approvedIntakes: dummyIntakes
    });

    expect(counts.duplicates).toBe(1);
    expect(outcomes[1].status).toBe('NEEDS_REVIEW');
    expect(outcomes[1].findings.some((f) => f.rule_code === 'DUPLICATE_APAAR_IN_BATCH')).toBe(true);
  });

  it('masks APAAR IDs safely for list displays', () => {
    expect(maskApaarId('APAAR-DEMO-0001')).toBe('APAAR-***-0001');
    expect(normaliseApaarId('  apaar-1234  ')).toBe('APAAR-1234');
  });
});

describe('Intake Monitoring & Counting Rules', () => {
  const rules: IntakeRule[] = [
    {
      id: 'r1',
      rule_version: 'demo-v1',
      seat_category: 'REGULAR',
      counts_toward_regular: true,
      uses_supernumerary_pool: false
    },
    {
      id: 'r2',
      rule_version: 'demo-v1',
      seat_category: 'SUPERNUMERARY',
      counts_toward_regular: false,
      uses_supernumerary_pool: true
    }
  ];

  it('computes regular vs supernumerary utilization accurately', () => {
    const card = computeIntakeCard(
      'CSE-01',
      'Computer Science',
      '2026-27',
      60,
      6,
      'DEMO-SEED-v1',
      'demo-v1',
      [
        { seat_category: 'REGULAR', count: 30 },
        { seat_category: 'SUPERNUMERARY', count: 3 }
      ],
      rules
    );

    expect(card.regular_count).toBe(30);
    expect(card.supernumerary_count).toBe(3);
    expect(card.utilisation).toBe(50);
    expect(card.over_capacity).toBe(false);
  });

  it('flags capacity overrun when regular admissions exceed approved intake', () => {
    const card = computeIntakeCard(
      'CSE-01',
      'Computer Science',
      '2026-27',
      60,
      6,
      'DEMO-SEED-v1',
      'demo-v1',
      [
        { seat_category: 'REGULAR', count: 65 },
        { seat_category: 'SUPERNUMERARY', count: 2 }
      ],
      rules
    );

    expect(card.regular_count).toBe(65);
    expect(card.over_capacity).toBe(true);
    expect(card.utilisation).toBe(108.3);
  });
});

describe('Purpose-Limited Verification Service', () => {
  it('discloses only whitelisted fields for enrolment check', () => {
    const student = {
      institution_code: 'DEMO-INST-001',
      programme_code: 'CSE-01',
      academic_year: '2026-27',
      admission_date: '2026-07-01',
      admission_status: 'ACTIVE'
    };

    const res = executeSimulatedVerification('ENROLMENT_STATUS', 'ALPHA-0001', student);
    expect(res.outcome).toBe('VERIFIED');
    expect(res.simulated).toBe(true);

    const disclosedNames = res.fields_disclosed.map((f) => f.name);
    expect(disclosedNames).toContain('institution_code');
    expect(disclosedNames).toContain('programme_code');
    expect(disclosedNames).toContain('admission_status');
    // admission_date should NOT be disclosed under ENROLMENT_STATUS
    expect(disclosedNames).not.toContain('admission_date');
  });

  it('returns NOT_FOUND for unknown record', () => {
    const res = executeSimulatedVerification('ENROLMENT_STATUS', 'NONEXISTENT');
    expect(res.outcome).toBe('NOT_FOUND');
    expect(res.fields_disclosed).toHaveLength(0);
    expect(res.simulated).toBe(true);
  });
});
