param(
    [string]$RustDatabaseUrl = $env:DATABASE_URL,
    [string]$ExpectedJsonPath,
    [string]$OutputPath = "rust-port/runtime/cutover-reconciliation.json"
)

$ErrorActionPreference = 'Stop'

function Require-Command {
    param([string]$Name)

    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if (-not $command) {
        throw "$Name is required to run cutover reconciliation."
    }
}

function Test-Command {
    param([string]$Name)

    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Invoke-PsqlJson {
    param(
        [string]$DatabaseUrl,
        [string]$Query
    )

    $output = & psql $DatabaseUrl -X -q -t -A -c $Query
    if ($LASTEXITCODE -ne 0) {
        throw "psql query failed: $Query"
    }

    $text = ($output -join "`n").Trim()
    if ([string]::IsNullOrWhiteSpace($text)) {
        return @()
    }

    return $text | ConvertFrom-Json
}

function Compare-Section {
    param(
        [string]$Name,
        [object]$Actual,
        [object]$Expected
    )

    $actualJson = $Actual | ConvertTo-Json -Depth 20 -Compress
    $expectedJson = $Expected | ConvertTo-Json -Depth 20 -Compress

    [pscustomobject]@{
        section = $Name
        status = $(if ($actualJson -eq $expectedJson) { 'match' } else { 'mismatch' })
        actual = $Actual
        expected = $Expected
    }
}

if ([string]::IsNullOrWhiteSpace($RustDatabaseUrl)) {
    throw 'RustDatabaseUrl or DATABASE_URL is required.'
}

if (-not (Test-Command 'psql')) {
    if (-not (Test-Command 'cargo')) {
        throw 'psql is not installed and cargo is not available for the Rust reconciliation fallback.'
    }

    $arguments = @('run', '-q', '-p', 'backend', '--bin', 'cutover_reconciliation', '--', '--database-url', $RustDatabaseUrl, '--output', $OutputPath)
    if (-not [string]::IsNullOrWhiteSpace($ExpectedJsonPath)) {
        $arguments += @('--expected-json-path', $ExpectedJsonPath)
    }

    & cargo @arguments
    exit $LASTEXITCODE
}

$summary = [ordered]@{
    generated_at_utc = (Get-Date).ToUniversalTime().ToString('o')
    source = 'rust-postgres'
    table_counts = Invoke-PsqlJson $RustDatabaseUrl @"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.domain, rows.table_name), '[]'::jsonb)
FROM (
  SELECT 'identity' AS domain, 'users' AS table_name, COUNT(*)::bigint AS count FROM users
  UNION ALL SELECT 'identity', 'user_details', COUNT(*)::bigint FROM user_details
  UNION ALL SELECT 'identity', 'user_history', COUNT(*)::bigint FROM user_history
  UNION ALL SELECT 'loads', 'loads', COUNT(*)::bigint FROM loads
  UNION ALL SELECT 'loads', 'load_legs', COUNT(*)::bigint FROM load_legs
  UNION ALL SELECT 'loads', 'offers', COUNT(*)::bigint FROM offers
  UNION ALL SELECT 'marketplace', 'conversations', COUNT(*)::bigint FROM conversations
  UNION ALL SELECT 'marketplace', 'messages', COUNT(*)::bigint FROM messages
  UNION ALL SELECT 'documents', 'load_documents', COUNT(*)::bigint FROM load_documents
  UNION ALL SELECT 'documents', 'leg_documents', COUNT(*)::bigint FROM leg_documents
  UNION ALL SELECT 'documents', 'kyc_documents', COUNT(*)::bigint FROM kyc_documents
  UNION ALL SELECT 'payments', 'escrows', COUNT(*)::bigint FROM escrows
  UNION ALL SELECT 'tms', 'stloads_handoffs', COUNT(*)::bigint FROM stloads_handoffs
  UNION ALL SELECT 'tms', 'stloads_handoff_events', COUNT(*)::bigint FROM stloads_handoff_events
  UNION ALL SELECT 'tms', 'stloads_sync_errors', COUNT(*)::bigint FROM stloads_sync_errors
  UNION ALL SELECT 'tms', 'stloads_reconciliation_log', COUNT(*)::bigint FROM stloads_reconciliation_log
  UNION ALL SELECT 'master_data', 'countries', COUNT(*)::bigint FROM countries
  UNION ALL SELECT 'master_data', 'cities', COUNT(*)::bigint FROM cities
  UNION ALL SELECT 'master_data', 'locations', COUNT(*)::bigint FROM locations
  UNION ALL SELECT 'master_data', 'load_types', COUNT(*)::bigint FROM load_types
  UNION ALL SELECT 'master_data', 'equipments', COUNT(*)::bigint FROM equipments
  UNION ALL SELECT 'master_data', 'commodity_types', COUNT(*)::bigint FROM commodity_types
) rows;
"@
    users_by_role_status = Invoke-PsqlJson $RustDatabaseUrl @"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.role_id, rows.status), '[]'::jsonb)
FROM (
  SELECT role_id, status, COUNT(*)::bigint AS count
  FROM users
  GROUP BY role_id, status
) rows;
"@
    loads_by_status = Invoke-PsqlJson $RustDatabaseUrl @"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.status), '[]'::jsonb)
FROM (
  SELECT status, COUNT(*)::bigint AS count
  FROM loads
  WHERE deleted_at IS NULL
  GROUP BY status
) rows;
"@
    legs_by_status = Invoke-PsqlJson $RustDatabaseUrl @"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.status_id), '[]'::jsonb)
FROM (
  SELECT status_id, COUNT(*)::bigint AS count, COALESCE(SUM(booked_amount), 0)::double precision AS booked_amount_total
  FROM load_legs
  WHERE deleted_at IS NULL
  GROUP BY status_id
) rows;
"@
    documents_by_provider = Invoke-PsqlJson $RustDatabaseUrl @"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.document_scope, rows.storage_provider), '[]'::jsonb)
FROM (
  SELECT 'load' AS document_scope, COALESCE(NULLIF(storage_provider, ''), 'unknown') AS storage_provider, COUNT(*)::bigint AS count
  FROM load_documents
  GROUP BY COALESCE(NULLIF(storage_provider, ''), 'unknown')
  UNION ALL
  SELECT 'leg', COALESCE(NULLIF(storage_provider, ''), 'unknown'), COUNT(*)::bigint
  FROM leg_documents
  GROUP BY COALESCE(NULLIF(storage_provider, ''), 'unknown')
  UNION ALL
  SELECT 'kyc', 'profile_kyc', COUNT(*)::bigint
  FROM kyc_documents
) rows;
"@
    payments_by_status = Invoke-PsqlJson $RustDatabaseUrl @"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.status, rows.currency), '[]'::jsonb)
FROM (
  SELECT status, currency, COUNT(*)::bigint AS count, COALESCE(SUM(amount), 0)::double precision AS amount_total
  FROM escrows
  GROUP BY status, currency
) rows;
"@
    tms_by_status = Invoke-PsqlJson $RustDatabaseUrl @"
SELECT COALESCE(jsonb_agg(row_to_json(rows) ORDER BY rows.status, rows.tenant_id), '[]'::jsonb)
FROM (
  SELECT status, tenant_id, COUNT(*)::bigint AS count, COUNT(load_id)::bigint AS materialized_loads
  FROM stloads_handoffs
  GROUP BY status, tenant_id
) rows;
"@
}

$result = [ordered]@{
    status = 'generated'
    summary = $summary
    comparisons = @()
}

if (-not [string]::IsNullOrWhiteSpace($ExpectedJsonPath)) {
    if (-not (Test-Path -LiteralPath $ExpectedJsonPath)) {
        throw "Expected summary file not found: $ExpectedJsonPath"
    }

    $expected = Get-Content -LiteralPath $ExpectedJsonPath -Raw | ConvertFrom-Json
    $comparisons = @()
    foreach ($section in @('table_counts', 'users_by_role_status', 'loads_by_status', 'legs_by_status', 'documents_by_provider', 'payments_by_status', 'tms_by_status')) {
        $comparisons += Compare-Section -Name $section -Actual $summary[$section] -Expected $expected.summary.$section
    }
    $result.comparisons = $comparisons
    $result.status = if (($comparisons | Where-Object { $_.status -ne 'match' }).Count -eq 0) { 'match' } else { 'mismatch' }
}

$outputDirectory = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}

$result | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
Write-Host "Cutover reconciliation $($result.status): $OutputPath"

if ($result.status -eq 'mismatch') {
    exit 1
}
