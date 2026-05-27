param(
    [switch]$RequireDatabase
)

$ErrorActionPreference = "Stop"

Push-Location (Join-Path $PSScriptRoot "..")
try {
    if ($RequireDatabase -and -not $env:RUST_TEST_DATABASE_URL -and -not $env:TEST_DATABASE_URL) {
        throw "RUST_TEST_DATABASE_URL or TEST_DATABASE_URL is required for the backend integration lane."
    }

    cargo test --workspace

    if ($env:DATABASE_URL) {
        cargo sqlx prepare --workspace --check
    } else {
        Write-Host "Skipping SQLx metadata check because DATABASE_URL is not set."
    }
} finally {
    Pop-Location
}
