param(
    [string]$HostedFrontendUrl = "",
    [string]$ArtifactDirectory = "target/p16-empty-state-smoke"
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$scanPaths = @(
    (Join-Path $root "crates/frontend-leptos/src/pages"),
    (Join-Path $root "crates/backend/src/screen_data.rs"),
    (Join-Path $root "crates/frontend-leptos/dist")
) | Where-Object { Test-Path $_ }

$bannedProductionStrings = @(
    "demo-tenant",
    "handoff-demo",
    "TMS-RUST-1001",
    "Reset sample",
    "Serving sample",
    "Dallas to Joliet produce reload",
    "245000",
    "rate variance",
    "ibm-cos://bucket/load-docs/rate-confirmation.pdf"
)

$violations = New-Object System.Collections.Generic.List[string]
$scanFiles = foreach ($path in $scanPaths) {
    if (Test-Path $path -PathType Container) {
        Get-ChildItem -Path $path -Recurse -File
    } else {
        Get-Item -Path $path
    }
}
foreach ($needle in $bannedProductionStrings) {
    $matches = Select-String -Path $scanFiles.FullName -Pattern ([regex]::Escape($needle)) -SimpleMatch:$false -ErrorAction SilentlyContinue
    foreach ($match in $matches) {
        $violations.Add("$($match.Path):$($match.LineNumber): $needle")
    }
}

if ($violations.Count -gt 0) {
    Write-Host "Production demo-data scan failed:" -ForegroundColor Red
    $violations | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}

if ($HostedFrontendUrl.Trim().Length -gt 0) {
    New-Item -ItemType Directory -Force -Path $ArtifactDirectory | Out-Null
    $artifactPath = Join-Path $ArtifactDirectory "hosted-frontend.html"
    $response = Invoke-WebRequest -Uri $HostedFrontendUrl -UseBasicParsing -TimeoutSec 60
    $response.Content | Set-Content -Path $artifactPath -Encoding UTF8
    foreach ($needle in $bannedProductionStrings) {
        if ($response.Content.Contains($needle)) {
            Write-Host "Hosted frontend contains banned production string: $needle" -ForegroundColor Red
            exit 1
        }
    }
    Write-Host "Hosted frontend snapshot saved to $artifactPath"
}

Write-Host "Production demo-data scan passed."
