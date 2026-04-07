param(
    [string]$BaseUrl = 'http://127.0.0.1:3001',
    [string]$AdminEmail = 'admin.smoke@stloads.test',
    [string]$AdminPassword = 'AdminPass123!',
    [string]$ShipperEmail = 'shipper.smoke@stloads.test',
    [string]$ShipperPassword = 'ShipperPass123!',
    [string]$CarrierEmail = 'carrier.smoke@stloads.test',
    [string]$CarrierPassword = 'CarrierPass123!',
    [long]$BookingLegId = 9311,
    [long]$OfferLegConversationId = 9401,
    [long]$OfferId = 9501,
    [long]$SeededHandoffId = 9601,
    [long]$SeededSyncErrorId = 9701,
    [string]$StripeWebhookSecret = '',
    [string]$TmsSharedSecret = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$BaseUrl = $BaseUrl.TrimEnd('/')
$timestamp = Get-Date -Format 'yyyyMMddHHmmss'
$newTmsLoadId = "TMS-SMOKE-$timestamp"
$newExternalHandoffId = "smoke-handoff-$timestamp"
$newPaymentIntentId = "pi_smoke_$timestamp"
$newTransferGroup = "smoke_transfer_group_$timestamp"
$newTransferId = "tr_smoke_$timestamp"

function Write-Step {
    param([string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Assert-Envelope {
    param(
        [Parameter(Mandatory = $true)]$Response,
        [Parameter(Mandatory = $true)][string]$Context
    )

    if ($null -eq $Response) {
        throw "$Context returned no response body."
    }

    if ($Response.status -ne 'ok') {
        throw "$Context returned unexpected envelope status '$($Response.status)'."
    }

    return $Response.data
}

function Invoke-StloadsApi {
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Path,
        [object]$Body = $null,
        [string]$BearerToken = '',
        [hashtable]$ExtraHeaders = @{}
    )

    $uri = "$BaseUrl$Path"
    $headers = @{}

    foreach ($key in $ExtraHeaders.Keys) {
        $headers[$key] = $ExtraHeaders[$key]
    }

    if (-not [string]::IsNullOrWhiteSpace($BearerToken)) {
        $headers['Authorization'] = "Bearer $BearerToken"
    }

    $invokeParams = @{
        Uri = $uri
        Method = $Method
        Headers = $headers
        ContentType = 'application/json'
    }

    if ($null -ne $Body) {
        $invokeParams.Body = ($Body | ConvertTo-Json -Depth 12)
    }

    return Invoke-RestMethod @invokeParams
}

function Invoke-StloadsHealth {
    $uri = "$BaseUrl/health"
    return Invoke-RestMethod -Uri $uri -Method Get
}

function Login-StloadsUser {
    param(
        [Parameter(Mandatory = $true)][string]$Email,
        [Parameter(Mandatory = $true)][string]$Password,
        [Parameter(Mandatory = $true)][string]$Label
    )

    $response = Invoke-StloadsApi -Method Post -Path '/auth/login' -Body @{
        email = $Email
        password = $Password
    }
    $data = Assert-Envelope -Response $response -Context "Login ($Label)"

    if (-not $data.success) {
        throw "Login failed for $Label: $($data.message)"
    }

    if ([string]::IsNullOrWhiteSpace($data.token)) {
        throw "Login succeeded for $Label but no token was returned."
    }

    return $data
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

Write-Step 'Checking backend health'
$health = Invoke-StloadsHealth
Assert-Flag -Condition ($health.status -eq 'ok') -Message 'Health endpoint did not report ok.'
Write-Host ("Health: deployment_target={0}, environment={1}, database_state={2}" -f $health.deployment_target, $health.environment, $health.database_state)

Write-Step 'Logging in seeded smoke users'
$adminLogin = Login-StloadsUser -Email $AdminEmail -Password $AdminPassword -Label 'admin'
$shipperLogin = Login-StloadsUser -Email $ShipperEmail -Password $ShipperPassword -Label 'shipper'
$carrierLogin = Login-StloadsUser -Email $CarrierEmail -Password $CarrierPassword -Label 'carrier'

$adminToken = $adminLogin.token
$shipperToken = $shipperLogin.token
$carrierToken = $carrierLogin.token

Write-Step 'Validating auth session resolution'
$adminSession = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/auth/session' -BearerToken $adminToken) -Context 'Admin session'
$shipperSession = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/auth/session' -BearerToken $shipperToken) -Context 'Shipper session'
$carrierSession = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/auth/session' -BearerToken $carrierToken) -Context 'Carrier session'
Assert-Flag -Condition ($adminSession.authenticated -and $adminSession.user.role_key -eq 'admin') -Message 'Admin session did not resolve correctly.'
Assert-Flag -Condition ($shipperSession.authenticated -and $shipperSession.user.role_key -eq 'shipper') -Message 'Shipper session did not resolve correctly.'
Assert-Flag -Condition ($carrierSession.authenticated -and $carrierSession.user.role_key -eq 'carrier') -Message 'Carrier session did not resolve correctly.'
Write-Host 'Auth sessions resolved for admin, shipper, and carrier.'

Write-Step 'Loading shipper and carrier board views'
$shipperBoard = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/dispatch/load-board?tab=all' -BearerToken $shipperToken) -Context 'Shipper load board'
$carrierBoard = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/dispatch/load-board?tab=all' -BearerToken $carrierToken) -Context 'Carrier load board'
Write-Host ("Load board rows: shipper={0}, carrier={1}" -f $shipperBoard.rows.Count, $carrierBoard.rows.Count)

Write-Step 'Booking the open smoke leg as the carrier'
$bookLeg = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/dispatch/load-board/$BookingLegId/book" -BearerToken $carrierToken -Body @{
    booked_amount = 2850.00
}) -Context 'Book load leg'
Assert-Flag -Condition ($bookLeg.success) -Message "Carrier booking failed: $($bookLeg.message)"
Write-Host $bookLeg.message

Write-Step 'Funding, holding, and releasing escrow through the payments routes'
$fundEscrow = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/payments/legs/$BookingLegId/fund" -BearerToken $adminToken -Body @{
    amount_cents = 285000
    currency = 'USD'
    platform_fee_cents = 15000
    payment_intent_id = $newPaymentIntentId
    charge_id = "ch_smoke_$timestamp"
    transfer_group = $newTransferGroup
    note = 'Smoke test funding step.'
}) -Context 'Fund escrow'
Assert-Flag -Condition ($fundEscrow.success) -Message "Escrow funding failed: $($fundEscrow.message)"

