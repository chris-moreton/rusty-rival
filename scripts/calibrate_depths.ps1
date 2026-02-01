param(
    [string]$Exe = ".\target-after12\release\rusty-rival.exe",
    [int]$TargetSeconds = 3,
    [int]$Hash = 128,
    [int]$MaxDepth = 30,
    [string]$FensJson = ".\scripts\perft_fens.json",
    [string]$DepthsJson = ".\scripts\perft_depths.json"
)

$ErrorActionPreference = "Stop"

function Run-Depth {
    param(
        [string]$Exe,
        [string]$Fen,
        [int]$Depth,
        [int]$Hash,
        [int]$TimeoutMs
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

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $Exe
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true

    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    [void]$proc.Start()

    foreach ($cmd in $cmds) {
        $proc.StandardInput.WriteLine($cmd)
    }
    $proc.StandardInput.Flush()
    $proc.StandardInput.Close()

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $exited = $proc.WaitForExit($TimeoutMs)
    if (-not $exited) {
        try { $proc.Kill() } catch {}
        $sw.Stop()
        return [pscustomobject]@{
            Fen = $Fen
            Depth = $Depth
            TimeMs = $sw.ElapsedMilliseconds
            Overrun = $true
            Line = ""
        }
    }
    $sw.Stop()
    $out = $proc.StandardOutput.ReadToEnd() -split "`r?`n"

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
        Overrun = $false
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
    $prevRes = $null
    while ($true) {
        Write-Host "Testing depth $depth for FEN: $fen"
        $timeoutMs = [int]([math]::Ceiling($TargetSeconds * 1.5 * 1000))
        $res = Run-Depth -Exe $Exe -Fen $fen -Depth $depth -Hash $Hash -TimeoutMs $timeoutMs
        $seconds = $res.TimeMs / 1000.0

        if ($res.Overrun -or $seconds -ge $TargetSeconds -or $depth -ge $MaxDepth) {
            $chosen = $depth
            $chosenMs = $res.TimeMs
            if (($res.Overrun -or $seconds -gt ($TargetSeconds * 1.5)) -and $prevRes -ne $null) {
                $chosen = $prevRes.Depth
                $chosenMs = $prevRes.TimeMs
            }
            $results += [pscustomobject]@{
                Fen = $fen
                Depth = $chosen
                TimeMs = $chosenMs
            }
            Write-Host "Calibrated $chosen ply in $($chosenMs) ms for FEN: $fen"
            break
        }

        $prevRes = $res
        $depth += 1
    }
}

$results | ConvertTo-Json -Depth 3 | Set-Content -Path $DepthsJson
Write-Host "Wrote depths to $DepthsJson"
