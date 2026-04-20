param(
    [string]$BaseUrl = 'https://portal.stloads.com',
    [Parameter(Mandatory = $true)][string]$ShipperEmail,
    [Parameter(Mandatory = $true)][string]$ShipperPassword,
    [Parameter(Mandatory = $true)][string]$CarrierEmail,
    [Parameter(Mandatory = $true)][string]$CarrierPassword,
    [Parameter(Mandatory = $true)][string]$BrokerEmail,
    [Parameter(Mandatory = $true)][string]$BrokerPassword,
    [Parameter(Mandatory = $true)][string]$FreightForwarderEmail,
    [Parameter(Mandatory = $true)][string]$FreightForwarderPassword,
    [Parameter(Mandatory = $true)][string]$AdminEmail,
    [Parameter(Mandatory = $true)][string]$AdminPassword
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

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

function Invoke-PhpLogin {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$LoginPagePath,
        [Parameter(Mandatory = $true)][string]$PostPath,
        [Parameter(Mandatory = $true)][string]$Email,
        [Parameter(Mandatory = $true)][string]$Password,
        [string]$RoleId = '',
        [Parameter(Mandatory = $true)][string]$ExpectedPath
    )

    Add-Type -AssemblyName System.Net.Http | Out-Null
    $cookieJar = New-Object System.Net.CookieContainer
    $handler = New-Object System.Net.Http.HttpClientHandler
    $handler.CookieContainer = $cookieJar
    $handler.AllowAutoRedirect = $true
    $handler.AutomaticDecompression = [System.Net.DecompressionMethods]::GZip -bor [System.Net.DecompressionMethods]::Deflate
    $client = New-Object System.Net.Http.HttpClient($handler)

    try {
        $client.DefaultRequestHeaders.Add('User-Agent', 'Codex-QA-Login-Check/1.0')

        $loginPageResponse = $client.GetAsync("$BaseUrl$LoginPagePath").GetAwaiter().GetResult()
        $loginPageResponse.EnsureSuccessStatusCode() | Out-Null
        $loginPageHtml = $loginPageResponse.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        $token = Get-CsrfToken -Html $loginPageHtml

        $pairs = New-Object 'System.Collections.Generic.List[System.Collections.Generic.KeyValuePair[string,string]]'
        $pairs.Add([System.Collections.Generic.KeyValuePair[string,string]]::new('_token', $token))
        $pairs.Add([System.Collections.Generic.KeyValuePair[string,string]]::new('email', $Email))
        $pairs.Add([System.Collections.Generic.KeyValuePair[string,string]]::new('password', $Password))

        if (-not [string]::IsNullOrWhiteSpace($RoleId)) {
            $pairs.Add([System.Collections.Generic.KeyValuePair[string,string]]::new('id', $RoleId))
        }

        $formContent = [System.Net.Http.FormUrlEncodedContent]::new($pairs)
        $postResponse = $client.PostAsync("$BaseUrl$PostPath", $formContent).GetAwaiter().GetResult()
        $postHtml = $postResponse.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        $finalPath = $postResponse.RequestMessage.RequestUri.AbsolutePath

        if ($finalPath -ne $ExpectedPath) {
            $title = if ($postHtml -match '<title>(.*?)</title>') { $matches[1] } else { '' }
            $knownMessage = ''
            foreach ($candidate in @('Invalid credentials', 'Login denied: Role mismatch', 'Please verify your email first.', 'requires revision', 'has been rejected', 'An error occurred while processing your request.')) {
                if ($postHtml -like "*$candidate*") {
                    $knownMessage = $candidate
                    break
                }
            }

            return [pscustomobject]@{
                label = $Label
                email = $Email
                success = $false
                final_path = $finalPath
                expected_path = $ExpectedPath
                page_title = $title
                content_length = $postHtml.Length
                note = if ($knownMessage) { $knownMessage } else { "Landed on $finalPath" }
            }
        }

        $dashboardResponse = $client.GetAsync("$BaseUrl$ExpectedPath").GetAwaiter().GetResult()
        $dashboardResponse.EnsureSuccessStatusCode() | Out-Null
        $dashboardHtml = $dashboardResponse.Content.ReadAsStringAsync().GetAwaiter().GetResult()

        [pscustomobject]@{
            label = $Label
            email = $Email
            success = $true
            final_path = $finalPath
            expected_path = $ExpectedPath
            page_title = if ($dashboardHtml -match '<title>(.*?)</title>') { $matches[1] } else { '' }
            content_length = $dashboardHtml.Length
            note = 'login_ok'
        }
    }
    finally {
        $client.Dispose()
        $handler.Dispose()
    }
}

$results = @()

Write-Step 'Verifying shipper login'
$results += Invoke-PhpLogin -Label 'shipper' -LoginPagePath '/normal-login?id=2' -PostPath '/login' -Email $ShipperEmail -Password $ShipperPassword -RoleId '2' -ExpectedPath '/dashboard'

Write-Step 'Verifying carrier login'
$results += Invoke-PhpLogin -Label 'carrier' -LoginPagePath '/normal-login?id=3' -PostPath '/login' -Email $CarrierEmail -Password $CarrierPassword -RoleId '3' -ExpectedPath '/dashboard'

Write-Step 'Verifying broker login'
$results += Invoke-PhpLogin -Label 'broker' -LoginPagePath '/normal-login?id=4' -PostPath '/login' -Email $BrokerEmail -Password $BrokerPassword -RoleId '4' -ExpectedPath '/dashboard'

Write-Step 'Verifying freight forwarder login'
$results += Invoke-PhpLogin -Label 'freight_forwarder' -LoginPagePath '/normal-login?id=5' -PostPath '/login' -Email $FreightForwarderEmail -Password $FreightForwarderPassword -RoleId '5' -ExpectedPath '/dashboard'

Write-Step 'Verifying admin login'
$results += Invoke-PhpLogin -Label 'admin' -LoginPagePath '/admin/login' -PostPath '/admin/login' -Email $AdminEmail -Password $AdminPassword -ExpectedPath '/admin_dashboard'

Write-Step 'PHP login verification summary'
$results | Format-Table -AutoSize