$holdEscrow = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/payments/legs/$BookingLegId/hold" -BearerToken $adminToken -Body @{
    note = 'Smoke test hold step.'
}) -Context 'Hold escrow'
Assert-Flag -Condition ($holdEscrow.success) -Message "Escrow hold failed: $($holdEscrow.message)"

$releaseEscrow = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/payments/legs/$BookingLegId/release" -BearerToken $adminToken -Body @{
    transfer_id = $newTransferId
    note = 'Smoke test release step.'
}) -Context 'Release escrow'
Assert-Flag -Condition ($releaseEscrow.success) -Message "Escrow release failed: $($releaseEscrow.message)"
Write-Host 'Escrow fund/hold/release path completed.'

Write-Step 'Testing Stripe webhook account sync'
$stripeWebhookHeaders = @{}
$stripeWebhookToken = $adminToken
if (-not [string]::IsNullOrWhiteSpace($StripeWebhookSecret)) {
    $stripeWebhookHeaders['x-stripe-webhook-secret'] = $StripeWebhookSecret
    $stripeWebhookToken = ''
}
$stripeWebhook = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/payments/webhooks/stripe' -BearerToken $stripeWebhookToken -ExtraHeaders $stripeWebhookHeaders -Body @{
    event_type = 'account.updated'
    stripe_account_id = 'acct_smoke_carrier_9103'
    payouts_enabled = $true
    kyc_status = 'verified'
    note = 'Smoke test account.updated webhook.'
}) -Context 'Stripe webhook'
Assert-Flag -Condition ($stripeWebhook.acknowledged) -Message "Stripe webhook failed: $($stripeWebhook.message)"
Write-Host $stripeWebhook.message

Write-Step 'Loading chat workspace and sending a carrier message'
$chatWorkspace = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path "/marketplace/chat-workspace?conversation_id=$OfferLegConversationId" -BearerToken $shipperToken) -Context 'Chat workspace'
Write-Host ("Chat workspace loaded with {0} message(s)." -f $chatWorkspace.messages.Count)

$sendMessage = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/marketplace/conversations/$OfferLegConversationId/messages" -BearerToken $carrierToken -Body @{
    body = 'Smoke test carrier reply from PowerShell.'
}) -Context 'Send chat message'
Assert-Flag -Condition ($sendMessage.success) -Message "Chat send failed: $($sendMessage.message)"

