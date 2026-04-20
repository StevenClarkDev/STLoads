param(
    [string]$BaseUrl = 'https://portal.stloads.com',
    [Parameter(Mandatory = $true)][string]$PendingOtpEmail,
    [Parameter(Mandatory = $true)][string]$PendingOtpPassword,
    [Parameter(Mandatory = $true)][string]$PendingReviewEmail,
    [Parameter(Mandatory = $true)][string]$PendingReviewPassword,
    [Parameter(Mandatory = $true)][string]$RevisionRequestedEmail,
    [Parameter(Mandatory = $true)][string]$RevisionRequestedPassword,
    [Parameter(Mandatory = $true)][string]$RejectedEmail,
    [Parameter(Mandatory = $true)][string]$RejectedPassword
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Net.Http | Out-Null

$BaseUrl = $BaseUrl.TrimEnd('/')

function Write-Step {
    param([string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Get-CsrfToken {
    param([Parameter(Mandatory = $true)][string]$Html)

    $match = [regex]::Match($Html, 'name="_token"\s+value="([^"]+)"')
    if (-not $match.Success) {
        throw 'Unable to locate Laravel CSRF token in the login form.'
    }

    return $match.Groups[1].Value
}

function New-PhpClient {
    $cookieJar = New-Object System.Net.CookieContainer
    $handler = New-Object System.Net.Http.HttpClientHandler
    $handler.CookieContainer = $cookieJar
    $handler.AllowAutoRedirect = $true
    $handler.AutomaticDecompression = [System.Net.DecompressionMethods]::GZip -bor [System.Net.DecompressionMethods]::Deflate
    $client = New-Object System.Net.Http.HttpClient($handler)
    $client.DefaultRequestHeaders.Add('User-Agent', 'Codex-PHP-Lifecycle-Check/1.0')

    return @{
        Client = $client
        Handler = $handler
    }
}

function Invoke-PhpLifecycleLogin {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$LoginPagePath,
        [Parameter(Mandatory = $true)][string]$PostPath,
        [Parameter(Mandatory = $true)][string]$Email,
        [Parameter(Mandatory = $true)][string]$Password,
        [Parameter(Mandatory = $true)][string]$RoleId,
        [Parameter(Mandatory = $true)][string[]]$AllowedPaths,
        [Parameter(Mandatory = $true)][string[]]$RequiredMarkers
    )

    $http = New-PhpClient
    $client = $http.Client
    $handler = $http.Handler

    try {
        $loginPageResponse = $client.GetAsync("$BaseUrl$LoginPagePath").GetAwaiter().GetResult()
        $loginPageResponse.EnsureSuccessStatusCode() | Out-Null
        $loginPageHtml = $loginPageResponse.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        $token = Get-CsrfToken -Html $loginPageHtml

        $pairs = New-Object 'System.Collections.Generic.List[System.Collections.Generic.KeyValuePair[string,string]]'
        $pairs.Add([System.Collections.Generic.KeyValuePair[string,string]]::new('_token', $token))
        $pairs.Add([System.Collections.Generic.KeyValuePair[string,string]]::new('email', $Email))
        $pairs.Add([System.Collections.Generic.KeyValuePair[string,string]]::new('password', $Password))
        $pairs.Add([System.Collections.Generic.KeyValuePair[string,string]]::new('id', $RoleId))

        $formContent = [System.Net.Http.FormUrlEncodedContent]::new($pairs)
        $postResponse = $client.PostAsync("$BaseUrl$PostPath", $formContent).GetAwaiter().GetResult()
        $postHtml = $postResponse.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        $finalUri = $postResponse.RequestMessage.RequestUri
        $finalPath = $finalUri.AbsolutePath
        $finalUrl = $finalUri.AbsoluteUri
        $matchedMarkers = @($RequiredMarkers | Where-Object { $postHtml -like "*$_*" })
        $pathMatched = $AllowedPaths -contains $finalPath
        $markerMatched = $matchedMarkers.Count -gt 0
        $success = $pathMatched -and $markerMatched

        [pscustomobject]@{
            label = $Label
            email = $Email
            success = $success
            final_path = $finalPath
            final_url = $finalUrl
            allowed_paths = ($AllowedPaths -join ', ')
            matched_markers = ($matchedMarkers -join ' | ')
            required_markers = ($RequiredMarkers -join ' | ')
            note = if ($success) {
                'state_ok'
            } elseif (-not $pathMatched -and $finalPath -eq '/dashboard') {
                'active_dashboard_login'
            } elseif (-not $pathMatched -and $finalPath -eq '/admin_dashboard') {
                'active_admin_login'
            } elseif (-not $pathMatched) {
                "unexpected_path:$finalPath"
            } elseif (-not $markerMatched) {
                'expected_state_copy_missing'
            } else {
                'unknown_state_mismatch'
            }
            page_title = if ($postHtml -match '<title>(.*?)</title>') { $matches[1] } else { '' }
            content_length = $postHtml.Length
        }
    }
    finally {
        $client.Dispose()
        $handler.Dispose()
    }
}

$results = @()

Write-Step 'Verifying pending OTP account behavior'
$results += Invoke-PhpLifecycleLogin `
    -Label 'pending_otp' `
    -LoginPagePath '/normal-login?id=2' `
    -PostPath '/login' `
    -Email $PendingOtpEmail `
    -Password $PendingOtpPassword `
    -RoleId '2' `
    -AllowedPaths @('/otp') `
    -RequiredMarkers @('Please verify your email first.', 'Verify OTP', 'Enter OTP')

Write-Step 'Verifying pending review account behavior'
$results += Invoke-PhpLifecycleLogin `
    -Label 'pending_review' `
    -LoginPagePath '/normal-login?id=3' `
    -PostPath '/login' `
    -Email $PendingReviewEmail `
    -Password $PendingReviewPassword `
    -RoleId '3' `
    -AllowedPaths @('/login') `
    -RequiredMarkers @('KYC Pending', 'pending admin approval', 'Enable Payouts')

Write-Step 'Verifying revision requested account behavior'
$results += Invoke-PhpLifecycleLogin `
    -Label 'revision_requested' `
    -LoginPagePath '/normal-login?id=4' `
    -PostPath '/login' `
    -Email $RevisionRequestedEmail `
    -Password $RevisionRequestedPassword `
    -RoleId '4' `
    -AllowedPaths @('/login') `
    -RequiredMarkers @('Revision', 'requires revision', 'Admin Remarks')

Write-Step 'Verifying rejected account behavior'
$results += Invoke-PhpLifecycleLogin `
    -Label 'rejected' `
    -LoginPagePath '/normal-login?id=5' `
    -PostPath '/login' `
    -Email $RejectedEmail `
    -Password $RejectedPassword `
    -RoleId '5' `
    -AllowedPaths @('/login') `
    -RequiredMarkers @('Rejected', 'has been rejected', 'contact support')

Write-Step 'PHP lifecycle-state verification summary'
$results | Format-Table -AutoSize

$failed = @($results | Where-Object { -not $_.success })
if ($failed.Count -gt 0) {
    throw "PHP lifecycle-state verification found $($failed.Count) failed checks."
}
