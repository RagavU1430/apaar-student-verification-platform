# APAAR Student Data Platform MVP

This is a runnable synthetic-data prototype derived from `PRD.md`. It demonstrates the smallest complete workflow:

- demo-token authentication with institution and AICTE roles;
- server-side institution scoping;
- bounded CSV ingestion with row-level validation;
- APAAR format checks explicitly labelled as **format validation, not identity verification**;
- duplicate candidates and configurable intake exceptions;
- transactional-style batch commit in the prototype store;
- audit events for validation, commit, and issue resolution;
- institution and AICTE dashboard endpoints.

## Run

Requires Node.js 20 or newer.

```powershell
npm test
npm start
```

The API listens on `http://localhost:3000`.

Demo bearer tokens:

| Token | Role | Scope |
| --- | --- | --- |
| `demo-institution` | Institution admin | `INST-001` |
| `demo-operator` | Institution operator | `INST-001` |
| `demo-aicte` | AICTE admin | All institutions |
| `demo-verifier` | Authorized verifier | No unrestricted search endpoint |

## Example upload

```powershell
$csv = @"
institution_code,programme_code,academic_year,apaar_id,institution_student_id,admission_date,admission_status,seat_category
INST-001,CSE,2026-27,ABCD1234,S-001,2026-07-01,active,regular
INST-001,CSE,2026-27,bad,S-002,2026-07-02,active,regular
"@
Invoke-RestMethod -Method Post -Uri http://localhost:3000/api/v1/batches `
  -Headers @{ Authorization = "Bearer demo-institution" } `
  -ContentType "application/json" -Body (@{ csv = $csv } | ConvertTo-Json)
```

## Deliberate prototype limits

This implementation uses an in-memory store and explicit demo tokens. It does not provide live APAAR verification, production authentication, PostgreSQL persistence, a React frontend, rate limiting, TLS, or a production retention policy. Those are release gates, not claims made by this MVP. The next production step is to port the tested domain functions into the PRD's Rust/Axum/SQLx modular monolith, add PostgreSQL constraints and migrations, and integrate only an officially approved identity adapter.
