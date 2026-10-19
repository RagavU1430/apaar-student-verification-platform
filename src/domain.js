export const ROLES = Object.freeze({
  INSTITUTION_ADMIN: "institution_admin",
  INSTITUTION_OPERATOR: "institution_operator",
  AICTE_ADMIN: "aicte_admin",
  VERIFIER: "authorized_verifier"
});

export const ACTIVE_STATUSES = new Set(["active", "accepted"]);

const REQUIRED_COLUMNS = [
  "institution_code",
  "programme_code",
  "academic_year",
  "apaar_id",
  "institution_student_id",
  "admission_date",
  "admission_status",
  "seat_category"
];

const APAAR_FORMAT = /^[A-Z0-9]{8,32}$/;
const DATE_FORMAT = /^\d{4}-\d{2}-\d{2}$/;

export function parseCsv(text, maxRows = 5000) {
  if (typeof text !== "string" || text.length === 0) {
    return { headers: [], rows: [], fileError: "CSV file is empty." };
  }
  if (text.length > 2_000_000) {
    return { headers: [], rows: [], fileError: "CSV file exceeds the 2 MB prototype limit." };
  }

  const lines = text.replace(/^\uFEFF/, "").split(/\r?\n/).filter((line) => line.trim() !== "");
  if (lines.length < 2) {
    return { headers: [], rows: [], fileError: "CSV must contain a header and at least one data row." };
  }
  if (lines.length - 1 > maxRows) {
    return { headers: [], rows: [], fileError: `CSV exceeds the ${maxRows}-row prototype limit.` };
  }

  const headers = splitCsvLine(lines[0]).map((header) => header.trim().toLowerCase());
  const missing = REQUIRED_COLUMNS.filter((column) => !headers.includes(column));
  if (missing.length > 0) {
    return { headers, rows: [], fileError: `Missing required columns: ${missing.join(", ")}.` };
  }

  const rows = lines.slice(1).map((line, index) => {
    const values = splitCsvLine(line);
    const row = { row_number: index + 2 };
    headers.forEach((header, columnIndex) => {
      row[header] = (values[columnIndex] ?? "").trim();
    });
    return row;
  });
  return { headers, rows, fileError: null };
}

function splitCsvLine(line) {
  const values = [];
  let value = "";
  let quoted = false;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (character === '"') {
      if (quoted && line[index + 1] === '"') {
        value += '"';
        index += 1;
      } else {
        quoted = !quoted;
      }
    } else if (character === "," && !quoted) {
      values.push(value);
      value = "";
    } else {
      value += character;
    }
  }
  values.push(value);
  return values;
}

export function validateRows(rows, context) {
  const issues = [];
  const candidates = [];
  const seenInBatch = new Map();

  for (const row of rows) {
    const rowIssues = [];
    const programme = context.programmes.find(
      (item) => item.institution_code === row.institution_code && item.code === row.programme_code
    );

    if (row.institution_code !== context.institution.code) {
      rowIssues.push(issue(row, "INSTITUTION_SCOPE", "error", "Record belongs to a different institution."));
    }
    if (!programme) {
      rowIssues.push(issue(row, "PROGRAMME_NOT_FOUND", "error", "Programme is not configured for this institution."));
    }
    if (!row.apaar_id) {
      rowIssues.push(issue(row, "APAAR_REQUIRED", "error", "APAAR ID is required."));
    } else if (!APAAR_FORMAT.test(row.apaar_id)) {
      rowIssues.push(issue(row, "APAAR_FORMAT", "error", "APAAR ID failed the prototype format check; this is not identity verification."));
    }
    if (!row.institution_student_id) {
      rowIssues.push(issue(row, "STUDENT_ID_REQUIRED", "error", "Institution student ID is required."));
    }
    if (!DATE_FORMAT.test(row.admission_date) || Number.isNaN(Date.parse(row.admission_date))) {
      rowIssues.push(issue(row, "ADMISSION_DATE", "error", "Admission date must use YYYY-MM-DD."));
    }
    if (!context.allowedStatuses.has(row.admission_status)) {
      rowIssues.push(issue(row, "ADMISSION_STATUS", "error", "Admission status is not supported."));
    }
    if (!context.allowedSeatCategories.has(row.seat_category)) {
      rowIssues.push(issue(row, "SEAT_CATEGORY", "error", "Seat category is not configured."));
    }

    if (row.apaar_id) {
      const prior = seenInBatch.get(row.apaar_id);
      if (prior) {
        rowIssues.push(issue(row, "DUPLICATE_IN_BATCH", "error", `APAAR ID also appears on row ${prior}.`));
      } else {
        seenInBatch.set(row.apaar_id, row.row_number);
      }
      const existing = context.admissions.filter(
        (admission) => admission.apaar_id === row.apaar_id && ACTIVE_STATUSES.has(admission.admission_status)
      );
      if (existing.length > 0) {
        rowIssues.push(issue(row, "ACTIVE_DUPLICATE", "review", "APAAR ID is already linked to an active admission; human review is required."));
      }
    }

    candidates.push({
      ...row,
      status: rowIssues.some((item) => item.severity === "error")
        ? "rejected"
        : rowIssues.length > 0
          ? "review"
          : "accepted",
      issues: rowIssues
    });
    issues.push(...rowIssues);
  }

  const activeByProgramme = new Map();
  for (const candidate of candidates.filter((item) => item.status === "accepted" && ACTIVE_STATUSES.has(item.admission_status))) {
    const key = `${candidate.institution_code}:${candidate.programme_code}:${candidate.academic_year}`;
    activeByProgramme.set(key, (activeByProgramme.get(key) ?? 0) + 1);
  }
  for (const [key, count] of activeByProgramme) {
    const [institutionCode, programmeCode, academicYear] = key.split(":");
    const programme = context.programmes.find((item) => item.institution_code === institutionCode && item.code === programmeCode);
    const intake = context.intakes.find((item) => item.institution_code === institutionCode && item.programme_code === programmeCode && item.academic_year === academicYear);
    if (programme && intake && count > intake.regular_intake) {
      const excess = count - intake.regular_intake;
      for (const candidate of candidates.filter((item) => item.programme_code === programmeCode && item.academic_year === academicYear && item.status === "accepted")) {
        candidate.status = "review";
        const capacityIssue = issue(candidate, "INTAKE_EXCEEDED", "review", `Configured demo intake exceeded by ${excess}; regular intake is ${intake.regular_intake}.`);
        candidate.issues.push(capacityIssue);
        issues.push(capacityIssue);
      }
    }
  }

  return { candidates, issues };
}

function issue(row, code, severity, message) {
  return { row_number: row.row_number, code, severity, message };
}

export function summarize(candidates, issues) {
  return {
    total: candidates.length,
    accepted: candidates.filter((row) => row.status === "accepted").length,
    rejected: candidates.filter((row) => row.status === "rejected").length,
    review: candidates.filter((row) => row.status === "review").length,
    errors: issues.filter((item) => item.severity === "error").length,
    warnings: issues.filter((item) => item.severity !== "error").length
  };
}
