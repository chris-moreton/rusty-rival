param(
    [string]$Exe = ".\target-after12\release\rusty-rival.exe",
    [int]$TargetSeconds = 3,
    [int]$Hash = 128,
    [int]$MaxTries = 4,
    [string]$FensJson = ".\scripts\perft_fens.json",
    [string]$DepthsJson = ".\scripts\perft_depths.json"
)

$ErrorActionPreference = "Stop"

function Run-Depth {
    param(
        [string]$Exe,
        [string]$Fen,
        [int]$Depth,
        [int]$Hash
    )

    $cmds = @(
        "uci"
        "isready"
        "setoption name Hash value $Hash"
        "setoption name Clear Hash"
        "position fen $Fen"
        "go depth $Depth"
        "state"
        "quit"
    )

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $out = $cmds | & $Exe
    $sw.Stop()

    $nodesLine = $out | Select-String -Pattern "^Nodes\s+(\d+)" | Select-Object -Last 1
    if (-not $nodesLine) {
        $tail = $out | Select-Object -Last 20
        $tail | ForEach-Object { Write-Host $_ }
        throw "No Nodes line found for depth $Depth and FEN: $Fen"
    }

    $ms = [int64]$sw.ElapsedMilliseconds
    if ($ms -le 0) { $ms = 1 }

    [pscustomobject]@{
        Fen = $Fen
        Depth = $Depth
        TimeMs = $ms
        Line = $nodesLine.Line
    }
}

if (-not (Test-Path $FensJson)) {
    throw "Missing FEN list: $FensJson. Run scripts/extract_perft_fens.ps1 first."
}

$fens = Get-Content $FensJson | ConvertFrom-Json

$results = @()
foreach ($fen in $fens) {
    $depth = 4
    $tries = 0
    while ($true) {
        $tries += 1
        $res = Run-Depth -Exe $Exe -Fen $fen -Depth $depth -Hash $Hash
        $seconds = $res.TimeMs / 1000.0
        if ($seconds -ge $TargetSeconds -or $depth -ge 30 -or $tries -ge $MaxTries) {
            $results += [pscustomobject]@{
                Fen = $fen
                Depth = $depth
                TimeMs = $res.TimeMs
            }
            Write-Host "Calibrated $depth ply in $($res.TimeMs) ms for FEN: $fen"
            break
        }
        if ($seconds -gt 0.0) {
            $ratio = $TargetSeconds / $seconds
            $delta = [int]([math]::Max(2, [math]::Ceiling([math]::Log($ratio, 2))))
            $depth += $delta
        } else {
            $depth += 2
        }
    }
}

$results | ConvertTo-Json -Depth 3 | Set-Content -Path $DepthsJson
Write-Host "Wrote depths to $DepthsJson"
