param(
    [string]$InputFile = ".\tests\perft.rs",
    [string]$OutputFile = ".\scripts\perft_fens.json"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $InputFile)) {
    throw "Missing input file: $InputFile"
}

$text = Get-Content $InputFile -Raw
$matches = [regex]::Matches($text, 'get_position\("([^"]+)"\)')
$fens = New-Object System.Collections.Generic.List[string]
foreach ($m in $matches) {
    $fens.Add($m.Groups[1].Value) | Out-Null
}

$unique = $fens | Select-Object -Unique
$unique | ConvertTo-Json -Depth 2 | Set-Content -Path $OutputFile
Write-Host "Wrote $($unique.Count) unique FENs to $OutputFile"
