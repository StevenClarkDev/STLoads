param(
    [string]$BaseUrl = 'https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud',
    [string]$AdminEmail = 'admin.smoke@stloads.test',
    [string]$AdminPassword = 'AdminPass123!',
    [string]$CarrierEmail = 'carrier.smoke@stloads.test',
    [string]$CarrierPassword = 'CarrierPass123!',
    [long]$CarrierUserId = 9103,
    [long]$BookingLegId = 9311,
    [string]$StripeSecret = $env:STRIPE_SECRET,
    [string]$StripeWebhookSecret = $env:STRIPE_WEBHOOK_SECRET_PLATFORM,
    [string]$StripeApiBaseUrl = 'https://api.stripe.com/v1',
    [string]$ReturnUrl = 'https://portal.stloads.com/settings/payouts?done=1',
    [string]$RefreshUrl = 'https://portal.stloads.com/settings/payouts?refresh=1'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$BaseUrl = $BaseUrl.TrimEnd('/')
$StripeApiBaseUrl = $StripeApiBaseUrl.TrimEnd('/')

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
        [string]$RawBody = '',
        [string]$BearerToken = '',
        [hashtable]$ExtraHeaders = @{}
    )

    $headers = @{}
    foreach ($key in $ExtraHeaders.Keys) {
        $headers[$key] = $ExtraHeaders[$key]
    }

    if (-not [string]::IsNullOrWhiteSpace($BearerToken)) {
        $headers['Authorization'] = "Bearer $BearerToken"
    }

    $invokeParams = @{
        Uri = "$BaseUrl$Path"
        Method = $Method
        Headers = $headers
        ContentType = 'application/json'
    }

    if (-not [string]::IsNullOrWhiteSpace($RawBody)) {
        $invokeParams.Body = $RawBody
    }
    elseif ($null -ne $Body) {
        $invokeParams.Body = ($Body | ConvertTo-Json -Depth 20 -Compress)
    }

    Invoke-RestMethod @invokeParams
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

    Assert-Flag -Condition $data.success -Message "Login failed for ${Label}: $($data.message)"
    Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($data.token)) -Message "Login succeeded for $Label but no token was returned."

    return $data
}

function Invoke-StripeForm {
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Path,
        [hashtable]$Form = @{}
    )

    Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($StripeSecret)) -Message 'STRIPE_SECRET is required for hosted Stripe verification.'
    Assert-Flag -Condition ($StripeSecret.StartsWith('sk_test_')) -Message 'Hosted Stripe verification only runs automatically with a Stripe test secret.'

    $headers = @{
        Authorization = "Bearer $StripeSecret"
    }

    $params = @{
        Uri = "$StripeApiBaseUrl$Path"
        Method = $Method
        Headers = $headers
    }

    if ($Method -eq 'Post') {
        $params.ContentType = 'application/x-www-form-urlencoded'
        $params.Body = $Form
    }

    Invoke-RestMethod @params
}

function Get-LatestChargeId {
    param([Parameter(Mandatory = $true)]$PaymentIntent)

    if ($null -eq $PaymentIntent.latest_charge) {
        return $null
    }

    if ($PaymentIntent.latest_charge -is [string]) {
        return $PaymentIntent.latest_charge
    }

    return $PaymentIntent.latest_charge.id
}

function New-StripeSignatureHeader {
    param(
        [Parameter(Mandatory = $true)][string]$Payload,
        [Parameter(Mandatory = $true)][string]$Secret
    )

    Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($Secret)) -Message 'A Stripe webhook secret is required for signed webhook verification.'

    $timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    $signedPayload = "$timestamp.$Payload"
    $hmac = [System.Security.Cryptography.HMACSHA256]::new([System.Text.Encoding]::UTF8.GetBytes($Secret))
    try {
        $hash = $hmac.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($signedPayload))
        $hex = -join ($hash | ForEach-Object { $_.ToString('x2') })
    }
    finally {
        $hmac.Dispose()
    }

    "t=$timestamp,v1=$hex"
}

Write-Step 'Checking hosted backend health'
$health = Invoke-RestMethod -Uri "$BaseUrl/health" -Method Get
Assert-Flag -Condition ($health.status -eq 'ok') -Message 'Hosted backend health did not return ok.'
Assert-Flag -Condition ($health.database_state -eq 'connected') -Message 'Hosted backend database is not connected.'
Write-Host ("Health ok: environment={0}, mailer={1}" -f $health.environment, $health.mailer_mode)

Write-Step 'Logging in smoke users'
$adminLogin = Login-StloadsUser -Email $AdminEmail -Password $AdminPassword -Label 'admin'
$carrierLogin = Login-StloadsUser -Email $CarrierEmail -Password $CarrierPassword -Label 'carrier'
$adminToken = $adminLogin.token
$carrierToken = $carrierLogin.token

