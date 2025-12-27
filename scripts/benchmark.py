#!/usr/bin/env python3
"""
Engine benchmark script for measuring search performance.

Usage:
    # Benchmark a single engine
    ./scripts/benchmark.py v4-performance --depth 12

    # Compare multiple engines
    ./scripts/benchmark.py v3-improved v4-performance --depth 12

    # Quick test with lower depth
    ./scripts/benchmark.py v4-performance --depth 8
"""

import argparse
import subprocess
import sys
import time
from pathlib import Path


# Benchmark positions - diverse set for comprehensive testing
BENCHMARK_POSITIONS = [
    # Middlegame positions
    ("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
     "Italian Game opening"),
    ("r1bq1rk1/ppp2ppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQ1RK1 w - - 0 8",
     "Giuoco Piano middlegame"),
    ("r2qkb1r/ppp2ppp/2n1bn2/3pp3/4P3/1NN1BP2/PPPP2PP/R2QKB1R w KQkq - 2 6",
     "Complex middlegame"),

    # Tactical positions
    ("r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4",
     "Scholar's mate threat"),
    ("r1b1k2r/ppppqppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 6 6",
     "Tactical middlegame"),
    ("r2q1rk1/ppp2ppp/2n1bn2/3p4/3P4/2NBPN2/PPP2PPP/R2Q1RK1 w - - 0 10",
     "Central tension"),

    # Quiet positions (good for measuring pruning efficiency)
    ("r1bq1rk1/pp2bppp/2n1pn2/2pp4/3P4/2NBPN2/PPP2PPP/R1BQ1RK1 w - - 0 9",
     "Quiet French structure"),
    ("r2q1rk1/pp2ppbp/2np1np1/8/3PP3/2N2N2/PP2BPPP/R1BQ1RK1 w - - 0 10",
     "King's Indian quiet"),

    # Endgame positions
    ("8/pp3kpp/2p2p2/3p4/3P4/2P2P2/PP3KPP/8 w - - 0 1",
     "Pawn endgame"),
    ("8/5pk1/6p1/8/5P2/6P1/5K2/8 w - - 0 1",
     "King + pawn endgame"),
    ("r4rk1/ppp2ppp/8/8/8/8/PPP2PPP/R4RK1 w - - 0 1",
     "Rook endgame"),
    ("3r2k1/ppp2ppp/8/8/8/8/PPP2PPP/3R2K1 w - - 0 1",
     "Single rook endgame"),
]


def get_engine_path(name: str, engine_dir: Path) -> Path:
    path = engine_dir / name / "rusty-rival"
    if not path.exists():
        print(f"Error: Engine not found: {path}")
        sys.exit(1)
    return path


def benchmark_position(engine_path: Path, fen: str, depth: int) -> dict:
    """Benchmark a single position and return results."""
    proc = subprocess.Popen(
        [str(engine_path)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )

    # Send UCI initialization
    proc.stdin.write("uci\n")
    proc.stdin.flush()

    # Wait for uciok
    while True:
        line = proc.stdout.readline()
        if "uciok" in line:
            break

    # Set position
    proc.stdin.write(f"position fen {fen}\n")
    proc.stdin.flush()

    # Start search with depth limit
    start_time = time.perf_counter()
    proc.stdin.write(f"go depth {depth}\n")
    proc.stdin.flush()

    nodes = 0
    nps = 0
    depth_reached = 0

    # Read output until bestmove
    while True:
        line = proc.stdout.readline()
        if line.startswith("info"):
            parts = line.split()
            for i, part in enumerate(parts):
                if part == "nodes" and i + 1 < len(parts):
                    nodes = int(parts[i + 1])
                if part == "nps" and i + 1 < len(parts):
                    nps = int(parts[i + 1])
                if part == "depth" and i + 1 < len(parts):
                    depth_reached = int(parts[i + 1])
        if line.startswith("bestmove"):
            break

    elapsed = time.perf_counter() - start_time

    # Clean up
    proc.stdin.write("quit\n")
    proc.stdin.flush()
    proc.wait()

    # Calculate NPS if not reported
    if nps == 0 and elapsed > 0:
        nps = int(nodes / elapsed)

    return {
        "nodes": nodes,
        "nps": nps,
        "time": elapsed,
        "depth": depth_reached
    }


def run_benchmark(engine_name: str, engine_dir: Path, depth: int) -> dict:
    """Run full benchmark suite on an engine."""
    engine_path = get_engine_path(engine_name, engine_dir)

    print(f"\nBenchmarking {engine_name} at depth {depth}")
    print("=" * 70)

    total_nodes = 0
    total_time = 0
    results = []

    for i, (fen, name) in enumerate(BENCHMARK_POSITIONS, 1):
        result = benchmark_position(engine_path, fen, depth)
        results.append(result)

        total_nodes += result["nodes"]
        total_time += result["time"]

        nodes_k = result["nodes"] / 1000
        nps_k = result["nps"] / 1000

        print(f"  {i:2d}. {name:30s} {nodes_k:8.1f}k nodes  {result['time']:5.2f}s  {nps_k:6.0f}k NPS")

    avg_nps = total_nodes / total_time if total_time > 0 else 0

    print("-" * 70)
    print(f"  Total: {total_nodes/1000000:.2f}M nodes in {total_time:.2f}s")
    print(f"  Average: {avg_nps/1000:.0f}k NPS")
    print("=" * 70)

    return {
        "engine": engine_name,
        "total_nodes": total_nodes,
        "total_time": total_time,
        "avg_nps": avg_nps,
        "results": results
    }


def compare_engines(benchmarks: list[dict]):
    """Compare benchmark results between engines."""
    if len(benchmarks) < 2:
        return

    print(f"\n{'='*70}")
    print("COMPARISON")
    print("=" * 70)

    # Sort by NPS
    benchmarks.sort(key=lambda x: x["avg_nps"], reverse=True)
    baseline = benchmarks[-1]["avg_nps"]  # Slowest as baseline

    for i, b in enumerate(benchmarks, 1):
        speedup = ((b["avg_nps"] / baseline) - 1) * 100 if baseline > 0 else 0
        speedup_str = f"+{speedup:.1f}%" if speedup > 0 else f"{speedup:.1f}%"

        print(f"  {i}. {b['engine']:20s} {b['avg_nps']/1000:6.0f}k NPS  ({speedup_str})")

    print("=" * 70)


def main():
    parser = argparse.ArgumentParser(description="Chess engine benchmark")
    parser.add_argument("engines", nargs="+", help="Engine version names to benchmark")
    parser.add_argument("--depth", "-d", type=int, default=10,
                        help="Search depth (default: 10)")

    args = parser.parse_args()

    script_dir = Path(__file__).parent
    engine_dir = script_dir.parent / "engines"

    benchmarks = []
    for engine_name in args.engines:
        result = run_benchmark(engine_name, engine_dir, args.depth)
        benchmarks.append(result)

    if len(benchmarks) > 1:
        compare_engines(benchmarks)


if __name__ == "__main__":
    main()
