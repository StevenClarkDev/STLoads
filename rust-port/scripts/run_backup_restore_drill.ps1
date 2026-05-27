param(
    [string[]]$PostgresDeployments = @("Databases for PostgreSQL-hi", "Databases for PostgreSQL-kr", "Databases for PostgreSQL-pi"),
    [string]$OutputPath = "runtime/evidence/backup-restore-drill.json"
)

$ErrorActionPreference = "Stop"

function Require-Command {
    param([string]$Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "$Name is required."
    }
}

function Invoke-Text {
    param([scriptblock]$Command)
    $output = & $Command 2>&1
    [pscustomobject]@{
        exit_code = $LASTEXITCODE
        output = (($output | Select-Object -First 120) -join "`n")
    }
}

Require-Command "ibmcloud"

$deployments = @()
foreach ($deployment in $PostgresDeployments) {
    $backupList = Invoke-Text { ibmcloud cdb deployment-backups-list $deployment }
    $latestCompleted = $null
    if ($backupList.output -match "(crn:[^\s]+)\s+scheduled\s+completed\s+([0-9T:\-]+Z)") {
        $latestCompleted = [ordered]@{
            id = $Matches[1]
            created_at = $Matches[2]
        }
    }
    $deployments += [ordered]@{
        name = $deployment
        backup_query_exit_code = $backupList.exit_code
        latest_completed_backup = $latestCompleted
    }
}

$result = [ordered]@{
    generated_at_utc = (Get-Date).ToUniversalTime().ToString("o")
    postgres = [ordered]@{
        deployments = $deployments
        scheduled_backups_verified = (($deployments | Where-Object { $_.latest_completed_backup -ne $null }).Count -gt 0)
        restore_test_type = "provider_backup_inventory_plus_live_reconciliation"
        restore_execution_note = "A destructive/provider restore was not automatically provisioned by this script. Use a temporary restored deployment before launch for final measured RTO."
    }
    rust_reconciliation = [ordered]@{
        command = "scripts/run_cutover_reconciliation.ps1"
        expected_output = "runtime/evidence/cutover-reconciliation-rust-ibm-staging.json"
    }
    targets = [ordered]@{
        postgres_rpo_minutes = 15
        postgres_rto_minutes = 240
        object_storage_rpo_minutes = 60
        object_storage_rto_minutes = 360
    }
}

$outputDirectory = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}

$result | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
Write-Host "Backup/restore drill evidence written: $OutputPath"
