param(
    [string]$ContainerName = "stloads-p3-postgres",
    [string]$Database = "stloads_p3_verify",
    [string]$User = "stloads",
    [string]$Password = "stloads_verify_pass",
    [int]$Port = 55432,
    [switch]$KeepContainer
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$existing = docker ps -a --filter "name=^/$ContainerName$" --format "{{.Names}}"
if ($existing -eq $ContainerName) {
    docker rm -f $ContainerName | Out-Null
}

docker run --name $ContainerName `
    -e "POSTGRES_DB=$Database" `
    -e "POSTGRES_USER=$User" `
    -e "POSTGRES_PASSWORD=$Password" `
    -p "${Port}:5432" `
    -d postgres:16-alpine | Out-Null

try {
    $ready = $false
    for ($i = 0; $i -lt 40; $i++) {
        docker exec $ContainerName pg_isready -U $User -d $Database | Out-Null
        if ($LASTEXITCODE -eq 0) {
            $ready = $true
            break
        }
        Start-Sleep -Seconds 1
    }

    if (-not $ready) {
        throw "PostgreSQL container did not become ready."
    }

    docker exec $ContainerName psql -U $User -d $Database -c "CREATE SCHEMA IF NOT EXISTS stloads_verify;" | Out-Null

    $env:DATABASE_URL = "postgres://${User}:${Password}@127.0.0.1:${Port}/${Database}?sslmode=disable&options=-csearch_path%3Dstloads_verify,public"

    Push-Location $root
    try {
        cargo sqlx migrate run --source crates/db/migrations
        if ($LASTEXITCODE -ne 0) {
            throw "cargo sqlx migrate run failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }

    $tables = docker exec $ContainerName psql -U $User -d $Database -tAc "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'stloads_verify';"
    Write-Host "Migration smoke passed. Tables in stloads_verify: $($tables.Trim())"
}
finally {
    if (-not $KeepContainer) {
        docker rm -f $ContainerName | Out-Null
    }
}
