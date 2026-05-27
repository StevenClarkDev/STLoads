param(
    [string]$FrontendSrc = "crates/frontend-leptos/src/pages",
    [int]$LargeFileBytes = 50000
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path $FrontendSrc
$pages = Get-ChildItem -Path $root -Filter "*.rs" -Recurse |
    Sort-Object Length -Descending |
    Select-Object @{Name = "Name"; Expression = { $_.FullName.Substring($root.Path.Length + 1) } }, Length, FullName

Write-Host "Frontend page inventory"
Write-Host "Source: $($root.Path)"
Write-Host ""

$pages | ForEach-Object {
    $status = if ($_.Length -gt $LargeFileBytes) { "refactor-needed" } else { "ok" }
    "{0,-32} {1,8} bytes  {2}" -f $_.Name, $_.Length, $status
}

$large = @($pages | Where-Object { $_.Length -gt $LargeFileBytes })
Write-Host ""
Write-Host ("Large page count: {0}" -f $large.Count)

if ($large.Count -gt 0) {
    Write-Host "Largest refactor targets:"
    $large | Select-Object -First 10 | ForEach-Object {
        "- {0} ({1} bytes)" -f $_.Name, $_.Length
    }
}
