// ==============================================================================
// AICTE APAAR Platform - Intake Monitoring & Counting Rules
// Direct TypeScript port of `backend/src/domain/intake.rs`
// ==============================================================================

import type { IntakeCard, IntakeRule } from './types';

export interface CategoryTally {
  seat_category: string;
  count: number;
}

export function computeIntakeCard(
  programmeCode: string,
  programmeName: string | null,
  academicYear: string,
  regularIntake: number,
  supernumeraryCapacity: number,
  sourceReference: string | null,
  ruleVersion: string,
  tallies: CategoryTally[],
  rules: IntakeRule[]
): IntakeCard {
  const ruleMap = new Map<string, { countsTowardRegular: boolean; usesSupernumerary: boolean }>();
  for (const r of rules) {
    ruleMap.set(r.seat_category, {
      countsTowardRegular: r.counts_toward_regular,
      usesSupernumerary: r.uses_supernumerary_pool
    });
  }

  let regularCount = 0;
  let supernumeraryCount = 0;

  for (const tally of tallies) {
    const config = ruleMap.get(tally.seat_category) ?? {
      // Default fallback rule: REGULAR counts towards regular, everything else supernumerary
      countsTowardRegular: tally.seat_category === 'REGULAR',
      usesSupernumerary: tally.seat_category !== 'REGULAR'
    };

    if (config.countsTowardRegular) {
      regularCount += tally.count;
    } else if (config.usesSupernumerary) {
      supernumeraryCount += tally.count;
    }
  }

  const utilisation = regularIntake > 0 ? (regularCount / regularIntake) * 100 : null;
  const overCapacity =
    (regularIntake > 0 && regularCount > regularIntake) ||
    (supernumeraryCapacity > 0 && supernumeraryCount > supernumeraryCapacity);

  return {
    programme_code: programmeCode,
    programme_name: programmeName,
    academic_year: academicYear,
    rule_version: ruleVersion,
    regular_count: regularCount,
    regular_intake: regularIntake,
    supernumerary_count: supernumeraryCount,
    supernumerary_capacity: supernumeraryCapacity,
    utilisation: utilisation !== null ? Math.round(utilisation * 10) / 10 : null,
    over_capacity: overCapacity,
    source_reference: sourceReference
  };
}
