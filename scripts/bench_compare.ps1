param(
    [string]$AfterExe = ".\target-after12\release\rusty-rival.exe",
    [string]$BeforeExe = ".\target-before\release\rusty-rival.exe",
    [int]$Hash = 128,
    [string]$DepthsJson = ".\scripts\perft_depths.json",
    [string]$CsvOut = ".\scripts\perft_compare.csv"
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
        "ucinewgame"
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

    $nodes = [int64]([regex]::Match($nodesLine.Line, "Nodes\s+(\d+)").Groups[1].Value)
    $ms = [int64]$sw.ElapsedMilliseconds
    if ($ms -le 0) { $ms = 1 }
    $nps = [int64]([math]::Round($nodes / ($ms / 1000.0)))

    [pscustomobject]@{
        Exe = $Exe
        Fen = $Fen
        Depth = $Depth
        Nodes = $nodes
        Nps = $nps
        TimeMs = $ms
        Line = $nodesLine.Line
    }
}

if (-not (Test-Path $DepthsJson)) {
    throw "Missing depths file: $DepthsJson. Run scripts/calibrate_depths.ps1 first."
}

$depths = Get-Content $DepthsJson | ConvertFrom-Json

$rows = @()
foreach ($entry in $depths) {
    $fen = $entry.Fen
    $depth = [int]$entry.Depth

    $after = Run-Depth -Exe $AfterExe -Fen $fen -Depth $depth -Hash $Hash
    $before = Run-Depth -Exe $BeforeExe -Fen $fen -Depth $depth -Hash $Hash

    $rows += [pscustomobject]@{
        Fen = $fen
        Depth = $depth
        AfterNodes = $after.Nodes
        AfterNps = $after.Nps
        AfterTimeMs = $after.TimeMs
        BeforeNodes = $before.Nodes
        BeforeNps = $before.Nps
        BeforeTimeMs = $before.TimeMs
    }
}

$rows | Export-Csv -NoTypeInformation -Path $CsvOut
Write-Host "Wrote results to $CsvOut"

$afterTotalNodes = ($rows | Measure-Object -Property AfterNodes -Sum).Sum
$beforeTotalNodes = ($rows | Measure-Object -Property BeforeNodes -Sum).Sum
$afterAvgNps = [int64](($rows | Measure-Object -Property AfterNps -Average).Average)
$beforeAvgNps = [int64](($rows | Measure-Object -Property BeforeNps -Average).Average)

Write-Host "After total nodes:  $afterTotalNodes"
Write-Host "Before total nodes: $beforeTotalNodes"
Write-Host "After avg nps:      $afterAvgNps"
Write-Host "Before avg nps:     $beforeAvgNps"
