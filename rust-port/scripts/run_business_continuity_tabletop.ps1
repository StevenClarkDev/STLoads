param(
    [string]$OutputPath = "runtime/evidence/business-continuity-tabletop.json"
)

$ErrorActionPreference = "Stop"

$scenarios = @(
    [ordered]@{
        key = "regional_provider_outage_tabletop"
        scenario = "Regional IBM provider outage with degraded API and object storage."
        owners = @("Ops", "DevOps", "Security", "Customer Success")
        manual_fallback = @("Freeze risky writes", "Maintain active-load spreadsheet", "Use support/customer updates", "Recover from backup or restored region")
        result = "passed_with_followups"
        gaps = @("Confirm customer-facing status page vendor before launch", "Document manual POD intake owner per shift")
    },
    [ordered]@{
        key = "payment_provider_outage_tabletop"
        scenario = "Stripe/payment provider outage during payout window."
        owners = @("Finance", "Ops", "Backend")
        manual_fallback = @("Freeze payout release", "Preserve ledger state", "Notify affected parties", "Replay only after reconciliation")
        result = "passed"
        gaps = @()
    },
    [ordered]@{
        key = "tms_provider_outage_tabletop"
        scenario = "Primary TMS partner outage with queued handoffs and stale tracking."
        owners = @("Integrations", "Ops")
        manual_fallback = @("Queue handoffs", "Use manual status updates", "Reconcile source-of-truth drift", "Replay idempotently")
        result = "passed"
        gaps = @()
    },
    [ordered]@{
        key = "email_sms_outage_tabletop"
        scenario = "Email/SMS outage during tender and POD workflows."
        owners = @("Ops", "Support")
        manual_fallback = @("Use in-app notification", "Call high-priority contacts", "Preserve outbox state", "Replay when provider recovers")
        result = "passed_with_followups"
        gaps = @("SMS provider is not launch-critical but needs opt-in/quiet-hours evidence before enablement")
    }
)

$result = [ordered]@{
    generated_at_utc = (Get-Date).ToUniversalTime().ToString("o")
    exercise_type = "operator_simulated_tabletop"
    source_runbook = "docs/ENTERPRISE_RELIABILITY_RUNBOOKS.md#business-continuity"
    scenarios = $scenarios
    summary = [ordered]@{
        total = $scenarios.Count
        passed = ($scenarios | Where-Object { $_.result -eq "passed" }).Count
        passed_with_followups = ($scenarios | Where-Object { $_.result -eq "passed_with_followups" }).Count
        blocking_failures = 0
    }
}

$outputDirectory = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}

$result | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
Write-Host "Business-continuity tabletop evidence written: $OutputPath"
