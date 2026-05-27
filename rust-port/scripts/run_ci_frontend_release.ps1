param(
    [string]$TrunkVersion = "0.21.14"
)

$ErrorActionPreference = "Stop"

Push-Location (Join-Path $PSScriptRoot "..")
try {
    rustup target add wasm32-unknown-unknown

    $installedVersion = $null
    try {
        $installedVersion = (trunk --version) -replace "^trunk\s+", ""
    } catch {
        $installedVersion = $null
    }

    if ($installedVersion -ne $TrunkVersion) {
        cargo install trunk --locked --version $TrunkVersion
    }

    Push-Location "crates/frontend-leptos"
    try {
        trunk build --release
    } finally {
        Pop-Location
    }
} finally {
    Pop-Location
}
