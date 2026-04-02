# Rusty Rival

A [UCI](https://en.wikipedia.org/wiki/Universal_Chess_Interface) chess engine written in Rust. Please take a look at the [UCI Documentation](https://en.wikipedia.org/wiki/Universal_Chess_Interface) for usage instructions.

Binaries for Windows, Mac OS and Linux are available on the [releases](https://github.com/chris-moreton/rusty-rival/releases) page.

## Features

- Alpha-beta search with iterative deepening and aspiration windows
- Lazy SMP multi-threaded search (configurable via UCI `Threads` option)
- Transposition table, killer moves, history heuristic, countermove heuristic
- Null move pruning, late move reductions, late move pruning, reverse futility pruning
- Singular extensions, probcut, multi-cut, SEE pruning
- Quiescence search with static exchange evaluation
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
