import { createHash, randomUUID } from "node:crypto";
import { createServer } from "node:http";
import { parseCsv, ROLES, summarize, validateRows } from "./domain.js";

const PORT = Number(process.env.PORT || 3000);
const store = {
  institutions: [
    { code: "INST-001", name: "Synthetic Institute of Technology", status: "active" },
    { code: "INST-002", name: "Synthetic College of Management", status: "active" }
  ],
  programmes: [
    { institution_code: "INST-001", code: "CSE", name: "Computer Science", level: "PGDM" },
    { institution_code: "INST-001", code: "ECE", name: "Electronics", level: "PGDM" },
    { institution_code: "INST-002", code: "MBA", name: "Business Administration", level: "MBA" }
  ],
  intakes: [
    { institution_code: "INST-001", programme_code: "CSE", academic_year: "2026-27", regular_intake: 2, rule_version: "demo-2026.1" },
    { institution_code: "INST-001", programme_code: "ECE", academic_year: "2026-27", regular_intake: 3, rule_version: "demo-2026.1" },
    { institution_code: "INST-002", programme_code: "MBA", academic_year: "2026-27", regular_intake: 2, rule_version: "demo-2026.1" }
  ],
  admissions: [],
  batches: [],
  audit: []
};

const sessions = new Map([
  ["demo-institution", { user_id: "user-inst-1", login: "institution.demo", role: ROLES.INSTITUTION_ADMIN, institution_code: "INST-001" }],
  ["demo-operator", { user_id: "user-op-1", login: "operator.demo", role: ROLES.INSTITUTION_OPERATOR, institution_code: "INST-001" }],
  ["demo-aicte", { user_id: "user-aicte-1", login: "aicte.demo", role: ROLES.AICTE_ADMIN, institution_code: null }],
  ["demo-verifier", { user_id: "user-verifier-1", login: "verifier.demo", role: ROLES.VERIFIER, institution_code: null }]
]);

const server = createServer(async (request, response) => {
  const requestId = request.headers["x-request-id"] || randomUUID();
  response.setHeader("x-request-id", requestId);
  response.setHeader("content-type", "application/json; charset=utf-8");
  try {
    if (request.method === "GET" && request.url === "/health/live") return send(response, 200, { status: "ok" });
    if (request.method === "POST" && request.url === "/api/v1/auth/login") return login(request, response);

    const user = authenticate(request);
    if (!user) return send(response, 401, { error: "UNAUTHENTICATED", message: "Authentication required.", request_id: requestId });
    const url = new URL(request.url, "http://localhost");

    if (request.method === "GET" && url.pathname === "/api/v1/me") return send(response, 200, user);
    if (request.method === "GET" && url.pathname === "/api/v1/institutions") return listInstitutions(response, user);
    if (request.method === "GET" && url.pathname === "/api/v1/dashboard/institution") return institutionDashboard(response, user);
    if (request.method === "GET" && url.pathname === "/api/v1/dashboard/aicte") return aicteDashboard(response, user);
    if (request.method === "POST" && url.pathname === "/api/v1/batches") return createBatch(request, response, user);

    const batchMatch = url.pathname.match(/^\/api\/v1\/batches\/([^/]+)(?:\/(issues|commit))?$/);
    if (request.method === "GET" && batchMatch) return getBatch(response, user, batchMatch[1], batchMatch[2]);
    if (request.method === "POST" && batchMatch?.[2] === "commit") return commitBatch(response, user, batchMatch[1]);

    const issueMatch = url.pathname.match(/^\/api\/v1\/issues\/([^/]+)\/resolve$/);
    if (request.method === "POST" && issueMatch) return resolveIssue(request, response, user, issueMatch[1]);
    return send(response, 404, { error: "NOT_FOUND", message: "Route not found.", request_id: requestId });
  } catch (error) {
    console.error(`[${requestId}] request failed: ${error.message}`);
    return send(response, 500, { error: "INTERNAL_ERROR", message: "Unexpected server error.", request_id: requestId });
  }
});

function login(request, response) {
  return readJson(request).then((body) => {
    const token = body?.token;
    const user = sessions.get(token);
    if (!user) return send(response, 401, { error: "INVALID_CREDENTIALS", message: "Use a documented demo token." });
    return send(response, 200, { access_token: token, token_type: "Bearer", user });
  });
}

function authenticate(request) {
  const value = request.headers.authorization || "";
  return value.startsWith("Bearer ") ? sessions.get(value.slice(7)) : null;
}

function listInstitutions(response, user) {
  if (user.role === ROLES.AICTE_ADMIN) return send(response, 200, { institutions: store.institutions });
  return send(response, 200, { institutions: store.institutions.filter((item) => item.code === user.institution_code) });
}

function createBatch(request, response, user) {
  if (![ROLES.INSTITUTION_ADMIN, ROLES.INSTITUTION_OPERATOR].includes(user.role)) {
    return send(response, 403, { error: "FORBIDDEN", message: "Institution role required." });
  }
  return readJson(request).then((body) => {
    const parsed = parseCsv(body?.csv);
    if (parsed.fileError) return send(response, 400, { error: "INVALID_FILE", message: parsed.fileError });
    const context = {
      institution: { code: user.institution_code },
      programmes: store.programmes,
      intakes: store.intakes,
      admissions: store.admissions,
      allowedStatuses: new Set(["active", "withdrawn", "completed", "transferred"]),
      allowedSeatCategories: new Set(["regular", "supernumerary", "nri", "oci", "foreign_national"])
    };
    const result = validateRows(parsed.rows, context);
    const batch = {
      id: randomUUID(),
      institution_code: user.institution_code,
      submitted_by: user.user_id,
      file_hash: createHash("sha256").update(body.csv).digest("hex"),
      schema_version: "student-records-v1",
      status: "validated",
      candidates: result.candidates,
      issues: result.issues,
      summary: summarize(result.candidates, result.issues),
      submitted_at: new Date().toISOString()
    };
    store.batches.push(batch);
    audit(user, "batch.validated", "batch", batch.id, { summary: batch.summary });
    return send(response, 201, { id: batch.id, status: batch.status, summary: batch.summary, issues: batch.issues });
  });
}

