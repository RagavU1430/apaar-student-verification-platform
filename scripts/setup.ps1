# Prepares a Windows development environment for this project.
#
#   powershell -ExecutionPolicy Bypass -File scripts/setup.ps1
#
# It checks for the two tools the Rust GNU toolchain needs on Windows and
# prints the PATH change you need. Nothing is installed silently.

$ErrorActionPreference = 'Stop'

function Test-Tool($name, $args = '--version') {
    $command = Get-Command $name -ErrorAction SilentlyContinue
    if ($command) {
        Write-Host "OK       $name -> $($command.Source)" -ForegroundColor Green
        return $true
    }
    Write-Host "MISSING  $name" -ForegroundColor Yellow
    return $false
}

Write-Host "== Checking prerequisites ==" -ForegroundColor Cyan

$hasCargo = Test-Tool 'cargo'
$hasGcc = Test-Tool 'gcc'
$hasDllTool = Test-Tool 'dlltool'
$hasNode = Test-Tool 'node'
$hasDocker = Test-Tool 'docker'

if (-not $hasCargo) {
    Write-Host @"
Rust is not on PATH. Install the GNU toolchain (no admin needed):

  winget install --id Rustlang.Rustup --exact
  & "$env:USERPROFILE\.cargo\bin\rustup-init.exe" -y --default-host x86_64-pc-windows-gnu --profile minimal
  # add %USERPROFILE%\.cargo\bin to PATH
"@ -ForegroundColor Yellow
}

if (-not ($hasGcc -and $hasDllTool)) {
    Write-Host @"
MinGW-w64 is missing. The GNU Rust target needs gcc and dlltool to link and to
generate Windows import libraries. Install it (portable, no admin needed):

  winget install --id BrechtSanders.WinLibs.POSIX.MSVCRT --exact

then add the mingw64\bin directory to PATH.
"@ -ForegroundColor Yellow
}

if (-not $hasDocker) {
    Write-Host "Docker is missing. It is optional - you can point DATABASE_URL at a managed PostgreSQL instead." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "== Next steps ==" -ForegroundColor Cyan
Write-Host "1. Copy .env.example to .env and set DATABASE_URL."
Write-Host "2. cd backend; cargo run --bin seed_users   # creates the demo accounts"
Write-Host "3. cd backend; cargo run                    # starts the API on :8080"
Write-Host "4. cd frontend; npm install; npm run dev    # starts the UI on :5173"
