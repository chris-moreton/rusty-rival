# Rusty Rival - Chess Engine

A chess engine written in Rust, using UCI protocol.

## Project Structure

```
rusty-rival/
├── src/                    # Main source code
│   ├── lib.rs             # Library exports
│   ├── main.rs            # UCI entry point
│   ├── uci.rs             # UCI protocol implementation
│   ├── search.rs          # Alpha-beta search with iterative deepening
│   ├── quiesce.rs         # Quiescence search
│   ├── evaluate.rs        # Position evaluation
│   ├── moves.rs           # Move generation
│   ├── make_move.rs       # Make/unmake move logic
│   ├── types.rs           # Core types (Position, Move, etc.)
│   ├── fen.rs             # FEN parsing/generation
│   ├── bitboards.rs       # Bitboard constants and utilities
│   ├── magic_bitboards.rs # Magic bitboard move generation
│   ├── hash.rs            # Zobrist hashing
│   ├── see.rs             # Static Exchange Evaluation
│   ├── move_scores.rs     # Move ordering/scoring
│   └── engine_constants.rs # Tunable engine parameters
├── tests/                  # Integration tests
│   ├── search.rs          # Search functionality tests
│   ├── make_move.rs       # Move making tests
│   ├── fen.rs             # FEN parsing tests
│   └── ...
├── engines/               # Compiled engine versions for comparison
│   ├── v1-baseline/       # Each version contains a rusty-rival binary
│   ├── v2-weak-queen/
│   └── ...
├── scripts/               # Python helper scripts
│   ├── benchmark.py       # Performance benchmarking
│   └── compete.py         # Engine vs engine matches
├── examples/              # Test programs
└── results/               # Competition results (PGN files)
```

## Building

```bash
# Standard release build
cargo build --release

# Optimized for native CPU (recommended)
# The following env vars should already be in the environment, don't add them to the command
# RUSTFLAGS="-C target-cpu=native"
# RUST_MIN_STACK=4097152
cargo build --release
```

## Testing

```bash
# Run all tests (release mode recommended for search tests)
RUST_MIN_STACK=4097152 cargo test --release

# Run specific test file
cargo test --release --test search

# Run single test
cargo test --release --test search it_finds_a_mate_in_3

# Single-threaded (avoids parallel test issues)
cargo test --release -- --test-threads=1
```

## Scripts

### benchmark.py

Measures search performance (nodes per second) across a set of test positions.

```bash
# Benchmark a single engine at depth 10
./scripts/benchmark.py v6-arrayvec --depth 10

# Compare multiple engines
./scripts/benchmark.py v5-retain-opt v6-arrayvec --depth 12
```

Output shows NPS (nodes per second) for each position and overall comparison.

### compete.py

Runs engine vs engine matches using python-chess and cutechess-cli.

```bash
# Match between two engines (100 games, 1s/move, uses opening book)
./scripts/compete.py v5-retain-opt v6-arrayvec --games 100 --time 1.0

# Quick test match
./scripts/compete.py v5-retain-opt v6-arrayvec --games 10 --time 0.5

# Disable opening book
./scripts/compete.py v5-retain-opt v6-arrayvec --games 10 --no-book

# Round-robin league (3+ engines)
./scripts/compete.py v4 v5 v6 --league --games 10 --time 0.5
```

Features:
- 50 opening positions (played twice, once per side)
- Elo difference calculation with error margins
- PGN output saved to `results/competitions/`

## Engine Versions

Engines are stored in `engines/<version>/rusty-rival`. Create new versions:

```bash
# Build and save new version - note three digit version number so alphabetical sorting works
RUSTFLAGS="-C target-cpu=native" cargo build --release
mkdir -p engines/v007-new-feature
cp target/release/rusty-rival engines/v007-new-feature/

# IMPORTANT: Always tag the commit when creating a new version
git tag v007-new-feature
```

**Always tag commits when creating new engine versions** - this allows us to recreate any version later.

## Key Source Files

### search.rs
- `iterative_deepening()` - Main search entry point
- `start_search()` - Root move search loop
- `search()` - Alpha-beta search with null move pruning, LMR, etc.
- Key features: aspiration windows, transposition table, killer moves, history heuristic

### quiesce.rs
- `quiesce()` - Quiescence search for tactical stability
- Only searches captures and promotions
- Uses SEE (Static Exchange Evaluation) for move pruning

### make_move.rs
- `make_move()` - Copy-based move making (original approach)
- `make_move_in_place()` - In-place move making with UnmakeInfo
- `unmake_move()` - Reverses an in-place move

### types.rs
- `Position` - Full board state (~156 bytes)
- `Pieces` - Per-side piece bitboards
- `SearchState` - Search state (hash table, killers, history)
- `UnmakeInfo` - State needed to unmake a move

## Performance Optimization History

1. **v5-retain-opt**: Used `retain()` for move filtering instead of `filter().collect()` - 4.5% speedup
2. **v6-arrayvec**: Used ArrayVec for PV paths instead of Vec - 3.9% speedup

## Common Issues

### Stack Overflow in Tests (IMPORTANT)

**The search tests WILL stack-overflow in debug mode** because:
1. Debug builds have much larger stack frames (no inlining, extra debug info)
2. Chess search recurses deeply (depth 9+ with many moves per ply)
3. Each recursion level uses significant stack space

**This is EXPECTED behavior, not a bug in the code.**

To run search tests successfully:
```bash
# Option 1: Use large stack (16MB) - recommended for debug mode
RUST_MIN_STACK=16777216 cargo test --test search

# Option 2: Use release mode (smaller stack frames)
cargo test --release --test search

# Option 3: Run quick perft test to verify correctness without deep search
echo "position startpos\ngo perft 5" | ./target/release/rusty-rival
```

**When you see stack overflow in tests, don't investigate - just increase stack or use release mode.**

### Slow Test Compilation in Release Mode
Running `cargo test --release` triggers full LTO compilation for ALL test files, which can take 10+ minutes. Instead:

```bash
# Quick debug test with large stack - use for most testing
RUST_MIN_STACK=16777216 cargo test --test search

# Only use release mode when actually needed for performance-sensitive tests
cargo test --release --test search
```

**Best Practice:** Verify correctness with perft (which uses less stack), then run benchmarks in release mode.

### Position Corruption
If the search returns wrong moves or crashes, check:
1. All `make_move_in_place()` calls have matching `unmake_move()` calls
2. `check_time!` macros only appear AFTER unmake operations
3. Early returns don't skip unmake operations

## Search Architecture

```
iterative_deepening()
  └── start_search()           # Loops through root moves
        └── search()           # Main alpha-beta
              ├── quiesce()    # At depth 0
              ├── null move    # Pruning
              ├── hash move    # From transposition table
              └── search()     # Recursive for each move
```

The position is passed by mutable reference. The current implementation uses copy-based move making where a new Position is created for each move in the tree.

## Git Commit Guidelines

- **Do NOT add Claude/AI attributions** to commit messages (no "Generated with Claude Code", no "Co-Authored-By: Claude")
- **Always tag commits when creating new engine versions** - a "new engine version" means copying the compiled binary into `engines/v*-name/` folder for benchmarking/comparison (e.g., `git tag v9-unmake-opt`)
- Keep commit messages concise and focused on what changed

## GitHub Issues

Performance-related issues are tracked on GitHub. Check open issues for optimization opportunities.
