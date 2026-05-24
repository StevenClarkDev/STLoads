param(
    [string]$DatabaseUrl = $env:DATABASE_URL,
    [string]$Key,
    [switch]$Clear,
    [int]$Limit = 50
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($DatabaseUrl)) {
    throw "DatabaseUrl or DATABASE_URL is required."
}

if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
    throw "psql is required to inspect or clear security throttles."
}

if ($Clear) {
    if ([string]::IsNullOrWhiteSpace($Key)) {
        throw "Key is required when using -Clear."
    }

    $sql = "DELETE FROM security_rate_limits WHERE key = :'key' RETURNING key;"
    & psql $DatabaseUrl -v "key=$Key" -c $sql
    exit $LASTEXITCODE
}

if ([string]::IsNullOrWhiteSpace($Key)) {
    $sql = @"
SELECT key, counter, window_started_at, locked_until, expires_at, updated_at
FROM security_rate_limits
WHERE expires_at > CURRENT_TIMESTAMP
ORDER BY updated_at DESC
LIMIT $Limit;
"@
    & psql $DatabaseUrl -c $sql
    exit $LASTEXITCODE
}

$sql = @"
SELECT key, counter, window_started_at, locked_until, expires_at, updated_at
FROM security_rate_limits
WHERE key = :'key';
"@
& psql $DatabaseUrl -v "key=$Key" -c $sql
exit $LASTEXITCODE
