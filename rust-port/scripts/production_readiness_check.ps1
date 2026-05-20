param(
    [string]$BackendBaseUrl = '',
    [string]$HostedFrontendUrl = '',
    [switch]$SkipEnvCheck,
    [switch]$SkipClippy,
    [switch]$SkipFrontendBuild,
    [switch]$SkipHostedSmoke,
    [switch]$SkipDemoDataScan,
    [switch]$SkipDbBackedTests
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Root = Resolve-Path (Join-Path $PSScriptRoot '..')
$FrontendRoot = Join-Path $Root 'crates/frontend-leptos'
$Report = [ordered]@{
    started_at = (Get-Date).ToUniversalTime().ToString('o')
    root = $Root.Path
    checks = @()
}

function Add-Result {
    param([string]$Name, [string]$Status, [string]$Detail = '')
    $script:Report.checks += [ordered]@{
        name = $Name
        status = $Status
        detail = $Detail
    }
}

function Invoke-ReadinessCommand {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [string]$WorkingDirectory = $Root.Path
    )

    Write-Host "`n==> $Name" -ForegroundColor Cyan
    Push-Location $WorkingDirectory
    try {
        & $FilePath @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "$Name failed with exit code $LASTEXITCODE."
        }
        Add-Result -Name $Name -Status 'passed'
    }
    finally {
        Pop-Location
    }
}

function Invoke-CargoTestGroup {
    param([string]$Group, [string[]]$Arguments)
    Invoke-ReadinessCommand -Name "test group: $Group" -FilePath 'cargo' -Arguments $Arguments
}

function Invoke-ClippyGate {
    Write-Host "`n==> cargo clippy --workspace --all-targets" -ForegroundColor Cyan
    Push-Location $Root.Path
    try {
        & cargo @('clippy', '--workspace', '--all-targets')
        if ($LASTEXITCODE -eq 0) {
            Add-Result -Name 'cargo clippy --workspace --all-targets' -Status 'passed'
            return
        }

        $fallbackToolchains = @('1.95.0-x86_64-pc-windows-msvc', '1.94.1-x86_64-pc-windows-msvc')
        foreach ($toolchain in $fallbackToolchains) {
            $clippyPath = (& rustup "+$toolchain" which clippy-driver 2>$null)
            if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($clippyPath)) {
                continue
            }
            Write-Host "Retrying clippy with installed toolchain $toolchain." -ForegroundColor Yellow
            & rustup @('run', $toolchain, 'cargo', 'clippy', '--workspace', '--all-targets')
            if ($LASTEXITCODE -eq 0) {
                Add-Result -Name 'cargo clippy --workspace --all-targets' -Status 'passed' -Detail "Used fallback toolchain $toolchain because the active clippy shim failed."
                return
            }
        }

        throw 'cargo clippy --workspace --all-targets failed.'
    }
    finally {
        Pop-Location
    }
}

function Assert-RequiredEnvironment {
    $required = @(
        'DATABASE_URL',
        'PUBLIC_BASE_URL',
        'CORS_ALLOWED_ORIGINS',
        'DOCUMENT_STORAGE_BACKEND',
        'OBJECT_STORAGE_BUCKET',
        'OBJECT_STORAGE_REGION',
        'OBJECT_STORAGE_ACCESS_KEY_ID',
        'OBJECT_STORAGE_SECRET_ACCESS_KEY',
        'STRIPE_SECRET_KEY',
        'STRIPE_WEBHOOK_SECRET',
        'ATMP_OUTBOUND_BASE_URL',
        'ATMP_INTEGRATION_SHARED_SECRET',
        'TMS_SHARED_SECRET',
        'MAIL_MAILER',
        'PORTAL_URL'
    )
    $missing = @()
    foreach ($name in $required) {
        $value = [Environment]::GetEnvironmentVariable($name)
        if ([string]::IsNullOrWhiteSpace($value)) {
            $missing += $name
        }
    }
    if ($missing.Count -gt 0) {
        throw "Missing production readiness environment variable(s): $($missing -join ', ')"
    }
    Add-Result -Name 'required environment variables' -Status 'passed' -Detail ($required -join ', ')
}

