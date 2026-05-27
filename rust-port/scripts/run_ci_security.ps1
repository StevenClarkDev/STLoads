param(
    [switch]$SkipCargoAudit,
    [string[]]$AllowedSecretPaths = @(
        ".env.ibm.example",
        "docs/IBM_DEPLOYMENT_NOTES.md",
        "docs/IBM_CODE_ENGINE_DEPLOYMENT.md",
        "docs/ENTERPRISE_TEST_LANES_AND_CI.md"
    )
)

$ErrorActionPreference = "Stop"

Push-Location (Join-Path $PSScriptRoot "..")
try {
    if (-not $SkipCargoAudit) {
        if (-not (Get-Command cargo-audit -ErrorAction SilentlyContinue)) {
            cargo install cargo-audit --locked
        }
        cargo audit
    }

    $patterns = @(
        "AKIA[0-9A-Z]{16}",
        "sk_live_[0-9A-Za-z]{16,}",
        "xox[baprs]-[0-9A-Za-z-]{10,}",
        "-----BEGIN (RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----",
        "(?i)(api[_-]?key|secret|token|password)\s*[:=]\s*['""][^'""]{12,}['""]"
    )

    $findings = @()
    foreach ($pattern in $patterns) {
        $output = rg --hidden --glob "!.git/**" --glob "!target/**" --glob "!node_modules/**" --glob "!playwright-report/**" --glob "!crates/frontend-leptos/dist/**" --line-number -- $pattern 2>$null
        if ($LASTEXITCODE -gt 1) {
            throw "Secret scan failed while running ripgrep."
        }
        foreach ($line in $output) {
            $lineText = [string]$line
            $path = ($lineText -split ":", 2)[0]
            if ($lineText -match '\$\(|\$\{') {
                continue
            }
            if ($lineText -match "legacy-remember") {
                continue
            }
            if ($AllowedSecretPaths -notcontains $path) {
                $findings += $lineText
            }
        }
    }

    if ($findings.Count -gt 0) {
        $findings | ForEach-Object { Write-Error $_ }
        throw "Secret scan found possible committed credentials."
    }

    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run_sensitive_output_scan.ps1

    if (Test-Path "package.json") {
        npm audit --audit-level=high
    }
} finally {
    Pop-Location
}