$markRead = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/marketplace/conversations/$OfferLegConversationId/read" -BearerToken $shipperToken -Body @{}) -Context 'Mark conversation read'
Assert-Flag -Condition ($markRead.success) -Message "Mark conversation read failed: $($markRead.message)"
Write-Host 'Chat send/read flow completed.'

Write-Step 'Reviewing the seeded offer as the shipper'
$offerReview = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/marketplace/offers/$OfferId/review" -BearerToken $shipperToken -Body @{
    decision = 'accept'
}) -Context 'Review offer'
Assert-Flag -Condition ($offerReview.success) -Message "Offer review failed: $($offerReview.message)"
Write-Host $offerReview.message

Write-Step 'Loading admin operations and reconciliation screens'
$operationsScreen = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/admin/stloads/operations' -BearerToken $adminToken) -Context 'Admin operations screen'
$reconciliationScreen = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/admin/stloads/reconciliation' -BearerToken $adminToken) -Context 'Admin reconciliation screen'
Write-Host ("Admin screens loaded: handoffs={0}, sync_issues={1}, reconciliation_rows={2}" -f $operationsScreen.handoffs.Count, $operationsScreen.recent_sync_issues.Count, $reconciliationScreen.logs.Count)

Write-Step 'Resolving the seeded sync error'
$resolveSyncError = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/admin/stloads/sync-errors/$SeededSyncErrorId/resolve" -BearerToken $adminToken -Body @{
    resolution_note = 'Resolved during PostgreSQL smoke validation.'
}) -Context 'Resolve sync error'
Assert-Flag -Condition ($resolveSyncError.success) -Message "Sync error resolution failed: $($resolveSyncError.message)"
Write-Host $resolveSyncError.message

Write-Step 'Exercising seeded withdraw and close handoff mutations'
$tmsHeaders = @{}
$tmsToken = $adminToken
if (-not [string]::IsNullOrWhiteSpace($TmsSharedSecret)) {
    $tmsHeaders['x-tms-shared-secret'] = $TmsSharedSecret
    $tmsToken = ''
}

$withdraw = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/tms/withdraw' -BearerToken $tmsToken -ExtraHeaders $tmsHeaders -Body @{
    handoff_id = $SeededHandoffId
    reason = 'Smoke test withdraw step.'
    pushed_by = 'smoke-script'
    source_module = 'smoke_test_backend.ps1'
}) -Context 'Withdraw handoff'
Assert-Flag -Condition ($withdraw.success) -Message "TMS withdraw failed: $($withdraw.message)"

$close = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/tms/close' -BearerToken $tmsToken -ExtraHeaders $tmsHeaders -Body @{
    handoff_id = $SeededHandoffId
    reason = 'Smoke test close step.'
    pushed_by = 'smoke-script'
    source_module = 'smoke_test_backend.ps1'
}) -Context 'Close handoff'
Assert-Flag -Condition ($close.success) -Message "TMS close failed: $($close.message)"
Write-Host 'Seeded handoff withdraw/close flow completed.'

