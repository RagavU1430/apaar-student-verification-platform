# End-to-end smoke test against a running API.
#
#   powershell -ExecutionPolicy Bypass -File scripts/smoke-test.ps1
#
# Requires: the API running on $BaseUrl with DATABASE_URL pointing at a
# migrated database, and the demo users seeded (cargo run --bin seed_users).

param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$Password = "DemoPass123!"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$sampleData = Join-Path $root "backend\sample-data"

function Login($identifier) {
    $body = @{ login_identifier = $identifier; password = $Password } | ConvertTo-Json
    $response = Invoke-RestMethod -Uri "$BaseUrl/api/v1/auth/login" -Method Post `
        -ContentType "application/json" -Body $body
    return $response.access_token
}

function Assert-True($condition, $message) {
    if (-not $condition) {
        throw "FAILED: $message"
    }
    Write-Host "  ok  $message" -ForegroundColor Green
}

Write-Host "== health ==" -ForegroundColor Cyan
$live = Invoke-RestMethod -Uri "$BaseUrl/health/live"
Assert-True ($live.status -eq "ok") "liveness"
$ready = Invoke-RestMethod -Uri "$BaseUrl/health/ready"
Assert-True ($ready.database -eq "up") "database reachable"

Write-Host "== authentication ==" -ForegroundColor Cyan
$alpha = Login "alpha.operator"
Assert-True ($alpha.Length -gt 20) "institution user signed in"
$aicte = Login "aicte.admin"
Assert-True ($aicte.Length -gt 20) "aicte user signed in"

Write-Host "== authorisation ==" -ForegroundColor Cyan
try {
    Invoke-RestMethod -Uri "$BaseUrl/api/v1/dashboard/aicte" -Headers @{ Authorization = "Bearer $alpha" } | Out-Null
    throw "FAILED: the institution user reached the AICTE dashboard"
} catch {
    Assert-True ($_.Exception.Response.StatusCode.value__ -eq 403) "institution user is refused the AICTE dashboard"
}

Write-Host "== upload ==" -ForegroundColor Cyan
$csv = Get-Content (Join-Path $sampleData "demo_batch.csv") -Raw
$boundary = "----smoketest"
$multipart = "--$boundary`r`n" +
"Content-Disposition: form-data; name=`"file`"; filename=`"demo_batch.csv`"`r`n" +
"Content-Type: text/csv`r`n`r`n" +
$csv + "`r`n" +
"--$boundary--`r`n"

$batch = Invoke-RestMethod -Uri "$BaseUrl/api/v1/batches" -Method Post `
    -Headers @{ Authorization = "Bearer $alpha"; "Content-Type" = "multipart/form-data; boundary=$boundary" } `
    -Body ([System.Text.Encoding]::UTF8.GetBytes($multipart))

Assert-True ($batch.counts.rows -eq 8) "every row was counted ($($batch.counts.rows))"
Assert-True ($batch.counts.accepted -ge 1) "at least one row was accepted"
Assert-True ($batch.counts.rejected -ge 3) "problem rows were rejected ($($batch.counts.rejected))"
Assert-True ($batch.counts.needs_review -ge 1) "duplicate candidate was flagged for review"

$issues = Invoke-RestMethod -Uri "$BaseUrl/api/v1/batches/$($batch.id)/issues" `
    -Headers @{ Authorization = "Bearer $alpha" }
$codes = $issues | ForEach-Object { $_.rule_code }
Assert-True ($codes -contains "APAAR_ID_MISSING") "APAAR_ID_MISSING reported"
Assert-True ($codes -contains "DUPLICATE_APAAR_IN_BATCH") "DUPLICATE_APAAR_IN_BATCH reported"
Assert-True ($codes -contains "ADMISSION_DATE_INVALID") "ADMISSION_DATE_INVALID reported"

Write-Host "== commit ==" -ForegroundColor Cyan
$commit = Invoke-RestMethod -Uri "$BaseUrl/api/v1/batches/$($batch.id)/commit" -Method Post `
    -Headers @{ Authorization = "Bearer $alpha" }
Assert-True ($commit.status -eq "COMMITTED") "batch committed"

Write-Host "== dashboards ==" -ForegroundColor Cyan
$dashboard = Invoke-RestMethod -Uri "$BaseUrl/api/v1/dashboard/institution" `
    -Headers @{ Authorization = "Bearer $alpha" }
Assert-True ($dashboard.counts.total_batches -ge 1) "institution dashboard shows the batch"
$aicteDashboard = Invoke-RestMethod -Uri "$BaseUrl/api/v1/dashboard/aicte" `
    -Headers @{ Authorization = "Bearer $aicte" }
Assert-True ($aicteDashboard.institutions.Count -ge 2) "aicte dashboard covers both institutions"

Write-Host ""
Write-Host "SMOKE TEST PASSED" -ForegroundColor Cyan
