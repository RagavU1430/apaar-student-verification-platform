use crate::domain::admission::normalise_apaar_id;
use crate::domain::validation::CandidateRow;
use std::collections::HashMap;

/// APAAR ids that appear more than once inside a single batch.
///
/// Returns the row numbers so the finding can point at the other rows.
pub fn in_batch_duplicates(rows: &[CandidateRow]) -> HashMap<String, Vec<usize>> {
    let mut seen: HashMap<String, Vec<usize>> = HashMap::new();
    for row in rows {
        let key = normalise_apaar_id(&row.apaar_id);
        if key.is_empty() {
            continue;
        }
        seen.entry(key).or_default().push(row.row_number);
    }
    seen.into_iter().filter(|(_, r)| r.len() > 1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(n: usize, apaar: &str) -> CandidateRow {
        CandidateRow {
            row_number: n,
            apaar_id: apaar.into(),
            institution_student_id: format!("S{n}"),
            full_name: None,
            programme_code: "CSE-01".into(),
            academic_year: "2026-27".into(),
            admission_date: "2026-07-01".into(),
            seat_category: "REGULAR".into(),
            admission_status: "ACTIVE".into(),
        }
    }

    #[test]
    fn detects_exact_collisions_with_row_numbers() {
        let rows = vec![row(2, "APAAR-1"), row(3, "apaar-1"), row(4, "APAAR-2")];
        let dups = in_batch_duplicates(&rows);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups["APAAR-1"], vec![2, 3]);
    }

    #[test]
    fn ignores_empty_ids() {
        let rows = vec![row(2, ""), row(3, "  ")];
        assert!(in_batch_duplicates(&rows).is_empty());
    }
}
