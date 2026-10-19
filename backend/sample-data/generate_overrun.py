"""Generates the intake-overrun sample batch.

The CSE-01 demo programme has 60 regular seats for 2026-27. This writes 65
distinct regular admissions so that committing the batch exceeds the approved
intake and raises an `INTAKE_CAPACITY_EXCEEDED` exception.

    python backend/sample-data/generate_overrun.py

All identifiers are synthetic and clearly marked as demo data.
"""

import csv
import pathlib

HERE = pathlib.Path(__file__).parent
COLUMNS = [
    "apaar_id",
    "institution_student_id",
    "full_name",
    "programme_code",
    "academic_year",
    "admission_date",
    "seat_category",
    "admission_status",
]


def main() -> None:
    rows = []
    for index in range(1, 66):
        rows.append(
            {
                "apaar_id": f"APAAR-OVER-{index:04d}",
                "institution_student_id": f"ALPHA-OVR-{index:04d}",
                "full_name": f"Demo Student {index:03d}",
                "programme_code": "CSE-01",
                "academic_year": "2026-27",
                "admission_date": "2026-07-15",
                "seat_category": "REGULAR",
                "admission_status": "ACTIVE",
            }
        )

    target = HERE / "intake_overrun.csv"
    with target.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=COLUMNS)
        writer.writeheader()
        writer.writerows(rows)

    print(f"wrote {len(rows)} rows to {target}")


if __name__ == "__main__":
    main()
