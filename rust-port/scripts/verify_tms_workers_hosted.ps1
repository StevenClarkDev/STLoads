param(
    [string]$BaseUrl = 'https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud',
    [string]$AdminEmail = 'admin.smoke@stloads.test',
    [string]$AdminPassword = 'AdminPass123!',
    [string]$DatabaseUrl = '',
    [int]$RetryTimeoutSeconds = 420,
    [int]$ReconciliationTimeoutSeconds = 420,
    [int]$PollSeconds = 20
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

    foreach ($candidate in @('rust-port/.env.ibm.secret', 'rust-port/.env.ibm.runtime')) {
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

function Invoke-StloadsApi {
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Path,
        [string]$BearerToken = '',
        [object]$Body = $null
    )

    $headers = @{}
    if (-not [string]::IsNullOrWhiteSpace($BearerToken)) {
        $headers['Authorization'] = "Bearer $BearerToken"
    }

    $params = @{
        Uri = "$BaseUrl$Path"
        Method = $Method
        Headers = $headers
    }

    if ($null -ne $Body) {
        $params.ContentType = 'application/json'
        $params.Body = ($Body | ConvertTo-Json -Depth 12)
    }

    Invoke-RestMethod @params
}

function Assert-Envelope {
    param(
        [Parameter(Mandatory = $true)]$Response,
        [Parameter(Mandatory = $true)][string]$Context
    )

    if ($null -eq $Response) {
        throw "$Context returned no body."
    }

    if ($Response.status -ne 'ok') {
        throw "$Context returned unexpected status '$($Response.status)'."
    }

    return $Response.data
}

function Login-StloadsUser {
    param(
        [Parameter(Mandatory = $true)][string]$Email,
        [Parameter(Mandatory = $true)][string]$Password
    )

    $response = Invoke-StloadsApi -Method Post -Path '/auth/login' -Body @{
        email = $Email
        password = $Password
    }
    $data = Assert-Envelope -Response $response -Context 'Admin login'
    Assert-Flag -Condition ($data.success) -Message "Admin login failed: $($data.message)"
    Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($data.token)) -Message 'Admin login returned no token.'
    return $data.token
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
            throw "psql command failed: $Sql"
        }
        return
    }

    if (Test-DockerPsqlAvailable) {
        $null = docker run --rm postgres:16-alpine psql "$DbUrl" -v ON_ERROR_STOP=1 -c "$Sql"
        if ($LASTEXITCODE -ne 0) {
            throw "psql command failed: $Sql"
        }
        return
    }

    throw 'Neither a local psql client nor a reachable Docker daemon is available for database writes.'
}

function Get-HandoffState {
    param(
        [Parameter(Mandatory = $true)][string]$DbUrl,
        [Parameter(Mandatory = $true)][long]$HandoffId
    )

    $row = Invoke-DbScalar -DbUrl $DbUrl -Sql @"
SELECT
    COALESCE(status, ''),
    COALESCE(retry_count::text, ''),
    COALESCE(load_id::text, ''),
    COALESCE(last_push_result, ''),
    COALESCE(tms_status, '')
FROM stloads_handoffs
WHERE id = $HandoffId
LIMIT 1;
"@

    if ([string]::IsNullOrWhiteSpace($row)) {
        return $null
    }

    $parts = $row.Split('|')
    [pscustomobject]@{
        status = $parts[0]
        retry_count = if ($parts[1]) { [int]$parts[1] } else { 0 }
        load_id = if ($parts[2]) { [long]$parts[2] } else { $null }
        last_push_result = $parts[3]
        tms_status = $parts[4]
    }
}

function Get-LoadDeletedAt {
    param(
        [Parameter(Mandatory = $true)][string]$DbUrl,
        [Parameter(Mandatory = $true)][long]$LoadId
    )

    $value = Invoke-DbScalar -DbUrl $DbUrl -Sql @"
SELECT
    COALESCE(
        to_char(deleted_at, 'YYYY-MM-DD') || 'T' || to_char(deleted_at, 'HH24:MI:SS'),
        ''
    )
FROM loads
WHERE id = $LoadId
LIMIT 1;
"@
    if ([string]::IsNullOrWhiteSpace($value)) {
        return $null
    }
    return $value
}

