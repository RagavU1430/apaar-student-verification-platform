use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::config::Config;
use crate::domain::intake::{compute_intake_usage, IntakeUsage};
use crate::error::AppResult;
use crate::repositories::{admissions, institutions};
use crate::services::validation::tallies_from_counts;

/// Recomputes intake usage for every configured programme/year.
///
/// When `institution_id` is `None` the calculation runs across all
/// institutions, which is what the AICTE dashboard needs.
pub async fn monitor(
    pool: &PgPool,
    config: &Config,
    institution_id: Option<Uuid>,
) -> AppResult<Vec<IntakeUsage>> {
    let intakes = match institution_id {
        Some(id) => institutions::list_intakes(pool, id).await?,
        None => institutions::list_intakes_all(pool).await?,
    };

    let rules = institutions::intake_rules(pool, &config.demo_rule_version).await?;
    let rule_map: HashMap<String, (bool, bool)> = rules
        .into_iter()
        .map(|r| {
            (
                r.seat_category,
                (r.counts_toward_regular, r.uses_supernumerary_pool),
            )
        })
        .collect();

    let tallies = admissions::active_tallies(pool, institution_id).await?;
    let mut grouped: HashMap<(Uuid, String), Vec<(String, i64)>> = HashMap::new();
    for (programme_id, year, category, count) in tallies {
        grouped
            .entry((programme_id, year))
            .or_default()
            .push((category, count));
    }

    let mut out = Vec::new();
    for intake in intakes {
        let counts = grouped
            .get(&(intake.programme_id, intake.academic_year.clone()))
            .cloned()
            .unwrap_or_default();
        let tallies = tallies_from_counts(&counts, &rule_map);

        out.push(compute_intake_usage(
            &intake.institution_id.to_string(),
            &intake.programme_id.to_string(),
            &intake.programme_code,
            &intake.academic_year,
            &intake.rule_version,
            intake.regular_intake as i64,
            intake.supernumerary_capacity as i64,
            &tallies,
        ));
    }

    Ok(out)
}

/// Intake usage for a single programme and year.
#[allow(clippy::too_many_arguments)]
pub async fn usage_for(
    pool: &PgPool,
    config: &Config,
    institution_id: Uuid,
    programme_id: Uuid,
    programme_code: &str,
    academic_year: &str,
    regular_intake: i64,
    supernumerary_capacity: i64,
) -> AppResult<IntakeUsage> {
    let rules = institutions::intake_rules(pool, &config.demo_rule_version).await?;
    let rule_map: HashMap<String, (bool, bool)> = rules
        .into_iter()
        .map(|r| {
            (
                r.seat_category,
                (r.counts_toward_regular, r.uses_supernumerary_pool),
            )
        })
        .collect();

    let counts = crate::services::validation::category_counts(
        pool,
        programme_id,
        academic_year,
        Some(institution_id),
    )
    .await?;
    let tallies = tallies_from_counts(&counts, &rule_map);

    Ok(compute_intake_usage(
        &institution_id.to_string(),
        &programme_id.to_string(),
        programme_code,
        academic_year,
        &config.demo_rule_version,
        regular_intake,
        supernumerary_capacity,
        &tallies,
    ))
}
