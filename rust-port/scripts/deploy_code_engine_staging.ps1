param(
    [string]$ProjectName = 'stloads-rust-staging',
    [string]$BuildContextName = '.ce-build-stloads',
    [string]$RuntimeSecretName = 'stloads-rust-runtime',
    [string]$BackendAppName = 'stloads-rust-backend',
    [string]$FrontendAppName = 'stloads-rust-frontend',
    [string]$BackendOutboundBaseUrl = 'https://dispatch-api.268io0zej89v.us-south.codeengine.appdomain.cloud',
    [string]$FrontendPublicUrl = 'https://portal.stloads.com',
    [string]$GoogleMapsApiKey = '',
    [switch]$SkipBackend,
    [switch]$SkipFrontend,
    [switch]$KeepBuildContext,
    [switch]$WhatIf
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Root = Resolve-Path (Join-Path $PSScriptRoot '..')
$Repo = Resolve-Path (Join-Path $Root '..')
$BuildContext = Join-Path $Repo $BuildContextName

function Write-Step {
    param([string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Remove-BuildContext {
    if (-not (Test-Path $BuildContext)) {
        return
    }

    $resolved = Resolve-Path $BuildContext
    if (-not $resolved.Path.StartsWith($Repo.Path)) {
        throw "Refusing to remove build context outside repo: $($resolved.Path)"
    }
    Remove-Item -LiteralPath $resolved.Path -Recurse -Force
}

function Invoke-Ce {
    param([Parameter(Mandatory = $true)][string[]]$Arguments)

    if ($WhatIf) {
        Write-Host "ibmcloud $($Arguments -join ' ')" -ForegroundColor DarkGray
        return
    }

    & ibmcloud @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "ibmcloud $($Arguments -join ' ') failed with exit code $LASTEXITCODE."
    }
}

function New-BuildContext {
    Write-Step "Preparing Code Engine build context $BuildContextName"
    Remove-BuildContext
    New-Item -ItemType Directory -Force -Path $BuildContext | Out-Null

    $excludeDirs = @('target', '.git', '.trunk', 'dist')
    $excludeFiles = @('*.log')
    robocopy $Root $BuildContext /E /XD $excludeDirs /XF $excludeFiles | Out-Host
    if ($LASTEXITCODE -gt 7) {
        throw "robocopy failed with exit code $LASTEXITCODE."
    }
}

try {
    Write-Step "Selecting Code Engine project $ProjectName"
    Invoke-Ce -Arguments @('ce', 'project', 'select', '--name', $ProjectName)
    if ($WhatIf) {
        Write-Step "Preparing Code Engine build context $BuildContextName"
        Write-Host "Would create clean build context from $($Root.Path) at $BuildContext." -ForegroundColor DarkGray
    } else {
        New-BuildContext
    }

    if (-not $SkipBackend) {
        Write-Step "Deploying backend app $BackendAppName"
        Invoke-Ce -Arguments @(
            'ce', 'app', 'update',
            '--name', $BackendAppName,
            '--build-source', $BuildContext,
            '--build-dockerfile', 'Dockerfile',
            '--env-from-secret', $RuntimeSecretName,
            '--env', "ATMP_OUTBOUND_BASE_URL=$BackendOutboundBaseUrl",
            '--port', '8080',
            '--cpu', '1',
            '--memory', '2G',
            '--min-scale', '1',
            '--max-scale', '2',
            '--request-timeout', '600',
            '--wait'
        )
    }

    if (-not $SkipFrontend) {
        Write-Step "Deploying frontend app $FrontendAppName"
        Invoke-Ce -Arguments @(
            'ce', 'app', 'update',
            '--name', $FrontendAppName,
            '--build-source', $BuildContext,
            '--build-dockerfile', 'Dockerfile.frontend',
            '--env', 'BACKEND_UPSTREAM=http://stloads-rust-backend.28hm0zrfwqqw.svc.cluster.local',
            '--env', 'BACKEND_API_BASE_URL=',
            '--env', "FRONTEND_PUBLIC_URL=$FrontendPublicUrl",
            '--env', "GOOGLE_MAPS_API_KEY=$GoogleMapsApiKey",
            '--port', '8080',
            '--cpu', '1',
            '--memory', '2G',
            '--min-scale', '1',
            '--max-scale', '2',
            '--request-timeout', '600'
        )
        Write-Host "Frontend build submitted. Use 'ibmcloud ce app get --name $FrontendAppName' to follow readiness; Code Engine frontend builds can exceed the CLI wait window." -ForegroundColor Yellow
    }
}
finally {
    if (-not $KeepBuildContext -and -not $WhatIf) {
        Write-Step 'Cleaning Code Engine build context'
        Remove-BuildContext
    }
}
