# Rusty Rival

A [UCI](https://en.wikipedia.org/wiki/Universal_Chess_Interface) chess engine written in Rust. Please take a look at the [UCI Documentation](https://en.wikipedia.org/wiki/Universal_Chess_Interface) for usage instructions.

Binaries for Windows, Mac OS and Linux are available on the [releases](https://github.com/chris-moreton/rusty-rival/releases) page.

## Building and Storing Engine Versions for Local Strength Testing

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
2. Store the executable in engines directory:
```bash
mkdir -p engines/v020-my-feature
cp target/release/rusty-rival engines/v020-my-feature/
git tag v020-my-feature
```

## Engine Configuration

Engines are configured via `engines/engines.json`. This allows you to:
- Add third-party UCI engines (Stockfish, Maia, etc.)
- Configure UCI options per engine (e.g., limit Stockfish's Elo)
- Run the same binary at different strength levels

### Configuration File Format

```json
{
  "v019-capture-fix": {
    "binary": "v019-capture-fix/rusty-rival"
  },
  "sf-1500": {
    "binary": "/opt/homebrew/bin/stockfish",
    "uci_options": {
      "UCI_LimitStrength": "true",
      "UCI_Elo": "1500"
    }
  },
  "sf-2000": {
    "binary": "/opt/homebrew/bin/stockfish",
    "uci_options": {
      "UCI_LimitStrength": "true",
      "UCI_Elo": "2000"
    }
  },
  "maia-1100": {
    "binary": "/usr/local/bin/lc0",
    "uci_options": {
      "WeightsFile": "/path/to/maia-1100.pb.gz"
    }
  }
}
```

- `binary`: Path to engine executable (relative to `engines/` or absolute)
- `uci_options`: Optional dict of UCI options sent after initialization

All engines must be listed in the config file to be recognized.

## Engine Competition (compete.py)

The `scripts/compete.py` script is a comprehensive engine testing framework with Elo tracking.

```bash
# Head-to-head match
./scripts/compete.py v27 v24 --games 100 --time 1.0

# Round-robin league
./scripts/compete.py v27 v24 v19 v1 --games 50 --time 0.5

# Random continuous competition
./scripts/compete.py --random --games 100 --time 0.5

# EPD endgame testing (Elo not updated)
./scripts/compete.py v27 v24 --epd eet.epd --time 1.0
```

**See [COMPETE.md](COMPETE.md) for full documentation** including:
- All competition modes (head-to-head, round-robin, gauntlet, random, EPD)
- Engine configuration and active/inactive status
- Elo rating system details
- Opening book information
- Time control options (fixed and random ranges)
- Troubleshooting guide

## Running Perft

Use *perft*, to determine the total number of positions encountered while playing through every move and every response to a certain depth.

```
position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1
READY
go perft 7
a5a4: 14,139,786  68,974,565 nps
a5a6: 16,022,983  74,845,580 nps
b4b1: 19,481,757  77,087,773 nps
b4b2: 12,755,330  77,515,349 nps
b4b3: 15,482,610  77,882,466 nps
b4f4: 3,069,955  78,064,051 nps
b4e4: 14,187,097  78,047,184 nps
b4d4: 15,996,777  78,099,996 nps
b4c4: 17,400,108  78,280,391 nps
b4a4: 11,996,400  78,291,255 nps
g2g3: 4,190,119  78,313,269 nps
g2g4: 13,629,805  78,198,877 nps
e2e3: 11,427,551  78,095,804 nps
e2e4: 8,853,383  78,005,965 nps
Time elapsed in perft is: 2.290587309s
178633661 nodes 78005965.50218341 nps
```