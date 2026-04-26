# Rusty Rival

A [UCI](https://en.wikipedia.org/wiki/Universal_Chess_Interface) chess engine written in Rust. Please take a look at the [UCI Documentation](https://en.wikipedia.org/wiki/Universal_Chess_Interface) for usage instructions.

Binaries for Windows, Mac OS and Linux are available on the [releases](https://github.com/chris-moreton/rusty-rival/releases) page.

## Play Online

Rusty Rival plays on Lichess as [RustyRival](https://lichess.org/@/RustyRival/perf/bullet).

**Lichess hardware & configuration:**
- Apple M1 Max (10 cores: 8 performance + 2 efficiency), 64 GB RAM
- Threads: 8
- Hash: 512 MB
- Ponder: enabled
- Move Overhead: 0 (handled by lichess-bot at 500ms)

## Features

- NNUE evaluation with embedded network (can fall back to handcrafted evaluation)
- Alpha-beta search with iterative deepening and aspiration windows
- Lazy SMP multi-threaded search (configurable via UCI `Threads` option)
- Ponder support
- Transposition table, killer moves, history heuristic, countermove heuristic
- Null move pruning, late move reductions, late move pruning, reverse futility pruning
- Singular extensions, probcut, multi-cut, SEE pruning
- Quiescence search with static exchange evaluation
- Syzygy endgame tablebase support
- SPSA-tuned evaluation and search parameters

## Building

1. Build the engine:
   
The engine can be about 50% faster if compiled on the machine on which it will be run in order to take advantage of cpu-specific instructions. To compile locally, it requires that [Rust](https://www.rust-lang.org/tools/install) be installed.

**Linux/macOS:**
```bash
RUSTFLAGS="-C target-cpu=native" RUST_MIN_STACK=4097152 cargo build --release
```

**Windows (PowerShell):**
```powershell
$env:RUSTFLAGS="-C target-cpu=native"; $env:RUST_MIN_STACK=4097152; cargo build --release
```

**Windows (cmd.exe):**
```cmd
set RUSTFLAGS=-C target-cpu=native
set RUST_MIN_STACK=4097152
cargo build --release
```
Note: `.cargo/config.toml` sets `-C target-cpu=native` for local builds by default to maximize performance. This makes the binaries non-portable across different CPUs. The release workflow overrides this to keep published binaries portable.
2. Store the executable in engines directory:
```bash
mkdir -p engines/v020-my-feature
cp target/release/rusty-rival engines/v020-my-feature/
git tag v020-my-feature
```

## UCI Options

| Option | Type | Default | Range | Description |
|--------|------|---------|-------|-------------|
| `Hash` | spin | 256 | 1–16384 | Transposition table size in MB. |
| `Clear Hash` | button | — | — | Clears the transposition table. |
| `Threads` | spin | 1 | 1–256 | Number of search threads (Lazy SMP). |
| `UseNNUE` | check | true | — | Use the NNUE neural network evaluation. When disabled, falls back to handcrafted evaluation. |
| `MultiPV` | spin | 1 | 1–20 | Number of principal variations to report. |
| `SyzygyPath` | string | `<empty>` | — | Path to Syzygy endgame tablebase files. |
| `Contempt` | spin | 0 | -1000–1000 | Contempt factor for draw avoidance. Positive values make the engine avoid draws. |
| `EvalNoise` | spin | 0 | 0–100 | Adds random noise to the evaluation for varied play. |
| `Ponder` | check | false | — | Enable pondering (thinking on the opponent's time). |
| `Move Overhead` | spin | 10 | 0–5000 | Time in ms reserved per move for communication overhead. |
| `UCI_ShowWDL` | check | false | — | Show win/draw/loss probabilities in search output. |
