param(
    [int]$Iterations = 25,
    [int]$MaxP95Milliseconds = 750,
    [string]$BaseUrl = "http://127.0.0.1:8080"
)

$ErrorActionPreference = "Stop"

if ($Iterations -lt 5) {
    throw "Iterations must be at least 5 for a useful smoke sample."
}

$durations = New-Object System.Collections.Generic.List[double]
for ($i = 0; $i -lt $Iterations; $i++) {
    $elapsed = Measure-Command {
        Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/health/live" | Out-Null
    }
    $durations.Add($elapsed.TotalMilliseconds)
}

$sorted = $durations | Sort-Object
$p95Index = [Math]::Min($sorted.Count - 1, [Math]::Ceiling($sorted.Count * 0.95) - 1)
$p95 = [Math]::Round($sorted[$p95Index], 2)

Write-Host "health/live p95=${p95}ms over $Iterations requests"
if ($p95 -gt $MaxP95Milliseconds) {
    throw "Performance smoke p95 ${p95}ms exceeded ${MaxP95Milliseconds}ms."
}
