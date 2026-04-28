param(
    [string]$BaseUrl = 'https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud',
    [string]$FrontendUrl = 'https://stloads-rust-frontend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud',
    [string]$DatabaseUrl = '',
    [string]$CarrierStripeAccountId = 'acct_1TOTnoLMsudLt19f',
    [string]$RuntimeEnvPath = 'rust-port/runtime/stloads-rust-runtime.generated.env',
    [string]$RuntimeFallbackPath = 'rust-port/.env.ibm.runtime',
    [string]$SecretFallbackPath = 'rust-port/.env.ibm.secret'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Step {
    param([string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Resolve-EnvValue {
    param(
        [string[]]$Paths,
        [Parameter(Mandatory = $true)][string]$Key
    )

    foreach ($path in $Paths) {
        if (-not (Test-Path $path)) {
            continue
        }

        $line = Get-Content $path | Where-Object { $_ -match "^${Key}=" } | Select-Object -First 1
        if ($null -ne $line) {
            return ($line -replace "^${Key}=", '').Trim()
        }
    }

    return $null
}

function Resolve-DatabaseUrl {
    param([string]$Value)

    if (-not [string]::IsNullOrWhiteSpace($Value)) {
        return $Value.Trim()
    }

    if (-not [string]::IsNullOrWhiteSpace($env:DATABASE_URL)) {
        return $env:DATABASE_URL.Trim()
    }

    $resolved = Resolve-EnvValue -Paths @($RuntimeEnvPath, $RuntimeFallbackPath, $SecretFallbackPath) -Key 'DATABASE_URL'
    if (-not [string]::IsNullOrWhiteSpace($resolved)) {
        return $resolved
    }

    throw 'DATABASE_URL was not provided and could not be resolved from the runtime env files.'
}

function Add-DatabaseHostaddrFallback {
    param([string]$DbUrl)

    if ([string]::IsNullOrWhiteSpace($DbUrl) -or $DbUrl -match '(^|[?&])hostaddr=') {
        return $DbUrl
    }

    try {
        $uri = [System.Uri]$DbUrl
        $resolved = Resolve-DnsName $uri.Host -Type A -ErrorAction Stop |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_.IPAddress) } |
            Select-Object -First 1 -ExpandProperty IPAddress

        if ([string]::IsNullOrWhiteSpace($resolved)) {
            return $DbUrl
        }

        if ($DbUrl.Contains('?')) {
            return "$DbUrl&hostaddr=$resolved"
        }

        return "$DbUrl?hostaddr=$resolved"
    }
    catch {
        return $DbUrl
    }
}

function Resolve-PsqlExecutable {
    $candidate = Get-Command psql -ErrorAction SilentlyContinue
    if ($null -ne $candidate) {
        return $candidate.Source
    }

    foreach ($path in @(
        'C:\Program Files\PostgreSQL\18\bin\psql.exe',
        'C:\Program Files\PostgreSQL\17\bin\psql.exe',
        'C:\Program Files\PostgreSQL\16\bin\psql.exe',
        'C:\Program Files\PostgreSQL\15\bin\psql.exe',
        'C:\Program Files\PostgreSQL\14\bin\psql.exe',
        'C:\Program Files\PostgreSQL\13\bin\psql.exe'
    )) {
        if (Test-Path $path) {
            return $path
        }
    }

    return $null
}

function Test-DockerPsqlAvailable {
    $docker = Get-Command docker -ErrorAction SilentlyContinue
    if ($null -eq $docker) {
        return $false
    }

    & $docker.Source version --format '{{.Server.Version}}' *> $null
    return ($LASTEXITCODE -eq 0)
}

function Escape-PsqlConninfoValue {
    param([string]$Value)

    if ($null -eq $Value) {
        return ''
    }

    return $Value.Replace("'", "''")
}

function Convert-DbUrlToPsqlConninfo {
    param([Parameter(Mandatory = $true)][string]$DbUrl)

    $uri = [System.Uri]$DbUrl
    $query = [System.Web.HttpUtility]::ParseQueryString($uri.Query)
    $userInfo = $uri.UserInfo.Split(':', 2)
    $username = [System.Uri]::UnescapeDataString($userInfo[0])
    $password = if ($userInfo.Count -gt 1) { [System.Uri]::UnescapeDataString($userInfo[1]) } else { '' }
    $dbName = $uri.AbsolutePath.TrimStart('/')
    $parts = @(
        "host='$(Escape-PsqlConninfoValue $uri.Host)'"
        "port='$($uri.Port)'"
        "dbname='$(Escape-PsqlConninfoValue $dbName)'"
        "user='$(Escape-PsqlConninfoValue $username)'"
    )

    if (-not [string]::IsNullOrWhiteSpace($password)) {
        $parts += "password='$(Escape-PsqlConninfoValue $password)'"
    }

    foreach ($key in @('sslmode', 'hostaddr')) {
        $value = $query[$key]
        if (-not [string]::IsNullOrWhiteSpace($value)) {
            $parts += "$key='$(Escape-PsqlConninfoValue $value)'"
        }
    }

    return ($parts -join ' ')
}

function Invoke-SeedDatabase {
    param([Parameter(Mandatory = $true)][string]$DbUrl)

    $seedScript = Join-Path $PSScriptRoot 'seed_postgres_smoke_data.sql'
    if (-not (Test-Path $seedScript)) {
        throw "Seed SQL not found at $seedScript"
    }

    $psql = Resolve-PsqlExecutable
    if (-not [string]::IsNullOrWhiteSpace($psql)) {
        $conninfo = Convert-DbUrlToPsqlConninfo -DbUrl $DbUrl
        & $psql $conninfo -v ON_ERROR_STOP=1 -f $seedScript | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw 'Local psql reseed failed.'
        }
        return 'psql'
    }

    if (Test-DockerPsqlAvailable) {
        Get-Content $seedScript | docker run --rm -i postgres:16-alpine psql "$DbUrl" -v ON_ERROR_STOP=1 | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw 'Docker-based psql reseed failed.'
        }
        return 'docker'
    }

    throw 'Neither a local psql client nor a reachable Docker daemon is available for reseeding.'
}