function Get-ReconciliationEvidence {
    param(
        [Parameter(Mandatory = $true)][string]$DbUrl,
        [Parameter(Mandatory = $true)][long]$HandoffId
    )

    $row = Invoke-DbScalar -DbUrl $DbUrl -Sql @"
SELECT
    COALESCE(action, ''),
    COALESCE(triggered_by, ''),
    COALESCE(stloads_status_to, ''),
    COALESCE(detail, '')
FROM stloads_reconciliation_log
WHERE handoff_id = $HandoffId
ORDER BY id DESC
LIMIT 1;
"@

    if ([string]::IsNullOrWhiteSpace($row)) {
        return $null
    }

    $parts = $row.Split('|')
    [pscustomobject]@{
        action = $parts[0]
        triggered_by = $parts[1]
        stloads_status_to = $parts[2]
        detail = $parts[3]
    }
}

function New-TmsPayload {
    param(
        [Parameter(Mandatory = $true)][string]$TmsLoadId,
        [Parameter(Mandatory = $true)][string]$ExternalHandoffId
    )

    $pickupStart = (Get-Date).ToUniversalTime().AddHours(4).ToString('o')
    $pickupEnd = (Get-Date).ToUniversalTime().AddHours(6).ToString('o')
    $dropoffStart = (Get-Date).ToUniversalTime().AddDays(1).AddHours(5).ToString('o')
    $dropoffEnd = (Get-Date).ToUniversalTime().AddDays(1).AddHours(8).ToString('o')

    return [ordered]@{
        tms_load_id = $TmsLoadId
        tenant_id = 'demo-tenant'
        external_handoff_id = $ExternalHandoffId
        party_type = 'shipper'
        freight_mode = 'truckload'
        equipment_type = 'Dry Van'
        commodity_description = 'Hosted TMS worker validation freight'
        weight = 40250.0
        weight_unit = 'lbs'
        piece_count = 18
        is_hazardous = $false
        temperature_data = $null
        container_data = $null
        securement_data = $null
        pickup_city = 'Dallas'
        pickup_state = 'TX'
        pickup_zip = '75201'
        pickup_country = 'US'
        pickup_address = '100 Worker Way, Dallas, TX'
        pickup_window_start = $pickupStart
        pickup_window_end = $pickupEnd
        pickup_instructions = 'Hosted worker validation pickup.'
        pickup_appointment_ref = "APT-PU-$TmsLoadId"
        dropoff_city = 'Memphis'
        dropoff_state = 'TN'
        dropoff_zip = '38103'
        dropoff_country = 'US'
        dropoff_address = '200 Reconcile Ave, Memphis, TN'
        dropoff_window_start = $dropoffStart
        dropoff_window_end = $dropoffEnd
        dropoff_instructions = 'Hosted worker validation delivery.'
        dropoff_appointment_ref = "APT-DO-$TmsLoadId"
        board_rate = 3200.0
        rate_currency = 'USD'
        accessorial_flags = @{ detention = $false }
        bid_type = 'Fixed'
        quote_status = 'open'
        tender_posture = 'tendered'
        compliance_passed = $true
        compliance_summary = @{ passed = $true; notes = @('hosted worker validation') }
        required_documents_status = @{ bol = 'required'; pod = 'required' }
        readiness = 'ready'
        pushed_by = 'verify_tms_workers_hosted.ps1'
        push_reason = 'Hosted TMS worker validation'
        source_module = 'verify_tms_workers_hosted.ps1'
        payload_version = '1.0'
        external_refs = @(
            @{ ref_type = 'validation'; ref_value = $TmsLoadId; ref_source = 'verify_tms_workers_hosted.ps1' }
        )
    }
}

