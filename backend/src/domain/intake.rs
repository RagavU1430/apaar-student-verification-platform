use serde::Serialize;

/// Per-seat-category counting configuration, versioned with the rule set.
///
/// `counts_toward_regular` decides whether a category consumes the regular
/// approved intake. `uses_supernumerary_pool` decides whether it consumes the
/// separately configured supernumerary capacity. Both come from configuration,
/// never from hard-coded policy (BUILD_PLAN.md A-04).
#[derive(Clone, Debug, Serialize)]
pub struct SeatRule {
    pub seat_category: String,
    pub counts_toward_regular: bool,
    pub uses_supernumerary_pool: bool,
    pub notes: Option<String>,
}

/// Tally of active admissions for one seat category.
#[derive(Clone, Debug)]
pub struct CategoryTally {
    pub seat_category: String,
    pub count: i64,
    pub counts_toward_regular: bool,
    pub uses_supernumerary_pool: bool,
}

/// Result of applying the counting rule to one programme/year.
#[derive(Clone, Debug, Serialize)]
pub struct IntakeUsage {
    pub institution_id: String,
    pub programme_id: String,
    pub programme_code: String,
    pub academic_year: String,
    pub rule_version: String,
    /// Active admissions counted against the regular capacity.
    pub regular_count: i64,
    /// Active admissions counted against the supernumerary pool.
    pub supernumerary_count: i64,
    /// Total active admissions under the rule.
    pub active_count: i64,
    /// Configured regular capacity.
    pub regular_intake: i64,
    /// Configured supernumerary capacity (never merged into regular seats).
    pub supernumerary_capacity: i64,
    /// regular_count - regular_intake, floored at 0.
    pub excess_over_regular: i64,
    /// supernumerary_count - supernumerary_capacity, floored at 0.
    pub excess_over_supernumerary: i64,
    /// True when the regular capacity is exceeded.
    pub over_capacity: bool,
    /// True when the supernumerary pool is exceeded.
    pub supernumerary_over_capacity: bool,
    /// True when no capacity is configured at all for the programme/year.
    pub capacity_unconfigured: bool,
    pub by_category: Vec<CategoryCount>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CategoryCount {
    pub seat_category: String,
    pub count: i64,
    pub counts_toward_regular: bool,
    pub uses_supernumerary_pool: bool,
    pub rule_version: String,
}

/// Applies the demo counting rule (BUILD_PLAN.md section 7.3).
///
/// Count active admissions grouped by institution + programme + academic year,
/// compare the categories that count towards regular seats against the
/// configured regular intake, and report supernumerary capacity separately.
/// This is a demo rule, not an official AICTE formula.
pub fn compute_intake_usage(
    institution_id: &str,
    programme_id: &str,
    programme_code: &str,
    academic_year: &str,
    rule_version: &str,
    regular_intake: i64,
    supernumerary_capacity: i64,
    tallies: &[CategoryTally],
) -> IntakeUsage {
    let regular_count: i64 = tallies
        .iter()
        .filter(|t| t.counts_toward_regular)
        .map(|t| t.count)
        .sum();
    let supernumerary_count: i64 = tallies
        .iter()
        .filter(|t| !t.counts_toward_regular && t.uses_supernumerary_pool)
        .map(|t| t.count)
        .sum();
    let active_count: i64 = tallies.iter().map(|t| t.count).sum();

    let capacity_unconfigured = regular_intake <= 0 && supernumerary_capacity <= 0;
    let over_capacity = regular_intake > 0 && regular_count > regular_intake;
    let supernumerary_over_capacity =
        supernumerary_capacity > 0 && supernumerary_count > supernumerary_capacity;

    IntakeUsage {
        institution_id: institution_id.to_string(),
        programme_id: programme_id.to_string(),
        programme_code: programme_code.to_string(),
        academic_year: academic_year.to_string(),
        rule_version: rule_version.to_string(),
        regular_count,
        supernumerary_count,
        active_count,
        regular_intake,
        supernumerary_capacity,
        excess_over_regular: (regular_count - regular_intake).max(0),
        excess_over_supernumerary: (supernumerary_count - supernumerary_capacity).max(0),
        over_capacity,
        supernumerary_over_capacity,
        capacity_unconfigured,
        by_category: tallies
            .iter()
            .map(|t| CategoryCount {
                seat_category: t.seat_category.clone(),
                count: t.count,
                counts_toward_regular: t.counts_toward_regular,
                uses_supernumerary_pool: t.uses_supernumerary_pool,
                rule_version: rule_version.to_string(),
            })
            .collect(),
    }
}

/// Utilisation ratio against regular intake.
///
/// Returns `None` when capacity is not configured, so the UI can show an
/// explicit "not configured" state rather than a misleading 0%.
pub fn intake_utilisation(regular_count: i64, regular_intake: i64) -> Option<f64> {
    if regular_intake <= 0 {
        return None;
    }
    Some(regular_count as f64 / regular_intake as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tally(cat: &str, count: i64, regular: bool, supern: bool) -> CategoryTally {
        CategoryTally {
            seat_category: cat.to_string(),
            count,
            counts_toward_regular: regular,
            uses_supernumerary_pool: supern,
        }
    }

    #[test]
    fn regular_and_supernumerary_pools_are_never_merged() {
        let usage = compute_intake_usage(
            "inst-1",
            "prog-1",
            "CSE",
            "2026-27",
            "demo-v1",
            60,
            6,
            &[
                tally("REGULAR", 55, true, false),
                tally("SUPERNUMERARY", 4, false, true),
                tally("NRI", 3, false, true),
            ],
        );
        assert_eq!(usage.regular_count, 55);
        assert_eq!(usage.supernumerary_count, 7);
        assert_eq!(usage.active_count, 62);
        assert!(!usage.over_capacity, "55 regular against 60 seats is not an exception");
        assert!(usage.supernumerary_over_capacity, "7 supernumerary against 6 seats");
        assert_eq!(usage.excess_over_regular, 0);
        assert_eq!(usage.excess_over_supernumerary, 1);
    }

    #[test]
    fn regular_overrun_is_an_exception() {
        let usage = compute_intake_usage(
            "inst-1",
            "prog-1",
            "CSE",
            "2026-27",
            "demo-v1",
            60,
            6,
            &[tally("REGULAR", 63, true, false)],
        );
        assert!(usage.over_capacity);
        assert_eq!(usage.excess_over_regular, 3);
    }

    #[test]
    fn unconfigured_capacity_is_explicit_and_not_an_exception() {
        let usage = compute_intake_usage(
            "inst-1",
            "prog-1",
            "CSE",
            "2026-27",
            "demo-v1",
            0,
            0,
            &[tally("REGULAR", 3, true, false)],
        );
        assert!(usage.capacity_unconfigured);
        assert!(!usage.over_capacity);
        assert!(intake_utilisation(3, 0).is_none());
    }

    #[test]
    fn within_capacity_is_not_an_exception() {
        let usage = compute_intake_usage(
            "inst-1",
            "prog-1",
            "CSE",
            "2026-27",
            "demo-v1",
            60,
            6,
            &[tally("REGULAR", 60, true, false)],
        );
        assert!(!usage.over_capacity);
        assert_eq!(intake_utilisation(60, 60), Some(1.0));
    }
}
