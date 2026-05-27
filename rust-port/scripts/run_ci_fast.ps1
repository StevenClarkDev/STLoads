param(
    [switch]$SkipClippy
)

$ErrorActionPreference = "Stop"

Push-Location (Join-Path $PSScriptRoot "..")
try {
    cargo fmt --all -- --check
    cargo check --workspace
    if (-not $SkipClippy) {
        cargo clippy --workspace --all-targets -- -D warnings
    }
    cargo test -p domain
    cargo test -p shared
} finally {
    Pop-Location
}