function Wait-ForRetryWorker {
    param(
        [Parameter(Mandatory = $true)][string]$DbUrl,
        [Parameter(Mandatory = $true)][long]$HandoffId
    )

    $deadline = (Get-Date).ToUniversalTime().AddSeconds($RetryTimeoutSeconds)
    do {
        $state = Get-HandoffState -DbUrl $DbUrl -HandoffId $HandoffId
        if ($null -ne $state -and $state.status -eq 'published' -and $null -ne $state.load_id) {
            return $state
        }

        Start-Sleep -Seconds $PollSeconds
    } while ((Get-Date).ToUniversalTime() -lt $deadline)

    return (Get-HandoffState -DbUrl $DbUrl -HandoffId $HandoffId)
}

function Wait-ForReconciliationWorker {
    param(
        [Parameter(Mandatory = $true)][string]$DbUrl,
        [Parameter(Mandatory = $true)][long]$HandoffId
    )

    $deadline = (Get-Date).ToUniversalTime().AddSeconds($ReconciliationTimeoutSeconds)
    do {
        $state = Get-HandoffState -DbUrl $DbUrl -HandoffId $HandoffId
        $evidence = Get-ReconciliationEvidence -DbUrl $DbUrl -HandoffId $HandoffId

        if (
            $null -ne $state -and
            $state.status -eq 'withdrawn' -and
            $null -ne $evidence -and
            $evidence.action -eq 'auto_withdraw' -and
            $evidence.triggered_by -eq 'rust_tms_reconciliation_worker'
        ) {
            return [pscustomobject]@{
                state = $state
                evidence = $evidence
            }
        }

        Start-Sleep -Seconds $PollSeconds
    } while ((Get-Date).ToUniversalTime() -lt $deadline)

    return [pscustomobject]@{
        state = Get-HandoffState -DbUrl $DbUrl -HandoffId $HandoffId
        evidence = Get-ReconciliationEvidence -DbUrl $DbUrl -HandoffId $HandoffId
    }
}

$DatabaseUrl = Add-DatabaseHostaddrFallback -DbUrl (Resolve-DatabaseUrl -Value $DatabaseUrl)
$timestamp = Get-Date -Format 'yyyyMMddHHmmss'
$retryTmsLoadId = "TMS-WORKER-RETRY-$timestamp"
$retryExternalId = "worker-retry-$timestamp"
$reconcileTmsLoadId = "TMS-WORKER-RECON-$timestamp"
$reconcileExternalId = "worker-recon-$timestamp"

Write-Step 'Checking hosted backend health'
$health = Invoke-RestMethod -Uri "$BaseUrl/health" -Method Get
Assert-Flag -Condition ($health.status -eq 'ok') -Message 'Hosted health endpoint did not report ok.'
Assert-Flag -Condition ($health.tms_retry_worker -eq 'enabled') -Message 'TMS retry worker is not enabled in hosted staging.'
Assert-Flag -Condition ($health.tms_reconciliation_worker -eq 'enabled') -Message 'TMS reconciliation worker is not enabled in hosted staging.'
Write-Host ("Health confirms retry={0}, reconciliation={1}, database={2}" -f $health.tms_retry_worker, $health.tms_reconciliation_worker, $health.database_state)

Write-Step 'Logging in as hosted admin operator'
$adminToken = Login-StloadsUser -Email $AdminEmail -Password $AdminPassword

Write-Step 'Creating a queued handoff for the retry worker'
$queued = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/tms/queue' -BearerToken $adminToken -Body (New-TmsPayload -TmsLoadId $retryTmsLoadId -ExternalHandoffId $retryExternalId)) -Context 'Queue handoff'
Assert-Flag -Condition ($queued.success) -Message "Queued handoff creation failed: $($queued.message)"
$queuedHandoffId = [long]$queued.handoff_id
$queuedInitial = Get-HandoffState -DbUrl $DatabaseUrl -HandoffId $queuedHandoffId
Assert-Flag -Condition (
    $null -ne $queuedInitial -and
    $queuedInitial.status -in @('queued', 'published')
) -Message 'Queued handoff was not persisted before retry validation started.'
Write-Host ("Queued handoff #{0} created for {1} with initial status {2}." -f $queuedHandoffId, $retryTmsLoadId, $queuedInitial.status)

