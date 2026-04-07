param(
    [string[]]$Paths
)

function Convert-SqlText([string]$sql) {
    $sql = $sql -replace 'NOW\(6\)', 'CURRENT_TIMESTAMP'
    $sql = $sql -replace 'UTC_TIMESTAMP\(6\)', 'CURRENT_TIMESTAMP'
    $sql = $sql -replace 'DATE_SUB\(CURRENT_TIMESTAMP, INTERVAL 30 DAY\)', "CURRENT_TIMESTAMP - INTERVAL '30 days'"
    $sql = $sql -replace 'DATE_ADD\(CURRENT_TIMESTAMP, INTERVAL 1 DAY\)', "CURRENT_TIMESTAMP + INTERVAL '1 day'"
    $sql = $sql -replace 'resolved = 0', 'resolved = FALSE'
    $sql = $sql -replace 'resolved = 1', 'resolved = TRUE'
    $sql = $sql -replace 'status <> ''closed''', 'status <> ''closed'''

    $counter = 0
    $builder = New-Object System.Text.StringBuilder
    foreach ($ch in $sql.ToCharArray()) {
        if ($ch -eq '?') {
            $counter++
            [void]$builder.Append('$' + $counter)
        } else {
            [void]$builder.Append($ch)
        }
    }
    return $builder.ToString()
}

function Convert-File([string]$path) {
    $content = Get-Content -LiteralPath $path -Raw

    $patternRaw = 'sqlx::query(?:_as::<_,\s*[^>]+>|_scalar::<_,\s*[^>]+>)?\(\s*r#"(?<sql>.*?)"#'
    $content = [regex]::Replace($content, $patternRaw, {
        param($m)
        $original = $m.Value
        $sql = $m.Groups['sql'].Value
        $converted = Convert-SqlText $sql
        return $original.Replace($sql, $converted)
    }, [System.Text.RegularExpressions.RegexOptions]::Singleline)

    $patternNormal = 'sqlx::query(?:_as::<_,\s*[^>]+>|_scalar::<_,\s*[^>]+>)?\(\s*"(?<sql>(?:[^"\\]|\\.)*)"'
    $content = [regex]::Replace($content, $patternNormal, {
        param($m)
        $original = $m.Value
        $sql = $m.Groups['sql'].Value
        $converted = Convert-SqlText $sql
        return $original.Replace($sql, $converted)
    }, [System.Text.RegularExpressions.RegexOptions]::Singleline)

    Set-Content -LiteralPath $path -Value $content
}

foreach ($path in $Paths) {
    Convert-File $path
}
