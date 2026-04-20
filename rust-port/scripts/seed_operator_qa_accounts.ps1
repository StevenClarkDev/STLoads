param(
    [string]$BaseUrl = 'http://127.0.0.1:3001',
    [string]$AdminEmail = 'admin.smoke@stloads.test',
    [string]$AdminPassword = 'AdminPass123!'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$BaseUrl = $BaseUrl.TrimEnd('/')

$accounts = @(
    @{
        key = 'broker_approved'
        name = 'Rust QA Broker'
        email = 'broker.qa@stloads.test'
        password = 'BrokerQaPass123!'
        role_key = 'broker'
        final_status = 'approved'
        onboarding = @{
            company_name = 'QA Broker Logistics LLC'
            company_address = '400 Broker Plaza, Houston, TX'
            fmcsa_broker_license_no = 'BRK-QA-4401'
            mc_authority_number = 'MC-BROKER-4401'
        }
    }
    @{
        key = 'freight_forwarder_approved'
        name = 'Rust QA Freight Forwarder'
        email = 'forwarder.qa@stloads.test'
        password = 'ForwarderQaPass123!'
        role_key = 'freight_forwarder'
        final_status = 'approved'
        onboarding = @{
            company_name = 'QA Forwarding Group'
            company_address = '500 Forwarder Way, Miami, FL'
            freight_forwarder_license = 'FF-QA-5501'
            customs_license = 'CUS-QA-5501'
        }
    }
    @{
        key = 'pending_otp'
        name = 'Rust QA Pending OTP'
        email = 'pending.otp.qa@stloads.test'
        password = 'PendingOtpQa123!'
        role_key = 'shipper'
        final_status = 'pending_otp'
    }
    @{
        key = 'pending_review'
        name = 'Rust QA Pending Review'
        email = 'pending.review.qa@stloads.test'
        password = 'PendingReviewQa123!'
        role_key = 'carrier'
        final_status = 'pending_review'
        onboarding = @{
            company_name = 'QA Pending Review Carrier'
            company_address = '610 Carrier Circle, Columbus, OH'
            dot_number = 'USDOT-QA-6101'
            mc_number = 'MC-QA-6101'
            equipment_types = 'Dry Van'
        }
    }
    @{
        key = 'revision_requested'
        name = 'Rust QA Revision Requested'
        email = 'revision.requested.qa@stloads.test'
        password = 'RevisionQa123!'
        role_key = 'shipper'
        final_status = 'revision_requested'
        onboarding = @{
            company_name = 'QA Revision Shipper'
            company_address = '710 Revision Road, Atlanta, GA'
            business_entity_id = 'SHIPPER-QA-7101'
            facility_address = '710 Revision Warehouse, Atlanta, GA'
            fulfillment_contact_info = 'dispatch+revision@stloads.test'
        }
    }
    @{
        key = 'rejected'
        name = 'Rust QA Rejected'
        email = 'rejected.qa@stloads.test'
        password = 'RejectedQa123!'
        role_key = 'broker'
        final_status = 'rejected'
        onboarding = @{
            company_name = 'QA Rejected Brokerage'
            company_address = '810 Compliance Lane, Phoenix, AZ'
            fmcsa_broker_license_no = 'BRK-QA-8101'
            mc_authority_number = 'MC-BROKER-8101'
        }
    }
)

function Write-Step {
    param([string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Invoke-StloadsApi {
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Path,
        [object]$Body = $null,
        [string]$BearerToken = ''
    )

    $headers = @{}
    if (-not [string]::IsNullOrWhiteSpace($BearerToken)) {
        $headers['Authorization'] = "Bearer $BearerToken"
    }

    $invokeParams = @{
        Uri = "$BaseUrl$Path"
        Method = $Method
        Headers = $headers
        ContentType = 'application/json'
    }

    if ($null -ne $Body) {
        $invokeParams.Body = ($Body | ConvertTo-Json -Depth 12)
    }

    return Invoke-RestMethod @invokeParams
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
        throw "Login failed for ${Label}: $($data.message)"
    }

    return $data
}

function Find-AdminUser {
    param(
        [Parameter(Mandatory = $true)][string]$BearerToken,
        [Parameter(Mandatory = $true)][string]$Email
    )

    $screen = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/admin/users' -BearerToken $BearerToken) -Context 'Admin user directory'
    return $screen.users | Where-Object { $_.email -eq $Email } | Select-Object -First 1
}

function Ensure-RegisteredAccount {
    param(
        [Parameter(Mandatory = $true)]$Account
    )

    $register = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/auth/register' -Body @{
        name = $Account.name
        email = $Account.email
        password = $Account.password
        password_confirmation = $Account.password
        role_key = $Account.role_key
        phone_no = $null
        address = $null
    }) -Context "Register $($Account.email)"

    if (-not $register.success -and $register.message -notlike '*already exists*') {
        throw "Registration failed for $($Account.email): $($register.message)"
    }

    if ($register.success) {
        if ([string]::IsNullOrWhiteSpace($register.dev_otp)) {
            throw "Registration succeeded for $($Account.email) but no development OTP was returned."
        }

        $verified = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/auth/verify-otp' -Body @{
            email = $Account.email
            otp = $register.dev_otp
            purpose = 'registration'
        }) -Context "Verify OTP $($Account.email)"

        if (-not $verified.success) {
            throw "OTP verification failed for $($Account.email): $($verified.message)"
        }
    }
}

