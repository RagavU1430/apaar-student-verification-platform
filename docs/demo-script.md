# Demo script (3-5 minutes)

Everything below uses synthetic data. Say the words in **bold** out loud - they are the exact
claims the platform supports.

## 1. Sign in as an institution (30s)

- Open the frontend, sign in as `alpha.operator` / `DemoPass123!`.
- Point at the header: **"This account is scoped to one institution. The scope comes from the
  token on the server, not from anything the browser sends."**

## 2. Institution dashboard (45s)

- Show approved intake for `CSE-01`: 60 regular + 6 supernumerary.
- Say: **"Regular and supernumerary seats are counted separately and never merged, using the
  versioned demo rule `demo-v1`."**
- Point out the source reference: **"This capacity is a seeded demo fixture. It is not an AICTE
  approval record - that source is still an open item."**

## 3. Upload a batch with deliberate problems (75s)

Upload `backend/sample-data/demo_batch.csv`. It contains:

| Row | Scenario | Expected finding |
| --- | --- | --- |
| 2 | valid admission | Accepted |
| 3 | APAAR id missing | `APAAR_ID_MISSING` - rejected |
| 4 | duplicate of row 2 | `DUPLICATE_APAAR_IN_BATCH` - needs review |
| 5 | malformed date | `ADMISSION_DATE_INVALID` - rejected |
| 6 | unknown programme | `PROGRAMME_UNKNOWN` - rejected |
| 7 | name missing | `FULL_NAME_MISSING` - accepted with warning |

- Say: **"No row is discarded silently. Every row gets a status and a reason code."**
- Open row 4 and show the evidence: **"This is a duplicate candidate, not a fraud verdict. A
  human reviewer decides."**
- On the APAAR format finding, say: **"This is a format check only. The platform has no live
  APAAR source and never labels a record as APAAR-verified."**

## 4. Intake overrun (45s)

Upload `backend/sample-data/intake_overrun.csv` (65 regular admissions against 60 seats).

- Commit the batch.
- Show the intake exception: **"63 against 60 regular seats - the excess is calculated from
  persisted data under rule `demo-v1`, and the source records and rule version are attached."**

## 5. Switch to the AICTE role (45s)

- Sign out, sign in as `aicte.admin`.
- Show the cross-institution aggregates and the exception queue.
- Say: **"Aggregate view only. An AICTE administrator sees institutions and exceptions, not
  unrestricted student records."**

## 6. Resolve an issue with a reason (30s)

- Open Issues, add a reason, click Resolve.
- Say: **"The finding and its evidence are never deleted. The decision, the reason and the
  reviewer are appended to the history."**

## 7. Negative test, if time allows (30s)

- While signed in as `alpha.operator`, try to open Beta's batch id.
- The API returns `403 PERMISSION_DENIED` and the attempt is written to the audit trail.
- Say: **"Authorization is enforced on the server on every request. Hiding a button is not
  access control."**

## 8. Close (20s)

> **"The value here is not the storage. It is that every submission becomes a controlled
> data-quality process: what was submitted, what was validated, what is still unresolved, and
> who decided what. Live APAAR verification, official intake rules and production governance
> remain integration gates, not hidden assumptions."**

## Things never to say

- "AI verifies APAAR" - the platform only checks syntax and duplicates.
- "APAAR-verified student" - no authoritative check exists.
- "Fraud detected" - findings are review items with evidence.
- "Production ready" - this is a prototype with documented gaps.
