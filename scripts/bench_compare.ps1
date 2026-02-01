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

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($true) {
        if (-not $proc.StandardOutput.EndOfStream) {
            $line = $proc.StandardOutput.ReadLine()
        } else {
            Start-Sleep -Milliseconds 10
            $line = $null
        }

        if ($line -and $line.StartsWith("bestmove")) {
            break
        }
    }

    $proc.StandardInput.WriteLine("state")
    $proc.StandardInput.WriteLine("quit")
    $proc.StandardInput.Flush()
    $proc.StandardInput.Close()

    $nodesLine = $null
    while (-not $proc.StandardOutput.EndOfStream) {
        $line = $proc.StandardOutput.ReadLine()
        if ($line -and $line.StartsWith("Nodes")) {
            $nodesLine = $line
            break
        }
    }

    $proc.WaitForExit(5000) | Out-Null
    $sw.Stop()

    if (-not $nodesLine) {
        throw "No Nodes line found for depth $Depth and FEN: $Fen"
    }

    $nodes = [int64]([regex]::Match($nodesLine, "Nodes\s+(\d+)").Groups[1].Value)
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
        Line = $nodesLine
    }
}

if (-not (Test-Path $DepthsJson)) {
    throw "Missing depths file: $DepthsJson. Run scripts/calibrate_depths.ps1 first."
}

$depths = Get-Content $DepthsJson | ConvertFrom-Json

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

Write-Host "After total nodes:  $afterTotalNodes"
Write-Host "Before total nodes: $beforeTotalNodes"
Write-Host "After avg nps:      $afterAvgNps"
Write-Host "Before avg nps:     $beforeAvgNps"