function Ensure-OnboardedPendingReview {
    param(
        [Parameter(Mandatory = $true)]$Account
    )

    Ensure-RegisteredAccount -Account $Account
    $login = Login-StloadsUser -Email $Account.email -Password $Account.password -Label $Account.key
    $token = $login.token

    $screen = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/auth/onboarding-screen' -BearerToken $token) -Context "Onboarding screen $($Account.email)"
    if (-not $screen.can_submit) {
        return $login
    }

    $submit = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/auth/onboarding' -BearerToken $token -Body $Account.onboarding) -Context "Submit onboarding $($Account.email)"
    if (-not $submit.success) {
        throw "Onboarding submission failed for $($Account.email): $($submit.message)"
    }

    return (Login-StloadsUser -Email $Account.email -Password $Account.password -Label $Account.key)
}

function Ensure-AdminCreatedPendingOtp {
    param(
        [Parameter(Mandatory = $true)][string]$AdminToken,
        [Parameter(Mandatory = $true)]$Account
    )

    $existing = Find-AdminUser -BearerToken $AdminToken -Email $Account.email
    if ($null -eq $existing) {
        $create = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/admin/users' -BearerToken $AdminToken -Body @{
            name = $Account.name
            email = $Account.email
            password = $Account.password
            password_confirmation = $Account.password
            role_key = $Account.role_key
            status_key = 'pending_otp'
            phone_no = $null
            address = $null
        }) -Context "Create pending OTP user $($Account.email)"

        if (-not $create.success) {
            throw "Admin create failed for $($Account.email): $($create.message)"
        }
    }

    $otpLogin = Login-StloadsUser -Email $Account.email -Password $Account.password -Label $Account.key
    $otpSession = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/auth/session' -BearerToken $otpLogin.token) -Context "Session $($Account.email)"
    $resend = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path '/auth/otp/resend' -Body @{
        email = $Account.email
        purpose = 'registration'
    }) -Context "OTP resend $($Account.email)"

    [pscustomobject]@{
        user_id = $otpSession.user.id
        status_label = $otpSession.user.account_status_label
        role_key = $otpSession.user.role_key
        otp_resend_message = $resend.message
    }
}

function Ensure-FinalLifecycleState {
    param(
        [Parameter(Mandatory = $true)][string]$AdminToken,
        [Parameter(Mandatory = $true)]$Account
    )

    $userLogin = Ensure-OnboardedPendingReview -Account $Account
    $userSession = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/auth/session' -BearerToken $userLogin.token) -Context "Session $($Account.email)"
    $userId = [int64]$userSession.user.id

    switch ($Account.final_status) {
        'pending_review' {
            return [pscustomobject]@{
                user_id = $userId
                status_label = $userSession.user.account_status_label
                role_key = $userSession.user.role_key
                login_message = $userLogin.message
            }
        }
        'approved' {
            $result = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/admin/users/$userId/review" -BearerToken $AdminToken -Body @{
                decision = 'approve'
                remarks = "Approved for IBM staging parity QA."
            }) -Context "Approve $($Account.email)"
            if (-not $result.success) {
                throw "Approve failed for $($Account.email): $($result.message)"
            }
        }
        'revision_requested' {
            $result = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/admin/users/$userId/review" -BearerToken $AdminToken -Body @{
                decision = 'revision'
                remarks = "Revision requested for parity QA coverage."
            }) -Context "Revision $($Account.email)"
            if (-not $result.success) {
                throw "Revision request failed for $($Account.email): $($result.message)"
            }
        }
        'rejected' {
            $result = Assert-Envelope -Response (Invoke-StloadsApi -Method Post -Path "/admin/users/$userId/review" -BearerToken $AdminToken -Body @{
                decision = 'reject'
                remarks = "Rejected for parity QA coverage."
            }) -Context "Reject $($Account.email)"
            if (-not $result.success) {
                throw "Reject failed for $($Account.email): $($result.message)"
            }
        }
        default {
            throw "Unsupported final status '$($Account.final_status)' for $($Account.email)."
        }
    }

    $verifiedLogin = Login-StloadsUser -Email $Account.email -Password $Account.password -Label $Account.key
    $verifiedSession = Assert-Envelope -Response (Invoke-StloadsApi -Method Get -Path '/auth/session' -BearerToken $verifiedLogin.token) -Context "Verified session $($Account.email)"

    [pscustomobject]@{
        user_id = $userId
        status_label = $verifiedSession.user.account_status_label
        role_key = $verifiedSession.user.role_key
        login_message = $verifiedLogin.message
    }
}

Write-Step 'Signing into the hosted Rust admin account'
$admin = Login-StloadsUser -Email $AdminEmail -Password $AdminPassword -Label 'admin'
$adminToken = $admin.token

$results = @()
foreach ($account in $accounts) {
    Write-Step "Seeding $($account.key)"
    if ($account.final_status -eq 'pending_otp') {
        $result = Ensure-AdminCreatedPendingOtp -AdminToken $adminToken -Account $account
    }
    else {
        $result = Ensure-FinalLifecycleState -AdminToken $adminToken -Account $account
    }

    $results += [pscustomobject]@{
        key = $account.key
        email = $account.email
        password = $account.password
        role_key = $result.role_key
        status_label = $result.status_label
        user_id = $result.user_id
    }
}

Write-Step 'QA operator account matrix'
$results | Sort-Object key | Format-Table -AutoSize
