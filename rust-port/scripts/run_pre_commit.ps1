$ErrorActionPreference = "Stop"

Push-Location (Join-Path $PSScriptRoot "..")
try {
    cargo fmt --all -- --check
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run_ci_security.ps1 -SkipCargoAudit
} finally {
    Pop-Location
}