function getBatch(response, user, id, suffix) {
  const batch = store.batches.find((item) => item.id === id);
  if (!batch) return send(response, 404, { error: "NOT_FOUND", message: "Batch not found." });
  if (!canAccessInstitution(user, batch.institution_code)) return send(response, 403, { error: "FORBIDDEN", message: "Batch is outside your institution scope." });
  if (suffix === "issues") return send(response, 200, { batch_id: id, issues: batch.issues });
  return send(response, 200, publicBatch(batch));
}

function commitBatch(response, user, id) {
  const batch = store.batches.find((item) => item.id === id);
  if (!batch) return send(response, 404, { error: "NOT_FOUND", message: "Batch not found." });
  if (![ROLES.INSTITUTION_ADMIN, ROLES.INSTITUTION_OPERATOR].includes(user.role) || !canAccessInstitution(user, batch.institution_code)) {
    return send(response, 403, { error: "FORBIDDEN", message: "You cannot commit this batch." });
  }
  if (batch.status === "committed") return send(response, 409, { error: "ALREADY_COMMITTED", message: "Batch was already committed." });
  const accepted = batch.candidates.filter((candidate) => candidate.status === "accepted");
  store.admissions.push(...accepted.map((candidate) => ({ ...candidate, batch_id: batch.id, committed_at: new Date().toISOString() })));
  batch.status = "committed";
  batch.summary.committed = accepted.length;
  audit(user, "batch.committed", "batch", batch.id, { committed: accepted.length });
  return send(response, 200, publicBatch(batch));
}

function resolveIssue(request, response, user, issueId) {
  if (![ROLES.AICTE_ADMIN, ROLES.INSTITUTION_ADMIN].includes(user.role)) {
    return send(response, 403, { error: "FORBIDDEN", message: "Reviewer role required." });
  }
  return readJson(request).then((body) => {
    const [batchId, rowNumber, code] = issueId.split(":");
    const batch = store.batches.find((item) => item.id === batchId);
    if (!batch || !canAccessInstitution(user, batch.institution_code)) return send(response, 404, { error: "NOT_FOUND", message: "Issue not found." });
    const target = batch.issues.find((item) => String(item.row_number) === rowNumber && item.code === code);
    if (!target) return send(response, 404, { error: "NOT_FOUND", message: "Issue not found." });
    target.status = body?.decision === "dismiss" ? "dismissed" : "resolved";
    target.resolved_by = user.user_id;
    target.resolution = typeof body?.reason === "string" ? body.reason.slice(0, 500) : "";
    audit(user, "issue.resolved", "issue", issueId, { decision: target.status });
    return send(response, 200, target);
  });
}

function institutionDashboard(response, user) {
  if (!canAccessInstitution(user, user.institution_code) || user.role === ROLES.AICTE_ADMIN) {
    return send(response, 403, { error: "FORBIDDEN", message: "Institution role required." });
  }
  return send(response, 200, dashboardFor([user.institution_code]));
}

function aicteDashboard(response, user) {
  if (user.role !== ROLES.AICTE_ADMIN) return send(response, 403, { error: "FORBIDDEN", message: "AICTE role required." });
  return send(response, 200, dashboardFor(store.institutions.map((item) => item.code)));
}

function dashboardFor(institutions) {
  const admissions = store.admissions.filter((item) => institutions.includes(item.institution_code));
  return {
    institutions,
    accepted_active_admissions: admissions.filter((item) => ACTIVE_STATUSES.has(item.admission_status)).length,
    batches: store.batches.filter((item) => institutions.includes(item.institution_code)).map((batch) => publicBatch(batch)),
    intake_exceptions: store.batches.flatMap((batch) => batch.issues.filter((item) => item.code === "INTAKE_EXCEEDED").map((item) => ({ batch_id: batch.id, ...item })))
  };
}

function canAccessInstitution(user, institutionCode) {
  return user.role === ROLES.AICTE_ADMIN || user.institution_code === institutionCode;
}

function publicBatch(batch) {
  return { id: batch.id, institution_code: batch.institution_code, status: batch.status, schema_version: batch.schema_version, summary: batch.summary, submitted_at: batch.submitted_at };
}

function audit(user, action, entityType, entityId, details) {
  store.audit.push({ id: randomUUID(), actor_id: user.user_id, actor_role: user.role, action, entity_type: entityType, entity_id: entityId, timestamp: new Date().toISOString(), details });
}

function readJson(request) {
  return new Promise((resolve, reject) => {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
      if (body.length > 3_000_000) request.destroy(new Error("request too large"));
    });
    request.on("end", () => {
      try { resolve(body ? JSON.parse(body) : {}); } catch { reject(new Error("invalid JSON")); }
    });
    request.on("error", reject);
  });
}

function send(response, status, body) {
  response.statusCode = status;
  response.end(JSON.stringify(body));
  return response;
}

if (process.env.NODE_ENV !== "test") {
  server.listen(PORT, () => console.log(`APAAR MVP API listening on http://localhost:${PORT}`));
}

export { server, store, sessions, validateRows };