Write-Step 'Creating or refreshing Stripe Connect onboarding link'
$connect = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/payments/connect/onboarding-link' -BearerToken $carrierToken -Body @{
    refresh_url = $RefreshUrl
    return_url = $ReturnUrl
}) -Context 'Stripe Connect onboarding link'
Assert-Flag -Condition $connect.success -Message "Stripe Connect onboarding link failed: $($connect.message)"
Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($connect.account_id)) -Message 'Stripe Connect response did not include an account id.'
Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($connect.onboarding_url)) -Message 'Stripe Connect response did not include an onboarding URL.'
Write-Host ("Connect link created for user {0}, account {1}." -f $connect.user_id, $connect.account_id)

Write-Step 'Booking the seeded smoke leg as the carrier'
$book = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/dispatch/load-board/$BookingLegId/book" -BearerToken $carrierToken -Body @{
    booked_amount = 2850.00
}) -Context 'Book smoke leg'
Assert-Flag -Condition $book.success -Message "Smoke booking failed: $($book.message)"
Write-Host $book.message

Write-Step 'Creating live Stripe test PaymentIntent through the hosted Rust funding route'
$fund = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/payments/legs/$BookingLegId/fund" -BearerToken $adminToken -Body @{
    amount_cents = 285000
    currency = 'usd'
    platform_fee_cents = 15000
    transfer_group = "LEG_$BookingLegId"
    note = 'Hosted Stripe verification PaymentIntent creation.'
}) -Context 'Create PaymentIntent'
Assert-Flag -Condition $fund.success -Message "PaymentIntent creation failed: $($fund.message)"
Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($fund.payment_intent_id)) -Message 'Funding response did not include a PaymentIntent id.'
Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($fund.client_secret)) -Message 'Funding response did not include a client secret.'
Write-Host ("PaymentIntent created: {0}" -f $fund.payment_intent_id)

Write-Step 'Confirming the Stripe test PaymentIntent'
$confirm = Invoke-StripeForm -Method Post -Path "/payment_intents/$($fund.payment_intent_id)/confirm" -Form @{
    payment_method = 'pm_card_visa'
    return_url = $ReturnUrl
}
Assert-Flag -Condition ($confirm.status -eq 'succeeded') -Message "PaymentIntent did not succeed after confirmation; status=$($confirm.status)."

$confirmed = Invoke-StripeForm -Method Get -Path "/payment_intents/$($fund.payment_intent_id)?expand[]=latest_charge"
$chargeId = Get-LatestChargeId -PaymentIntent $confirmed
Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($chargeId)) -Message 'Confirmed PaymentIntent did not expose a latest_charge id.'
Write-Host ("PaymentIntent confirmed with charge {0}." -f $chargeId)

Write-Step 'Sending signed Stripe payment_intent.succeeded webhook to hosted Rust backend'
$webhookObject = [ordered]@{
    id = "evt_stloads_hosted_$([DateTimeOffset]::UtcNow.ToUnixTimeSeconds())"
    type = 'payment_intent.succeeded'
    data = [ordered]@{
        object = [ordered]@{
            id = $fund.payment_intent_id
            amount = 285000
            currency = 'usd'
            latest_charge = $chargeId
            transfer_group = "LEG_$BookingLegId"
            application_fee_amount = 15000
            metadata = [ordered]@{
                leg_id = "$BookingLegId"
            }
        }
    }
}
$webhookPayload = $webhookObject | ConvertTo-Json -Depth 20 -Compress
$signature = New-StripeSignatureHeader -Payload $webhookPayload -Secret $StripeWebhookSecret
$webhook = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/payments/webhooks/stripe' -ExtraHeaders @{
    'Stripe-Signature' = $signature
} -RawBody $webhookPayload) -Context 'Signed Stripe webhook'
Assert-Flag -Condition $webhook.acknowledged -Message "Signed Stripe webhook was not acknowledged: $($webhook.message)"
Write-Host $webhook.message

Write-Step 'Releasing escrow through hosted Rust backend'
$release = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/payments/legs/$BookingLegId/release" -BearerToken $adminToken -Body @{
    note = 'Hosted Stripe verification transfer release.'
}) -Context 'Release escrow'
Assert-Flag -Condition $release.success -Message "Escrow release failed: $($release.message)"
Assert-Flag -Condition (-not [string]::IsNullOrWhiteSpace($release.transfer_id)) -Message 'Release succeeded but no transfer id was returned.'
Write-Host ("Escrow released with transfer {0}." -f $release.transfer_id)

Write-Step 'Hosted Stripe verification summary'
[ordered]@{
    base_url = $BaseUrl
    carrier_user_id = $CarrierUserId
    carrier_account_id = $connect.account_id
    leg_id = $BookingLegId
    payment_intent_id = $fund.payment_intent_id
    charge_id = $chargeId
    transfer_id = $release.transfer_id
    result = 'ok'
} | ConvertTo-Json -Depth 5
