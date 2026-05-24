param(
    [Parameter(Mandatory = $true)]
    [string]$EnvFile,

    [Parameter(Mandatory = $true)]
    [ValidateSet('local', 'ci', 'staging', 'pilot', 'production')]
    [string]$TargetEnvironment,

    [switch]$AllowPlaceholders
)

$ErrorActionPreference = 'Stop'

function Read-EnvFile {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Env file not found: $Path"
    }

    $values = @{}
    foreach ($line in Get-Content -LiteralPath $Path) {
        $trimmed = $line.Trim()
        if ($trimmed.Length -eq 0 -or $trimmed.StartsWith('#')) {
            continue
        }

        $separator = $trimmed.IndexOf('=')
        if ($separator -lt 1) {
            continue
        }

        $key = $trimmed.Substring(0, $separator).Trim()
        $value = $trimmed.Substring($separator + 1).Trim().Trim('"').Trim("'")
        $values[$key] = $value
    }

    return $values
}

function Test-Placeholder {
    param([string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return $true
    }

    $lower = $Value.Trim().ToLowerInvariant()
    return (
        $lower -eq 'null' -or
        $lower -eq 'none' -or
        $lower -eq 'todo' -or
        $lower -eq 'tbd' -or
        $lower -eq 'changeme' -or
        $lower -eq 'change_me' -or
        $lower -eq 'replace-me' -or
        $lower -eq 'replace_me' -or
        $lower.Contains('placeholder') -or
        $lower.Contains('replace_me') -or
        $lower.Contains('replace-me') -or
        $lower.Contains('example.com') -or
        $lower.StartsWith('your_') -or
        $lower.StartsWith('your-') -or
        $lower.StartsWith('http://localhost') -or
        $lower.StartsWith('https://localhost')
    )
}

function Add-Failure {
    param(
        [System.Collections.Generic.List[string]]$Failures,
        [string]$Message
    )
    $Failures.Add($Message) | Out-Null
}

$envValues = Read-EnvFile -Path $EnvFile
$failures = [System.Collections.Generic.List[string]]::new()

$commonRequired = @(
    'APP_ENV',
    'DEPLOYMENT_TARGET',
    'BIND_ADDR',
    'PORT',
    'PUBLIC_BASE_URL',
    'CORS_ALLOWED_ORIGINS',
    'RUN_MIGRATIONS',
    'DATABASE_URL',
    'STLOADS_DATABASE_SCHEMA',
    'DOCUMENT_STORAGE_BACKEND',
    'OBJECT_STORAGE_BUCKET',
    'OBJECT_STORAGE_REGION',
    'OBJECT_STORAGE_ENDPOINT',
    'OBJECT_STORAGE_ACCESS_KEY_ID',
    'OBJECT_STORAGE_SECRET_ACCESS_KEY',
    'OBJECT_STORAGE_FORCE_PATH_STYLE',
    'OBJECT_STORAGE_PREFIX',
    'STRIPE_SECRET',
    'STRIPE_WEBHOOK_SECRET_PLATFORM',
    'STRIPE_WEBHOOK_SECRET_CONNECT',
    'STRIPE_CONNECT_REFRESH_URL',
    'STRIPE_CONNECT_RETURN_URL',
    'TMS_SHARED_SECRET',
    'TMS_RETRY_WORKER_ENABLED',
    'TMS_RECONCILIATION_WORKER_ENABLED',
    'MAIL_MAILER',
    'MAIL_HOST',
    'MAIL_FROM_ADDRESS',
    'MAIL_FAIL_OPEN',
    'PORTAL_URL',
    'FRONTEND_PUBLIC_URL',
    'KILL_SWITCH_PAYMENTS',
    'KILL_SWITCH_BOOKING',
    'KILL_SWITCH_TMS_PUSHES',
    'KILL_SWITCH_NOTIFICATIONS',
    'KILL_SWITCH_DOCUMENT_UPLOADS'
)

if ($TargetEnvironment -in @('staging', 'pilot', 'production')) {
    foreach ($key in $commonRequired) {
        if (-not $envValues.ContainsKey($key) -or [string]::IsNullOrWhiteSpace($envValues[$key])) {
            Add-Failure $failures "$key is required for $TargetEnvironment"
            continue
        }

        if (-not $AllowPlaceholders -and (Test-Placeholder $envValues[$key])) {
            Add-Failure $failures "$key contains a placeholder value"
        }
    }
}

if ($TargetEnvironment -eq 'local') {
    foreach ($key in @('APP_ENV', 'BIND_ADDR', 'PORT')) {
        if (-not $envValues.ContainsKey($key) -or [string]::IsNullOrWhiteSpace($envValues[$key])) {
            Add-Failure $failures "$key is required for local"
        }
    }
}

if ($TargetEnvironment -eq 'ci') {
    foreach ($key in @('APP_ENV', 'BIND_ADDR', 'PORT')) {
        if (-not $envValues.ContainsKey($key) -or [string]::IsNullOrWhiteSpace($envValues[$key])) {
            Add-Failure $failures "$key is required for ci"
        }
    }
}

if ($envValues.ContainsKey('RUN_MIGRATIONS') -and $envValues['RUN_MIGRATIONS'].ToLowerInvariant() -ne 'false') {
    Add-Failure $failures 'RUN_MIGRATIONS must be false for web runtime env files'
}

if ($TargetEnvironment -in @('staging', 'pilot', 'production')) {
    if ($envValues.ContainsKey('DOCUMENT_STORAGE_BACKEND') -and $envValues['DOCUMENT_STORAGE_BACKEND'].ToLowerInvariant() -eq 'local') {
        Add-Failure $failures "$TargetEnvironment cannot use DOCUMENT_STORAGE_BACKEND=local"
    }

    if ($envValues.ContainsKey('CORS_ALLOWED_ORIGINS')) {
        $origins = $envValues['CORS_ALLOWED_ORIGINS'].Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ }
        if ($origins.Count -eq 0 -or $origins -contains '*') {
            Add-Failure $failures "$TargetEnvironment must define explicit CORS_ALLOWED_ORIGINS"
        }
    }
}

if ($TargetEnvironment -eq 'production') {
    if ($envValues.ContainsKey('APP_ENV') -and $envValues['APP_ENV'].ToLowerInvariant() -ne 'production') {
        Add-Failure $failures 'production validation requires APP_ENV=production'
    }

    if ($envValues.ContainsKey('MAIL_MAILER') -and $envValues['MAIL_MAILER'].ToLowerInvariant() -ne 'smtp') {
        Add-Failure $failures 'production requires MAIL_MAILER=smtp'
    }

    if ($envValues.ContainsKey('MAIL_FAIL_OPEN') -and $envValues['MAIL_FAIL_OPEN'].ToLowerInvariant() -ne 'false') {
        Add-Failure $failures 'production requires MAIL_FAIL_OPEN=false'
    }
}

if ($TargetEnvironment -eq 'staging') {
    if ($envValues.ContainsKey('APP_ENV') -and $envValues['APP_ENV'].ToLowerInvariant() -ne 'staging') {
        Add-Failure $failures 'staging validation requires APP_ENV=staging'
    }
}

if ($failures.Count -gt 0) {
    Write-Host "Runtime env validation failed for ${TargetEnvironment}:" -ForegroundColor Red
    foreach ($failure in $failures) {
        Write-Host "- $failure" -ForegroundColor Red
    }
    exit 1
}

Write-Host "Runtime env validation passed for ${TargetEnvironment}: $EnvFile"
