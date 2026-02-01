param(
    [string]$AfterExe = ".\target-after12\release\rusty-rival.exe",
    [string]$BeforeExe = ".\target-before\release\rusty-rival.exe",
    [int]$Hash = 128,
    [string]$DepthsJson = ".\scripts\perft_depths.json",
    [string]$CsvOut = ".\scripts\perft_compare.csv",
    [int]$MaxPositions = 0
)

$ErrorActionPreference = "Stop"

function Run-Depth {
    param(
        [string]$Exe,
        [string]$Fen,
        [int]$Depth,
        [int]$Hash
    )

    $exePath = (Resolve-Path $Exe).Path
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exePath
    $psi.WorkingDirectory = Split-Path $exePath -Parent
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true

    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    [void]$proc.Start()

    $proc.StandardInput.WriteLine("uci")
    $proc.StandardInput.WriteLine("isready")
    $proc.StandardInput.WriteLine("setoption name Hash value $Hash")
    $proc.StandardInput.WriteLine("setoption name Clear Hash")
    $proc.StandardInput.WriteLine("position fen $Fen")
    $proc.StandardInput.WriteLine("go depth $Depth")
    $proc.StandardInput.Flush()

    $lastInfoLine = $null
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($true) {
        if (-not $proc.StandardOutput.EndOfStream) {
            $line = $proc.StandardOutput.ReadLine()
        } else {
            Start-Sleep -Milliseconds 10
            $line = $null
        }

        # Capture info lines that contain node counts
        if ($line -and $line.StartsWith("info") -and $line -match "nodes") {
            $lastInfoLine = $line
        }

        if ($line -and $line.StartsWith("bestmove")) {
            break
        }
    }
    $sw.Stop()

    $proc.StandardInput.WriteLine("quit")
    $proc.StandardInput.Flush()
    $proc.StandardInput.Close()
    $proc.WaitForExit(5000) | Out-Null

    if (-not $lastInfoLine) {
        throw "No info line with nodes found for depth $Depth and FEN: $Fen"
    }

    # Parse nodes and nps from info line: "info depth 17 nodes 1234567 nps 2000000 ..."
    $nodesMatch = [regex]::Match($lastInfoLine, "nodes\s+(\d+)")
    $npsMatch = [regex]::Match($lastInfoLine, "nps\s+(\d+)")
    $timeMatch = [regex]::Match($lastInfoLine, "time\s+(\d+)")

    if (-not $nodesMatch.Success) {
        throw "Could not parse nodes from info line: $lastInfoLine"
    }

    $nodes = [int64]$nodesMatch.Groups[1].Value
    $nps = if ($npsMatch.Success) { [int64]$npsMatch.Groups[1].Value } else { 0 }
    $ms = if ($timeMatch.Success) { [int64]$timeMatch.Groups[1].Value } else { [int64]$sw.ElapsedMilliseconds }
    if ($ms -le 0) { $ms = 1 }
    if ($nps -eq 0) { $nps = [int64]([math]::Round($nodes / ($ms / 1000.0))) }

    [pscustomobject]@{
        Exe = $Exe
        Fen = $Fen
        Depth = $Depth
        Nodes = $nodes
        Nps = $nps
        TimeMs = $ms
        Line = $lastInfoLine
    }
}

if (-not (Test-Path $DepthsJson)) {
    throw "Missing depths file: $DepthsJson. Run scripts/calibrate_depths.ps1 first."
}

$depths = Get-Content $DepthsJson | ConvertFrom-Json

# Limit positions if MaxPositions is specified
if ($MaxPositions -gt 0 -and $MaxPositions -lt $depths.Count) {
    $depths = $depths | Select-Object -First $MaxPositions
}

$rows = @()
$total = $depths.Count
$idx = 0
$afterTotalNodes = 0
$beforeTotalNodes = 0
$afterTotalNps = 0
$beforeTotalNps = 0
foreach ($entry in $depths) {
    $idx += 1
    $fen = $entry.Fen
    $depth = [int]$entry.Depth

    $after = Run-Depth -Exe $AfterExe -Fen $fen -Depth $depth -Hash $Hash
    $before = Run-Depth -Exe $BeforeExe -Fen $fen -Depth $depth -Hash $Hash

    $afterTotalNodes += $after.Nodes
    $beforeTotalNodes += $before.Nodes
    $afterTotalNps += $after.Nps
    $beforeTotalNps += $before.Nps
    $afterAvgNps = [int64]($afterTotalNps / $idx)
    $beforeAvgNps = [int64]($beforeTotalNps / $idx)

    Write-Host "[$idx/$total] depth $depth"
    Write-Host "  after  nodes=$($after.Nodes) nps=$($after.Nps) time=${($after.TimeMs)}ms"
    Write-Host "  before nodes=$($before.Nodes) nps=$($before.Nps) time=${($before.TimeMs)}ms"
    Write-Host "  totals after nodes=$afterTotalNodes avg_nps=$afterAvgNps | before nodes=$beforeTotalNodes avg_nps=$beforeAvgNps"

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

Write-Host ""
Write-Host "=== Final Summary ==="
Write-Host "After total nodes:  $afterTotalNodes"
Write-Host "Before total nodes: $beforeTotalNodes"
Write-Host "After avg nps:      $afterAvgNps"
Write-Host "Before avg nps:     $beforeAvgNps"

if ($beforeAvgNps -gt 0) {
    $speedChange = (($afterAvgNps - $beforeAvgNps) / $beforeAvgNps) * 100
    $sign = if ($speedChange -ge 0) { "+" } else { "" }
    Write-Host ("Speed change:       {0}{1:F2}%" -f $sign, $speedChange)
    if ($speedChange -gt 0) {
        Write-Host "Result: FASTER" -ForegroundColor Green
    } elseif ($speedChange -lt 0) {
        Write-Host "Result: SLOWER" -ForegroundColor Red
    } else {
        Write-Host "Result: NO CHANGE" -ForegroundColor Yellow
    }
}
