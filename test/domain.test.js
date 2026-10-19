import test from "node:test";
import assert from "node:assert/strict";
import { parseCsv, summarize, validateRows } from "../src/domain.js";

const context = {
  institution: { code: "INST-001" },
  programmes: [{ institution_code: "INST-001", code: "CSE" }],
  intakes: [{ institution_code: "INST-001", programme_code: "CSE", academic_year: "2026-27", regular_intake: 1 }],
  admissions: [],
  allowedStatuses: new Set(["active", "withdrawn", "completed", "transferred"]),
  allowedSeatCategories: new Set(["regular", "supernumerary", "nri", "oci", "foreign_national"])
};

test("parses required CSV columns and rows", () => {
  const result = parseCsv("institution_code,programme_code,academic_year,apaar_id,institution_student_id,admission_date,admission_status,seat_category\nINST-001,CSE,2026-27,ABCD1234,S-1,2026-07-01,active,regular");
  assert.equal(result.fileError, null);
  assert.equal(result.rows.length, 1);
});

test("flags malformed APAAR values without claiming identity verification", () => {
  const result = validateRows([{ row_number: 2, institution_code: "INST-001", programme_code: "CSE", academic_year: "2026-27", apaar_id: "bad", institution_student_id: "S-1", admission_date: "2026-07-01", admission_status: "active", seat_category: "regular" }], context);
  assert.equal(result.issues[0].code, "APAAR_FORMAT");
  assert.match(result.issues[0].message, /not identity verification/);
});

test("detects duplicate rows", () => {
  const rows = ["ABCD1234", "ABCD1234"].map((apaar_id, index) => ({ row_number: index + 2, institution_code: "INST-001", programme_code: "CSE", academic_year: "2026-27", apaar_id, institution_student_id: `S-${index}`, admission_date: "2026-07-01", admission_status: "active", seat_category: "regular" }));
  const result = validateRows(rows, context);
  assert.ok(result.issues.some((item) => item.code === "DUPLICATE_IN_BATCH"));
  assert.deepEqual(summarize(result.candidates, result.issues), { total: 2, accepted: 1, rejected: 1, review: 0, errors: 1, warnings: 0 });
});

test("detects configured intake overrun for accepted active records", () => {
  const rows = ["ABCD1234", "EFGH5678"].map((apaar_id, index) => ({ row_number: index + 2, institution_code: "INST-001", programme_code: "CSE", academic_year: "2026-27", apaar_id, institution_student_id: `S-${index}`, admission_date: "2026-07-01", admission_status: "active", seat_category: "regular" }));
  const result = validateRows(rows, context);
  assert.ok(result.issues.some((item) => item.code === "INTAKE_EXCEEDED"));
  assert.equal(result.candidates.every((candidate) => candidate.status === "review"), true);
});