Write-Step 'Pushing a new handoff and reconciling it through webhook + requeue flows'
$pushPayload = [ordered]@{
    tms_load_id = $newTmsLoadId
    tenant_id = 'demo-tenant'
    external_handoff_id = $newExternalHandoffId
    party_type = 'shipper'
    freight_mode = 'truckload'
    equipment_type = 'Dry Van'
    commodity_description = 'Smoke test packaged goods'
    weight = 41000.0
    weight_unit = 'lbs'
    piece_count = 22
    is_hazardous = $false
    temperature_data = $null
    container_data = $null
    securement_data = $null
    pickup_city = 'Newark'
    pickup_state = 'NJ'
    pickup_zip = '07114'
    pickup_country = 'US'
    pickup_address = '100 Port Way, Newark, NJ'
    pickup_window_start = (Get-Date).ToUniversalTime().AddHours(8).ToString('o')
    pickup_window_end = (Get-Date).ToUniversalTime().AddHours(10).ToString('o')
    pickup_instructions = 'Check in at dock 7'
    pickup_appointment_ref = 'APT-PICKUP-SMOKE'
    dropoff_city = 'Chicago'
    dropoff_state = 'IL'
    dropoff_zip = '60601'
    dropoff_country = 'US'
    dropoff_address = '400 Market Ave, Chicago, IL'
    dropoff_window_start = (Get-Date).ToUniversalTime().AddDays(1).AddHours(8).ToString('o')
    dropoff_window_end = (Get-Date).ToUniversalTime().AddDays(1).AddHours(12).ToString('o')
    dropoff_instructions = 'Delivery by noon if possible'
    dropoff_appointment_ref = 'APT-DROPOFF-SMOKE'
    board_rate = 3350.0
    rate_currency = 'USD'
    accessorial_flags = @{ lumper = $true }
    bid_type = 'Fixed'
    quote_status = 'open'
    tender_posture = 'tendered'
    compliance_passed = $true
    compliance_summary = @{ passed = $true; notes = @('seeded for smoke test') }
    required_documents_status = @{ bol = 'required'; pod = 'required' }
    readiness = 'ready'
    pushed_by = 'smoke-script'
    push_reason = 'PostgreSQL smoke validation'
    source_module = 'smoke_test_backend.ps1'
    payload_version = '1.0'
    external_refs = @(
        @{ ref_type = 'load_number'; ref_value = "SMOKE-TMS-$timestamp"; ref_source = 'smoke_script' }
    )
}
$pushHandoff = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/tms/push' -BearerToken $tmsToken -ExtraHeaders $tmsHeaders -Body $pushPayload) -Context 'Push handoff'
Assert-Flag -Condition ($pushHandoff.success) -Message "TMS push failed: $($pushHandoff.message)"
$newHandoffId = [long]$pushHandoff.handoff_id

$statusWebhook = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/tms/webhook/status' -BearerToken $tmsToken -ExtraHeaders $tmsHeaders -Body @{
    tms_load_id = $newTmsLoadId
    tenant_id = 'demo-tenant'
    tms_status = 'dispatched'
    status_at = (Get-Date).ToUniversalTime().ToString('o')
    source_module = 'smoke_test_backend.ps1'
    pushed_by = 'smoke-script'
    detail = 'Rate update webhook from smoke test.'
    rate_update = 3450.0
}) -Context 'Status webhook'
Assert-Flag -Condition ($statusWebhook.success) -Message "TMS status webhook failed: $($statusWebhook.message)"

$requeue = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/tms/requeue' -BearerToken $tmsToken -ExtraHeaders $tmsHeaders -Body @{
    handoff_id = $newHandoffId
    pushed_by = 'smoke-script'
    source_module = 'smoke_test_backend.ps1'
}) -Context 'Requeue handoff'
Assert-Flag -Condition ($requeue.success) -Message "TMS requeue failed: $($requeue.message)"

$cancelWebhook = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/api/stloads/webhook/cancel' -BearerToken $tmsToken -ExtraHeaders $tmsHeaders -Body @{
    tms_load_id = $newTmsLoadId
    tenant_id = 'demo-tenant'
    reason = 'Smoke test cancel webhook.'
    pushed_by = 'smoke-script'
    source_module = 'smoke_test_backend.ps1'
}) -Context 'Cancel webhook'
Assert-Flag -Condition ($cancelWebhook.success) -Message "TMS cancel webhook failed: $($cancelWebhook.message)"

$closeWebhook = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/api/stloads/webhook/close' -BearerToken $tmsToken -ExtraHeaders $tmsHeaders -Body @{
    tms_load_id = $newTmsLoadId
    tenant_id = 'demo-tenant'
    reason = 'Smoke test close webhook.'
    pushed_by = 'smoke-script'
    source_module = 'smoke_test_backend.ps1'
}) -Context 'Close webhook'
Assert-Flag -Condition ($closeWebhook.success) -Message "TMS close webhook failed: $($closeWebhook.message)"
Write-Host 'TMS push/webhook/requeue/cancel/close flow completed.'

Write-Step 'Final summary'
$summary = [ordered]@{
    base_url = $BaseUrl
    admin_user = $AdminEmail
    shipper_user = $ShipperEmail
    carrier_user = $CarrierEmail
    booking_leg_id = $BookingLegId
    offer_id = $OfferId
    seeded_sync_error_id = $SeededSyncErrorId
    seeded_handoff_id = $SeededHandoffId
    dynamic_tms_load_id = $newTmsLoadId
    dynamic_handoff_id = $newHandoffId
    result = 'ok'
}
$summary | ConvertTo-Json -Depth 5