function Assert-NoProductionDemoFixtures {
    $sharedCargo = Get-Content -Path (Join-Path $Root 'crates/shared/Cargo.toml') -Raw
    if ($sharedCargo -match '(?ms)^\s*default\s*=\s*\[[^\]]*demo-fixtures') {
        throw 'The shared demo-fixtures feature is enabled by default.'
    }

    $sourceRoots = @(
        (Join-Path $Root 'crates/backend/src'),
        (Join-Path $Root 'crates/frontend-leptos/src'),
        (Join-Path $Root 'crates/shared/src')
    )
    $denyPatterns = @(
        '\bsample_[a-zA-Z0-9_]+\(',
        'Lorem ipsum',
        'fake carrier',
        'fake payment',
        'placeholder carrier',
        'placeholder load'
    )
    $findings = @()

    foreach ($sourceRoot in $sourceRoots) {
        if (-not (Test-Path $sourceRoot)) {
            continue
        }
        foreach ($file in Get-ChildItem -Path $sourceRoot -Recurse -File -Include *.rs) {
            $lines = Get-Content -Path $file.FullName
            $text = $lines -join "`n"
            foreach ($pattern in $denyPatterns) {
                $matches = [regex]::Matches($text, $pattern)
                foreach ($match in $matches) {
                    if ($match.Value -like 'sample_*' -and $match.Value -like '*without_sample_rows*') {
                        continue
                    }
                    $prefix = $text.Substring(0, $match.Index)
                    $lineNumber = ($prefix -split "`n").Count
                    $contextStart = [Math]::Max(0, $lineNumber - 4)
                    $contextEnd = [Math]::Min($lines.Count - 1, $lineNumber - 1)
                    $context = ($lines[$contextStart..$contextEnd] -join "`n")
                    if ($match.Value -like 'sample_*' -and $context -match '#\[cfg\(feature = "demo-fixtures"\)\]') {
                        continue
                    }
                    $relative = Resolve-Path -Path $file.FullName -Relative
                    $findings += "${relative}:$lineNumber $pattern"
                }
            }
        }
    }

    if ($findings.Count -gt 0) {
        throw "Production demo or placeholder source strings found:`n$($findings -join "`n")"
    }

    if (-not [string]::IsNullOrWhiteSpace($HostedFrontendUrl)) {
        Invoke-ReadinessCommand -Name 'hosted frontend demo-data scan' -FilePath 'powershell' -Arguments @(
            '-ExecutionPolicy', 'Bypass',
            '-File', (Join-Path $PSScriptRoot 'verify_frontend_no_demo_data.ps1'),
            '-HostedFrontendUrl', $HostedFrontendUrl
        )
    } else {
        Add-Result -Name 'hosted frontend demo-data scan' -Status 'skipped' -Detail 'HostedFrontendUrl not supplied.'
    }

    Add-Result -Name 'source demo fixture gate' -Status 'passed'
}

Write-Host "STLoads production readiness check" -ForegroundColor Green
Write-Host "Root: $($Root.Path)"

if ($SkipEnvCheck) {
    Add-Result -Name 'required environment variables' -Status 'skipped' -Detail 'SkipEnvCheck supplied.'
} else {
    Assert-RequiredEnvironment
}

Invoke-ReadinessCommand -Name 'cargo fmt --all --check' -FilePath 'cargo' -Arguments @('fmt', '--all', '--check')

if ($SkipClippy) {
    Add-Result -Name 'cargo clippy --workspace --all-targets' -Status 'skipped' -Detail 'SkipClippy supplied.'
} else {
    Invoke-ClippyGate
}

Invoke-CargoTestGroup -Group 'contract tests' -Arguments @('test', '-p', 'backend', '--test', 'atmp_contract', '--', '--nocapture')
Invoke-CargoTestGroup -Group 'tenant isolation tests' -Arguments @('test', '-p', 'backend', '--test', 'tenant_scope', '--', '--nocapture')
Invoke-CargoTestGroup -Group 'auth/RBAC tests' -Arguments @('test', '-p', 'backend', '--test', 'integration_auth', '--', '--nocapture')
Invoke-CargoTestGroup -Group 'marketplace tests' -Arguments @('test', '-p', 'backend', '--test', 'marketplace_workflows', '--', '--nocapture')
Invoke-CargoTestGroup -Group 'booking concurrency tests' -Arguments @('test', '-p', 'backend', '--test', 'marketplace_workflows', 'offers_tenders_book_now_and_cancellations_emit_events_and_lock_booking', '--', '--nocapture')
Invoke-CargoTestGroup -Group 'document tests' -Arguments @('test', '-p', 'backend', '--test', 'document_workflows', '--', '--nocapture')
Invoke-CargoTestGroup -Group 'payment webhook tests' -Arguments @('test', '-p', 'backend', 'routes::payments::tests', '--', '--nocapture')
Invoke-CargoTestGroup -Group 'ATMP outbound retry tests' -Arguments @('test', '-p', 'backend', '--test', 'atmp_outbound', '--', '--nocapture')
Invoke-CargoTestGroup -Group 'backend unit and route tests' -Arguments @('test', '-p', 'backend', '--', '--nocapture')

if ($SkipDbBackedTests) {
    Add-Result -Name 'db-backed acceptance tests' -Status 'skipped' -Detail 'SkipDbBackedTests supplied.'
} else {
    Invoke-CargoTestGroup -Group 'db-backed acceptance tests' -Arguments @('test', '-p', 'db', '--test', 'lifecycle_integration', '--', '--nocapture')
}

if ($SkipFrontendBuild) {
    Add-Result -Name 'frontend build tests' -Status 'skipped' -Detail 'SkipFrontendBuild supplied.'
} else {
    Invoke-ReadinessCommand -Name 'frontend build tests' -FilePath 'trunk' -Arguments @('build', '--release') -WorkingDirectory $FrontendRoot
}

if ($SkipHostedSmoke) {
    Add-Result -Name 'backend smoke test script' -Status 'skipped' -Detail 'SkipHostedSmoke supplied.'
} elseif (-not [string]::IsNullOrWhiteSpace($BackendBaseUrl)) {
    Invoke-ReadinessCommand -Name 'backend smoke test script' -FilePath 'powershell' -Arguments @(
        '-ExecutionPolicy', 'Bypass',
        '-File', (Join-Path $PSScriptRoot 'smoke_test_backend.ps1'),
        '-BaseUrl', $BackendBaseUrl,
        '-ReadinessSummaryPath', (Join-Path $Root 'target/production-readiness-smoke.json')
    )
} else {
    Add-Result -Name 'backend smoke test script' -Status 'skipped' -Detail 'BackendBaseUrl not supplied.'
}

if ($SkipDemoDataScan) {
    Add-Result -Name 'demo data and placeholder scan' -Status 'skipped' -Detail 'SkipDemoDataScan supplied.'
} else {
    Assert-NoProductionDemoFixtures
}

$Report.finished_at = (Get-Date).ToUniversalTime().ToString('o')
$Report.result = 'ok'
$Report | ConvertTo-Json -Depth 6
