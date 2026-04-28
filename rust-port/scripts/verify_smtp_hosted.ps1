param(
    [string]$BaseUrl = 'https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud',
    [string]$DatabaseUrl = '',
    [int]$PollSeconds = 10,
    [int]$TimeoutSeconds = 180
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$BaseUrl = $BaseUrl.TrimEnd('/')

function Write-Step {
    param([string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Assert-Flag {
    param(
        [Parameter(Mandatory = $true)][bool]$Condition,
        [Parameter(Mandatory = $true)][string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function Resolve-DatabaseUrl {
    param([string]$Value)

    if (-not [string]::IsNullOrWhiteSpace($Value)) {
        return $Value.Trim()
    }

    if (-not [string]::IsNullOrWhiteSpace($env:DATABASE_URL)) {
        return $env:DATABASE_URL.Trim()
    }

    foreach ($candidate in @('rust-port/.env.ibm.secret', 'rust-port/.env.ibm.runtime', 'rust-port/runtime/stloads-rust-runtime.generated.env')) {
        if (-not (Test-Path $candidate)) {
            continue
        }

        $line = Get-Content $candidate | Where-Object { $_ -match '^DATABASE_URL=' } | Select-Object -First 1
        if ($null -ne $line) {
            return ($line -replace '^DATABASE_URL=', '').Trim()
        }
    }

    throw 'DATABASE_URL was not provided and could not be resolved from env files.'
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

function Invoke-DbScalar {
    param(
        [Parameter(Mandatory = $true)][string]$DbUrl,
        [Parameter(Mandatory = $true)][string]$Sql
    )

    $psql = Resolve-PsqlExecutable
    if (-not [string]::IsNullOrWhiteSpace($psql)) {
        $conninfo = Convert-DbUrlToPsqlConninfo -DbUrl $DbUrl
        $result = & $psql $conninfo -At -F "|" -v ON_ERROR_STOP=1 -c "$Sql"
        if ($LASTEXITCODE -ne 0) {
            throw "psql query failed: $Sql"
        }
        return ($result | Out-String).Trim()
    }

    if (Test-DockerPsqlAvailable) {
        $result = docker run --rm postgres:16-alpine psql "$DbUrl" -At -F "|" -v ON_ERROR_STOP=1 -c "$Sql"
        if ($LASTEXITCODE -ne 0) {
            throw "psql query failed: $Sql"
        }
        return ($result | Out-String).Trim()
    }

    throw 'Neither a local psql client nor a reachable Docker daemon is available for database checks.'
}

function Get-EmailOutboxRow {
    param(
        [Parameter(Mandatory = $true)][string]$DbUrl,
        [Parameter(Mandatory = $true)][string]$Email
    )

    $sqlEmail = $Email.Replace("'", "''")
    $row = Invoke-DbScalar -DbUrl $DbUrl -Sql @"
SELECT
    id::text,
    COALESCE(status, ''),
    COALESCE(attempts::text, ''),
    COALESCE(max_attempts::text, ''),
    COALESCE(last_error, ''),
    COALESCE(template_name, ''),
    COALESCE(subject, ''),
    COALESCE(sent_at::text, '')
FROM email_outbox
WHERE to_email = '$sqlEmail'
ORDER BY id DESC
LIMIT 1;
"@

    if ([string]::IsNullOrWhiteSpace($row)) {
        return $null
    }

    $parts = $row.Split('|')
    [pscustomobject]@{
        id = [long]$parts[0]
        status = $parts[1]
        attempts = if ($parts[2]) { [int]$parts[2] } else { 0 }
        max_attempts = if ($parts[3]) { [int]$parts[3] } else { 0 }
        last_error = $parts[4]
        template_name = $parts[5]
        subject = $parts[6]
        sent_at = $parts[7]
    }
}

function Wait-ForOutboxRow {
    param(
        [Parameter(Mandatory = $true)][string]$DbUrl,
        [Parameter(Mandatory = $true)][string]$Email
    )

    $deadline = (Get-Date).ToUniversalTime().AddSeconds($TimeoutSeconds)
    do {
        $row = Get-EmailOutboxRow -DbUrl $DbUrl -Email $Email
        if ($null -ne $row -and $row.status -in @('sent', 'failed', 'retry', 'processing')) {
            return $row
        }

        Start-Sleep -Seconds $PollSeconds
    } while ((Get-Date).ToUniversalTime() -lt $deadline)

    return (Get-EmailOutboxRow -DbUrl $DbUrl -Email $Email)
}

$DatabaseUrl = Add-DatabaseHostaddrFallback -DbUrl (Resolve-DatabaseUrl -Value $DatabaseUrl)
$email = "smtp.validation.$((Get-Date).ToUniversalTime().ToString('yyyyMMddHHmmss'))@stloads.test"

Write-Step 'Checking hosted backend health'
$health = Invoke-RestMethod -Uri "$BaseUrl/health" -Method Get
Assert-Flag -Condition ($health.status -eq 'ok') -Message 'Hosted health endpoint did not report ok.'
Assert-Flag -Condition ($health.mailer_mode -eq 'smtp') -Message "Hosted mailer mode is '$($health.mailer_mode)' instead of 'smtp'."
Assert-Flag -Condition ($health.mail_outbox -eq 'enabled') -Message "Hosted mail outbox is '$($health.mail_outbox)' instead of 'enabled'."
Write-Host ("Health confirms mailer={0}, outbox={1}" -f $health.mailer_mode, $health.mail_outbox)

Write-Step 'Triggering a registration OTP through hosted SMTP'
$body = @{
    name = 'SMTP Validation'
    email = $email
    password = 'Password123!'
    password_confirmation = 'Password123!'
    role_key = 'shipper'
    phone_no = '555-0199'
    address = '100 SMTP Validation Way'
} | ConvertTo-Json

$response = Invoke-RestMethod -Uri "$BaseUrl/auth/register" -Method Post -ContentType 'application/json' -Body $body
Assert-Flag -Condition ($response.status -eq 'ok') -Message "Register envelope returned '$($response.status)'."
Assert-Flag -Condition ($response.data.success) -Message "Hosted registration mail flow failed: $($response.data.message)"
Write-Host $response.data.message

Write-Step 'Waiting for the email outbox record to settle'
$row = Wait-ForOutboxRow -DbUrl $DatabaseUrl -Email $email
Assert-Flag -Condition ($null -ne $row) -Message "No email_outbox row was written for $email."
Assert-Flag -Condition ($row.template_name -eq 'registration_otp') -Message "Unexpected template '$($row.template_name)' for $email."
Assert-Flag -Condition ($row.status -eq 'sent') -Message "Hosted SMTP validation did not finish in 'sent' state. Final status=$($row.status); last_error=$($row.last_error)"
Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($row.sent_at)) -Message 'Hosted SMTP validation row has no sent_at timestamp.'

Write-Step 'Hosted SMTP validation summary'
$summary = [ordered]@{
    base_url = $BaseUrl
    email = $email
    email_outbox_id = $row.id
    template_name = $row.template_name
    status = $row.status
    attempts = $row.attempts
    sent_at = $row.sent_at
    result = 'ok'
}
$summary | ConvertTo-Json -Depth 5
