param(
    [string]$AfterExe = ".\target-after10\release\rusty-rival.exe",
    [string]$BeforeExe = ".\target-before\release\rusty-rival.exe",
    [int]$Depth = 18,
    [int]$Threads = 1,
    [int]$Hash = 128
)

$ErrorActionPreference = "Stop"

function Run-Depth {
    param(
        [string]$Exe,
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
        "position startpos"
        "go depth $Depth"
        "quit"
    )

    $out = $cmds | & $Exe
    $pattern = "info score cp .* depth $Depth .* nodes .* nps .* multipv 1"
    $line = $out | Select-String -Pattern $pattern | Select-Object -Last 1
    if (-not $line) {
        throw "No matching info line found for depth $Depth in output from $Exe."
    }

    $nodes = [int64]([regex]::Match($line.Line, "nodes\s+(\d+)").Groups[1].Value)
    $nps = [int64]([regex]::Match($line.Line, "nps\s+(\d+)").Groups[1].Value)

    [pscustomobject]@{
        Exe = $Exe
        Line = $line.Line
        Nodes = $nodes
        Nps = $nps
    }
}

$afterResult = Run-Depth -Exe $AfterExe -Depth $Depth -Threads $Threads -Hash $Hash
$beforeResult = Run-Depth -Exe $BeforeExe -Depth $Depth -Threads $Threads -Hash $Hash

$afterResult
$beforeResult
