$ErrorActionPreference = "Stop"

Push-Location (Join-Path $PSScriptRoot "..")
try {
    $patterns = @(
        "tracing::(debug|info|warn|error)!\([^;]*(password|secret|token|authorization|client_secret|webhook)",
        "(println!|eprintln!)\([^;]*(password|secret|token|authorization|client_secret|webhook)",
        "console\.(log|warn|error)\([^;]*(password|secret|token|authorization|clientSecret|webhook)"
    )

    $findings = @()
    foreach ($pattern in $patterns) {
        $output = rg --hidden --glob "!.git/**" --glob "!target/**" --glob "!node_modules/**" --glob "!playwright-report/**" --glob "!crates/frontend-leptos/dist/**" --glob "!docs/**" --line-number -- $pattern 2>$null
        if ($LASTEXITCODE -gt 1) {
            throw "Sensitive output scan failed while running ripgrep."
        }

        foreach ($line in $output) {
            $lineText = [string]$line
            if ($lineText -match "run_sensitive_output_scan.ps1") {
                continue
            }
            $findings += $lineText
        }
    }

    if ($findings.Count -gt 0) {
        $findings | ForEach-Object { Write-Error $_ }
        throw "Sensitive output scan found possible secret-bearing logs or console output."
    }
} finally {
    Pop-Location
}
