param(
    [string]$BackendTag = "stloads-backend:ci",
    [string]$FrontendTag = "stloads-frontend:ci"
)

$ErrorActionPreference = "Stop"

Push-Location (Join-Path $PSScriptRoot "..")
try {
    docker build -f Dockerfile -t $BackendTag .
    docker build -f Dockerfile.frontend -t $FrontendTag .
} finally {
    Pop-Location
}
