param(
    [string]$BaseUrl = 'https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud',
    [string]$FrontendUrl = 'https://stloads-rust-frontend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud',
    [string]$AdminEmail = 'admin.smoke@stloads.test',
    [string]$AdminPassword = 'AdminPass123!',
    [string]$ShipperEmail = 'shipper.smoke@stloads.test',
    [string]$ShipperPassword = 'ShipperPass123!',
    [string]$CarrierEmail = 'carrier.smoke@stloads.test',
    [string]$CarrierPassword = 'CarrierPass123!',
    [string]$BrokerEmail = 'broker.qa@stloads.test',
    [string]$BrokerPassword = 'BrokerQaPass123!',
    [string]$FreightForwarderEmail = 'forwarder.qa@stloads.test',
    [string]$FreightForwarderPassword = 'ForwarderQaPass123!',
    [string]$PendingOtpEmail = 'pending.otp.qa@stloads.test',
    [string]$PendingOtpPassword = 'PendingOtpQa123!',
    [string]$PendingReviewEmail = 'pending.review.qa@stloads.test',
    [string]$PendingReviewPassword = 'PendingReviewQa123!',
    [string]$RevisionRequestedEmail = 'revision.requested.qa@stloads.test',
    [string]$RevisionRequestedPassword = 'RevisionQa123!',
    [string]$RejectedEmail = 'rejected.qa@stloads.test',
    [string]$RejectedPassword = 'RejectedQa123!'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Net.Http | Out-Null

$BaseUrl = $BaseUrl.TrimEnd('/')
$FrontendUrl = $FrontendUrl.TrimEnd('/')

function Write-Step {
    param([string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function New-HttpClient {
    $handler = [System.Net.Http.HttpClientHandler]::new()
    $handler.AutomaticDecompression = [System.Net.DecompressionMethods]::GZip -bor [System.Net.DecompressionMethods]::Deflate
    $client = [System.Net.Http.HttpClient]::new($handler)
    $client.DefaultRequestHeaders.Add('User-Agent', 'Codex-Rust-QA-Matrix/1.0')
    return @{
        Client = $client
        Handler = $handler
    }
}

function Invoke-JsonRequest {
    param(
        [Parameter(Mandatory = $true)][System.Net.Http.HttpClient]$Client,
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Url,
        [object]$Body = $null,
        [string]$BearerToken = ''
    )

    $request = [System.Net.Http.HttpRequestMessage]::new([System.Net.Http.HttpMethod]::$Method, $Url)
    if (-not [string]::IsNullOrWhiteSpace($BearerToken)) {
        $request.Headers.Authorization = [System.Net.Http.Headers.AuthenticationHeaderValue]::new('Bearer', $BearerToken)
    }

    if ($null -ne $Body) {
        $json = $Body | ConvertTo-Json -Depth 12
        $request.Content = [System.Net.Http.StringContent]::new($json, [System.Text.Encoding]::UTF8, 'application/json')
    }

    $response = $Client.SendAsync($request).GetAwaiter().GetResult()
    $rawBody = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult()

    $jsonBody = $null
    if (-not [string]::IsNullOrWhiteSpace($rawBody)) {
        try {
            $jsonBody = $rawBody | ConvertFrom-Json
        }
        catch {
            $jsonBody = $null
        }
    }

    [pscustomobject]@{
        StatusCode = [int]$response.StatusCode
        IsSuccessStatusCode = $response.IsSuccessStatusCode
        Body = $rawBody
        Json = $jsonBody
    }
}

function Add-Result {
    param(
        [System.Collections.Generic.List[object]]$Results,
        [Parameter(Mandatory = $true)][string]$Account,
        [Parameter(Mandatory = $true)][string]$Check,
        [Parameter(Mandatory = $true)][bool]$Passed,
        [Parameter(Mandatory = $true)][string]$Detail
    )

    $Results.Add([pscustomobject]@{
        account = $Account
        check = $Check
        passed = $Passed
        detail = $Detail
    }) | Out-Null
}

function Test-FrontendRoute {
    param(
        [Parameter(Mandatory = $true)][System.Net.Http.HttpClient]$Client,
        [Parameter(Mandatory = $true)][string]$Path
    )

    Invoke-JsonRequest -Client $Client -Method Get -Url "$FrontendUrl$Path"
}

function Login-RustUser {
    param(
        [Parameter(Mandatory = $true)][System.Net.Http.HttpClient]$Client,
        [Parameter(Mandatory = $true)][string]$Email,
        [Parameter(Mandatory = $true)][string]$Password,
        [Parameter(Mandatory = $true)][string]$Label
    )

    $response = Invoke-JsonRequest -Client $Client -Method Post -Url "$BaseUrl/auth/login" -Body @{
        email = $Email
        password = $Password
    }

    if ($response.StatusCode -ne 200 -or $null -eq $response.Json -or $response.Json.status -ne 'ok') {
        throw "Rust login request for $Label returned unexpected status $($response.StatusCode)."
    }

    if (-not $response.Json.data.success) {
        throw "Rust login failed for ${Label}: $($response.Json.data.message)"
    }

    return $response.Json.data
}

function Expect-RouteStatus {
    param(
        [Parameter(Mandatory = $true)][System.Net.Http.HttpClient]$Client,
        [Parameter(Mandatory = $true)][string]$BearerToken,
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][int[]]$AllowedStatus
    )

    $response = Invoke-JsonRequest -Client $Client -Method Get -Url "$BaseUrl$Path" -BearerToken $BearerToken
    [pscustomobject]@{
        Passed = $AllowedStatus -contains $response.StatusCode
        StatusCode = $response.StatusCode
        Json = $response.Json
        Body = $response.Body
    }
}

$http = New-HttpClient
$client = $http.Client
$results = [System.Collections.Generic.List[object]]::new()

try {
    Write-Step 'Checking hosted frontend and backend health'
    foreach ($frontendPath in @('/', '/loads', '/profile', '/admin/account-lifecycle', '/health')) {
        $frontendResponse = Test-FrontendRoute -Client $client -Path $frontendPath
        Add-Result -Results $results -Account 'frontend' -Check "route $frontendPath" -Passed ($frontendResponse.StatusCode -eq 200) -Detail "HTTP $($frontendResponse.StatusCode)"
    }

    $backendHealth = Invoke-JsonRequest -Client $client -Method Get -Url "$BaseUrl/health"
    Add-Result -Results $results -Account 'backend' -Check 'health' -Passed ($backendHealth.StatusCode -eq 200) -Detail "HTTP $($backendHealth.StatusCode)"

    Write-Step 'Checking approved role accounts against hosted Rust routes'
    $approvedAccounts = @(
        @{
            label = 'admin'
            email = $AdminEmail
            password = $AdminPassword
            expected_role = 'admin'
            session_status = 'Approved'
            allow = @('/auth/session', '/admin/users', '/admin/loads?tab=all', '/master-data/screen', '/admin/stloads/operations')
            deny = @('/none')
        }
        @{
            label = 'shipper'
            email = $ShipperEmail
            password = $ShipperPassword
            expected_role = 'shipper'
            session_status = 'Approved'
            allow = @('/auth/session', '/auth/profile-screen', '/dispatch/load-board?tab=all', '/marketplace/chat-workspace')
            deny = @('/admin/users', '/master-data/screen', '/admin/stloads/operations')
        }
        @{
            label = 'carrier'
            email = $CarrierEmail
            password = $CarrierPassword
            expected_role = 'carrier'
            session_status = 'Approved'
            allow = @('/auth/session', '/auth/profile-screen', '/dispatch/load-board?tab=recommended', '/marketplace/chat-workspace')
            deny = @('/admin/users', '/master-data/screen', '/admin/stloads/operations')
        }
        @{
            label = 'broker'
            email = $BrokerEmail
            password = $BrokerPassword
            expected_role = 'broker'
            session_status = 'Approved'
            allow = @('/auth/session', '/auth/profile-screen', '/dispatch/load-board?tab=all', '/marketplace/chat-workspace')
            deny = @('/admin/users', '/master-data/screen', '/admin/stloads/operations')
        }
        @{
            label = 'freight_forwarder'
            email = $FreightForwarderEmail
            password = $FreightForwarderPassword
            expected_role = 'freight_forwarder'
            session_status = 'Approved'
            allow = @('/auth/session', '/auth/profile-screen', '/dispatch/load-board?tab=all', '/marketplace/chat-workspace', '/admin/stloads/operations')
            deny = @('/admin/users', '/master-data/screen')
        }
    )

    foreach ($account in $approvedAccounts) {
        $login = Login-RustUser -Client $client -Email $account.email -Password $account.password -Label $account.label
        Add-Result -Results $results -Account $account.label -Check 'login' -Passed $true -Detail $login.message
        Add-Result -Results $results -Account $account.label -Check 'session role' -Passed ($login.session.user.role_key -eq $account.expected_role) -Detail "role=$($login.session.user.role_key)"
        Add-Result -Results $results -Account $account.label -Check 'session status' -Passed ($login.session.user.account_status_label -eq $account.session_status) -Detail "status=$($login.session.user.account_status_label)"

        foreach ($path in $account.allow) {
            $response = Expect-RouteStatus -Client $client -BearerToken $login.token -Path $path -AllowedStatus @(200)
            Add-Result -Results $results -Account $account.label -Check "allow $path" -Passed $response.Passed -Detail "HTTP $($response.StatusCode)"
        }

        foreach ($path in $account.deny) {
            if ($path -eq '/none') {
                continue
            }
            $response = Expect-RouteStatus -Client $client -BearerToken $login.token -Path $path -AllowedStatus @(401, 403)
            Add-Result -Results $results -Account $account.label -Check "deny $path" -Passed $response.Passed -Detail "HTTP $($response.StatusCode)"
        }
    }

    Write-Step 'Checking lifecycle-state Rust accounts'
    $lifecycleAccounts = @(
        @{
            label = 'pending_otp'
            email = $PendingOtpEmail
            password = $PendingOtpPassword
            expected_role = 'shipper'
            expected_status = 'Pending OTP'
            expected_message = 'OTP verification is still pending'
            expect_submit = $false
            expect_requires_otp = $true
        }
        @{
            label = 'pending_review'
            email = $PendingReviewEmail
            password = $PendingReviewPassword
            expected_role = 'carrier'
            expected_status = 'Pending Review'
            expected_message = 'waiting for admin review'
            expect_submit = $false
            expect_requires_otp = $false
        }
        @{
            label = 'revision_requested'
            email = $RevisionRequestedEmail
            password = $RevisionRequestedPassword
            expected_role = 'shipper'
            expected_status = 'Revision Requested'
            expected_message = 'requested onboarding revisions'
            expect_submit = $true
            expect_requires_otp = $false
        }
        @{
            label = 'rejected'
            email = $RejectedEmail
            password = $RejectedPassword
            expected_role = 'broker'
            expected_status = 'Rejected'
            expected_message = 'currently rejected'
            expect_submit = $false
            expect_requires_otp = $false
        }
    )

    foreach ($account in $lifecycleAccounts) {
        $login = Login-RustUser -Client $client -Email $account.email -Password $account.password -Label $account.label
        Add-Result -Results $results -Account $account.label -Check 'login' -Passed $true -Detail $login.message
        Add-Result -Results $results -Account $account.label -Check 'role' -Passed ($login.session.user.role_key -eq $account.expected_role) -Detail "role=$($login.session.user.role_key)"
        Add-Result -Results $results -Account $account.label -Check 'status' -Passed ($login.session.user.account_status_label -eq $account.expected_status) -Detail "status=$($login.session.user.account_status_label)"
        Add-Result -Results $results -Account $account.label -Check 'status message' -Passed ($login.message -like "*$($account.expected_message)*") -Detail $login.message

        $onboarding = Expect-RouteStatus -Client $client -BearerToken $login.token -Path '/auth/onboarding-screen' -AllowedStatus @(200)
        $canSubmit = $false
        $requiresOtp = $false
        if ($onboarding.Passed -and $null -ne $onboarding.Json -and $onboarding.Json.status -eq 'ok') {
            $canSubmit = [bool]$onboarding.Json.data.can_submit
            $requiresOtp = [bool]$onboarding.Json.data.requires_otp
        }

        Add-Result -Results $results -Account $account.label -Check 'onboarding screen' -Passed $onboarding.Passed -Detail "HTTP $($onboarding.StatusCode)"
        Add-Result -Results $results -Account $account.label -Check 'onboarding can_submit' -Passed ($canSubmit -eq $account.expect_submit) -Detail "can_submit=$canSubmit"
        Add-Result -Results $results -Account $account.label -Check 'onboarding requires_otp' -Passed ($requiresOtp -eq $account.expect_requires_otp) -Detail "requires_otp=$requiresOtp"
    }

    Write-Step 'Rust role matrix summary'
    $results | Sort-Object account, check | Format-Table -AutoSize

    $failed = @($results | Where-Object { -not $_.passed })
    if ($failed.Count -gt 0) {
        throw "Rust role matrix found $($failed.Count) failed checks."
    }

    Write-Host "`nRust hosted role matrix passed." -ForegroundColor Green
}
finally {
    $client.Dispose()
    $http.Handler.Dispose()
}
