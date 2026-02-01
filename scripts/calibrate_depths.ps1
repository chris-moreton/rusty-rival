param(
    [string]$Exe = ".\target-after12\release\rusty-rival.exe",
    [int]$TargetSeconds = 3,
    [int]$Threads = 1,
    [int]$Hash = 128,
    [string]$FensJson = ".\scripts\perft_fens.json",
    [string]$DepthsJson = ".\scripts\perft_depths.json"
)

$ErrorActionPreference = "Stop"

function Run-Depth {
    param(
        [string]$Exe,
        [string]$Fen,
        [int]$Depth,
        [int]$Threads,
        [int]$Hash
    )

    $cmds = @(
        "uci"
        "isready"
        "setoption name Threads value $Threads"
        "setoption name Hash value $Hash"
        "ucinewgame"
        "position fen $Fen"
        "go depth $Depth"
        "quit"
    )

    $out = $cmds | & $Exe
    $pattern = "info score cp .* depth $Depth .* nodes .* nps .* multipv 1"
    $line = $out | Select-String -Pattern $pattern | Select-Object -Last 1
    if (-not $line) {
        throw "No matching info line found for depth $Depth and FEN: $Fen"
    }

    $ms = [int64]([regex]::Match($line.Line, "time\s+(\d+)").Groups[1].Value)
    if ($ms -le 0) {
        $ms = 1
    }

    [pscustomobject]@{
        Fen = $Fen
        Depth = $Depth
        TimeMs = $ms
        Line = $line.Line
    }
}

if (-not (Test-Path $FensJson)) {
    throw "Missing FEN list: $FensJson. Run scripts/extract_perft_fens.ps1 first."
}

$fens = Get-Content $FensJson | ConvertFrom-Json

$results = @()
foreach ($fen in $fens) {
    $depth = 6
    while ($true) {
        $res = Run-Depth -Exe $Exe -Fen $fen -Depth $depth -Threads $Threads -Hash $Hash
        $seconds = $res.TimeMs / 1000.0
        if ($seconds -ge $TargetSeconds -or $depth -ge 30) {
            $results += [pscustomobject]@{
                Fen = $fen
                Depth = $depth
                TimeMs = $res.TimeMs
            }
            break
        }
        $depth += 1
    }
}

$results | ConvertTo-Json -Depth 3 | Set-Content -Path $DepthsJson
Write-Host "Wrote depths to $DepthsJson"
