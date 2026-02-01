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
    $timedOut = $false
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

        if (-not $timedOut -and $sw.ElapsedMilliseconds -ge $TimeoutMs) {
            $timedOut = $true
            try {
                $proc.StandardInput.WriteLine("stop")
                $proc.StandardInput.Flush()
            } catch {}
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

    $ms = [int64]$sw.ElapsedMilliseconds
    if ($ms -le 0) { $ms = 1 }

    [pscustomobject]@{
        Fen = $Fen
        Depth = $Depth
        TimeMs = $ms
        Overrun = $timedOut
        Line = $nodesLine
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
