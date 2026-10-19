// ==============================================================================
// AICTE APAAR Platform - Verification Service & Purpose-Limited Disclosure
// Direct TypeScript port of `backend/src/domain/verification.rs`
// ==============================================================================

import type { DisclosedField, VerificationResponse } from './types';

export interface PurposePolicy {
  purpose: string;
  label: string;
  description: string;
  allowedFields: string[];
}

export const PURPOSE_POLICIES: Record<string, PurposePolicy> = {
  ENROLMENT_STATUS: {
    purpose: 'ENROLMENT_STATUS',
    label: 'Current Enrolment Status Check',
    description: 'Verify if the student is currently enrolled in an approved institution.',
    allowedFields: ['institution_code', 'programme_code', 'academic_year', 'admission_status']
  },
  DEGREE_ELIGIBILITY: {
    purpose: 'DEGREE_ELIGIBILITY',
    label: 'Degree & Qualification Eligibility Verification',
    description: 'Verify programme level, academic year, and completion/enrollment for certificate issuance.',
    allowedFields: ['institution_code', 'programme_code', 'programme_level', 'academic_year', 'admission_status']
  },
  EMPLOYMENT_BACKGROUND: {
    purpose: 'EMPLOYMENT_BACKGROUND',
    label: 'Employer / Background Screening',
    description: 'Confirmation of institutional affiliation and academic tenure without disclosing personal details.',
    allowedFields: ['institution_code', 'programme_code', 'academic_year', 'admission_date']
  }
};

export const VERIFICATION_LIMITATIONS = [
  'Simulated adapter: no authoritative government API or live DigiLocker/APAAR gateway was contacted.',
  'Data reflects platform-ingested institutional submissions under demo counting rule demo-v1.',
  'Format check passed does NOT constitute confirmed identity verification.',
  'Field disclosure is strictly restricted to the declared, authenticated purpose.'
];

export function executeSimulatedVerification(
  purpose: string,
  _recordRef: string,
  matchedAdmission?: {
    institution_code: string;
    programme_code: string;
    programme_level?: string;
    academic_year: string;
    admission_date: string;
    admission_status: string;
  }
): VerificationResponse {
  const policy = PURPOSE_POLICIES[purpose] ?? PURPOSE_POLICIES.ENROLMENT_STATUS;
  const verifiedAt = new Date().toISOString();

  if (!matchedAdmission) {
    return {
      outcome: 'NOT_FOUND',
      fields_disclosed: [],
      source: 'simulated-apaar-adapter',
      verified_at: verifiedAt,
      limitations: VERIFICATION_LIMITATIONS,
      simulated: true
    };
  }

  const fields: DisclosedField[] = [];
  for (const fieldName of policy.allowedFields) {
    if (fieldName in matchedAdmission) {
      fields.push({
        name: fieldName,
        value: String((matchedAdmission as Record<string, string>)[fieldName]),
        justification: `Permitted under purpose policy: ${policy.label}`
      });
    }
  }

  const outcome = matchedAdmission.admission_status === 'ACTIVE' || matchedAdmission.admission_status === 'ENROLLED'
    ? 'VERIFIED'
    : 'NEEDS_REVIEW';

  return {
    outcome,
    fields_disclosed: fields,
    source: 'simulated-apaar-adapter (demo)',
    verified_at: verifiedAt,
    limitations: VERIFICATION_LIMITATIONS,
    simulated: true
  };
}