function Invoke-DbNonQuery {
    param(
        [Parameter(Mandatory = $true)][string]$DbUrl,
        [Parameter(Mandatory = $true)][string]$Sql
    )

    $psql = Resolve-PsqlExecutable
    if (-not [string]::IsNullOrWhiteSpace($psql)) {
        $conninfo = Convert-DbUrlToPsqlConninfo -DbUrl $DbUrl
        $null = & $psql $conninfo -v ON_ERROR_STOP=1 -c "$Sql"
        if ($LASTEXITCODE -ne 0) {
            throw 'Local psql command failed.'
        }
        return
    }

    if (Test-DockerPsqlAvailable) {
        $null = docker run --rm postgres:16-alpine psql "$DbUrl" -v ON_ERROR_STOP=1 -c "$Sql"
        if ($LASTEXITCODE -ne 0) {
            throw 'Docker-based psql command failed.'
        }
        return
    }

    throw 'Neither a local psql client nor a reachable Docker daemon is available for database writes.'
}

function Require-EnvValue {
    param(
        [string[]]$Paths,
        [Parameter(Mandatory = $true)][string]$Key
    )

    $value = Resolve-EnvValue -Paths $Paths -Key $Key
    if ([string]::IsNullOrWhiteSpace($value)) {
        throw "$Key could not be resolved from the runtime env files."
    }

    return $value
}

$resolvedDbUrl = Add-DatabaseHostaddrFallback -DbUrl (Resolve-DatabaseUrl -Value $DatabaseUrl)
$env:DATABASE_URL = $resolvedDbUrl
$env:STRIPE_SECRET = Require-EnvValue -Paths @($RuntimeEnvPath, $RuntimeFallbackPath, $SecretFallbackPath) -Key 'STRIPE_SECRET'
$env:STRIPE_WEBHOOK_SECRET_PLATFORM = Require-EnvValue -Paths @($RuntimeEnvPath, $RuntimeFallbackPath, $SecretFallbackPath) -Key 'STRIPE_WEBHOOK_SECRET_PLATFORM'

Write-Step 'Reseeding the hosted PostgreSQL smoke dataset'
$seedMethod = Invoke-SeedDatabase -DbUrl $resolvedDbUrl

Write-Step 'Running hosted API smoke validation'
& (Join-Path $PSScriptRoot 'smoke_test_backend.ps1') -BaseUrl $BaseUrl

Write-Step 'Running hosted Rust role and lifecycle matrix validation'
& (Join-Path $PSScriptRoot 'verify_rust_role_matrix.ps1') -BaseUrl $BaseUrl -FrontendUrl $FrontendUrl

Write-Step 'Running hosted SMTP validation'
& (Join-Path $PSScriptRoot 'verify_smtp_hosted.ps1') -BaseUrl $BaseUrl -DatabaseUrl $resolvedDbUrl

Write-Step 'Running hosted TMS worker validation'
& (Join-Path $PSScriptRoot 'verify_tms_workers_hosted.ps1') -BaseUrl $BaseUrl -DatabaseUrl $resolvedDbUrl

Write-Step 'Reseeding the hosted PostgreSQL smoke dataset before Stripe release validation'
$secondSeedMethod = Invoke-SeedDatabase -DbUrl $resolvedDbUrl

if (-not [string]::IsNullOrWhiteSpace($CarrierStripeAccountId)) {
    Write-Step 'Restoring the known onboarded staging carrier Stripe account'
    $sqlAccountId = $CarrierStripeAccountId.Replace("'", "''")
    Invoke-DbNonQuery -DbUrl $resolvedDbUrl -Sql "UPDATE users SET stripe_connect_account_id = '$sqlAccountId', updated_at = CURRENT_TIMESTAMP WHERE id = 9103;"
}

Write-Step 'Running hosted Stripe release validation'
& (Join-Path $PSScriptRoot 'verify_stripe_hosted.ps1') -BaseUrl $BaseUrl -StripeSecret $env:STRIPE_SECRET -StripeWebhookSecret $env:STRIPE_WEBHOOK_SECRET_PLATFORM

Write-Step 'Hosted backend cutover summary'
[ordered]@{
    base_url = $BaseUrl
    frontend_url = $FrontendUrl
    database_reseed = 'ok'
    first_seed_method = $seedMethod
    second_seed_method = $secondSeedMethod
    smoke_test_backend = 'ok'
    rust_role_matrix = 'ok'
    smtp_hosted = 'ok'
    tms_workers_hosted = 'ok'
    stripe_hosted_release = 'ok'
    result = 'ok'
} | ConvertTo-Json -Depth 5