Write-Step 'Waiting for the hosted retry worker to publish the queued handoff'
$queuedFinal = Wait-ForRetryWorker -DbUrl $DatabaseUrl -HandoffId $queuedHandoffId
Assert-Flag -Condition ($null -ne $queuedFinal) -Message 'Queued handoff disappeared before retry validation completed.'
Assert-Flag -Condition ($queuedFinal.status -eq 'published') -Message "Retry worker did not publish the handoff within timeout. Final status: $($queuedFinal.status)"
Assert-Flag -Condition ($null -ne $queuedFinal.load_id) -Message 'Retry worker published the handoff without materializing a load projection.'
Write-Host ("Retry worker published handoff #{0} as load #{1}." -f $queuedHandoffId, $queuedFinal.load_id)

Write-Step 'Creating a published handoff for reconciliation validation'
$published = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/tms/push' -BearerToken $adminToken -Body (New-TmsPayload -TmsLoadId $reconcileTmsLoadId -ExternalHandoffId $reconcileExternalId)) -Context 'Push handoff'
Assert-Flag -Condition ($published.success) -Message "Published handoff creation failed: $($published.message)"
$publishedHandoffId = [long]$published.handoff_id
$publishedState = Get-HandoffState -DbUrl $DatabaseUrl -HandoffId $publishedHandoffId
Assert-Flag -Condition ($null -ne $publishedState -and $publishedState.status -eq 'published') -Message 'Published reconciliation handoff did not materialize correctly.'
Assert-Flag -Condition ($null -ne $publishedState.load_id) -Message 'Published reconciliation handoff has no load projection.'
Write-Host ("Published handoff #{0} created with load #{1}." -f $publishedHandoffId, $publishedState.load_id)

Write-Step 'Injecting a cancelled-upstream drift state for the reconciliation worker'
Invoke-DbNonQuery -DbUrl $DatabaseUrl -Sql @"
UPDATE stloads_handoffs
SET tms_status = 'cancelled',
    tms_status_at = CURRENT_TIMESTAMP,
    updated_at = CURRENT_TIMESTAMP
WHERE id = $publishedHandoffId;
"@

Write-Step 'Waiting for the hosted reconciliation worker to auto-withdraw the drifted handoff'
$reconciliationResult = Wait-ForReconciliationWorker -DbUrl $DatabaseUrl -HandoffId $publishedHandoffId
$reconciledState = $reconciliationResult.state
$reconciliationEvidence = $reconciliationResult.evidence
Assert-Flag -Condition ($null -ne $reconciledState) -Message 'Reconciliation handoff disappeared before validation completed.'
Assert-Flag -Condition ($reconciledState.status -eq 'withdrawn') -Message "Reconciliation worker did not withdraw the cancelled handoff within timeout. Final status: $($reconciledState.status)"
Assert-Flag -Condition ($null -ne $reconciliationEvidence) -Message 'No reconciliation log row was written for the worker validation scenario.'
Assert-Flag -Condition ($reconciliationEvidence.action -eq 'auto_withdraw') -Message "Unexpected reconciliation action '$($reconciliationEvidence.action)'."
Assert-Flag -Condition ($reconciliationEvidence.triggered_by -eq 'rust_tms_reconciliation_worker') -Message "Unexpected reconciliation trigger '$($reconciliationEvidence.triggered_by)'."
$deletedAt = Get-LoadDeletedAt -DbUrl $DatabaseUrl -LoadId $publishedState.load_id
Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($deletedAt)) -Message 'Reconciliation worker withdrew the handoff but did not soft-delete the load projection.'
Write-Host ("Reconciliation worker withdrew handoff #{0} and soft-deleted load #{1}." -f $publishedHandoffId, $publishedState.load_id)

Write-Step 'Hosted TMS worker validation summary'
$summary = [ordered]@{
    base_url = $BaseUrl
    queued_handoff_id = $queuedHandoffId
    queued_final_status = $queuedFinal.status
    queued_load_id = $queuedFinal.load_id
    reconciliation_handoff_id = $publishedHandoffId
    reconciliation_final_status = $reconciledState.status
    reconciliation_action = $reconciliationEvidence.action
    reconciliation_triggered_by = $reconciliationEvidence.triggered_by
    reconciliation_load_deleted_at = $deletedAt
    result = 'ok'
}
$summary | ConvertTo-Json -Depth 5
