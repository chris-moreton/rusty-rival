# Engine Competition Framework (compete.py)

A comprehensive engine vs engine testing harness with Elo tracking, multiple competition modes, and detailed statistics.

---

## Setup

### Prerequisites

- Python 3.10+
- A virtual environment with required packages

### Step 1: Python Environment Setup

**Linux/macOS:**
```bash
cd /path/to/rusty-rival
python3 -m venv .venv
source .venv/bin/activate
pip install -r web/requirements.txt
```

**Windows (PowerShell):**
```powershell
cd C:\path\to\rusty-rival
python -m venv .venv
& .\.venv\Scripts\Activate.ps1
pip install -r web/requirements.txt
```

**Windows (cmd.exe):**
```cmd
cd C:\path\to\rusty-rival
python -m venv .venv
.venv\Scripts\activate.bat
pip install -r web/requirements.txt
```

This installs all required packages including:
- `python-chess` - Chess library for game management
- `Flask` and `Flask-SQLAlchemy` - Database integration for Elo tracking
- `psycopg` - PostgreSQL driver (for database connection)
- `python-dotenv` - Environment variable loading

If PowerShell gives an execution policy error, run this first:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Step 2: Environment Variables

Create a `.env` file in the project root for database configuration (optional, for Elo tracking):

```
DATABASE_URL=your_database_url_here
```

### Step 3: Installing Stockfish

Download Stockfish from [stockfishchess.org/download](https://stockfishchess.org/download) or [GitHub releases](https://github.com/official-stockfish/Stockfish/releases).

Place the binary in `engines/stockfish/`:

```
engines/
  stockfish/
    stockfish-windows-x86-64-avx2.exe   # Windows
    stockfish-ubuntu-x86-64-avx2        # Linux
    stockfish-macos-m1-apple-silicon    # macOS Apple Silicon
```

The script auto-detects Stockfish and creates virtual engines at various Elo levels:
- `sf-1400`, `sf-1600`, `sf-1800`, `sf-2000`, `sf-2200`, `sf-2400`, `sf-2600`, `sf-2800`, `sf-3000`
- `sf-full` (full strength)

### Step 4: Installing Rusty Rival Versions

Download release binaries from [GitHub releases](https://github.com/chris-moreton/rusty-rival/releases) or build from source.

**Release binaries:**

Create a version directory and place the binary inside:

```
engines/
  v1.0.13/
    rusty-rival-v1.0.13-windows-x86_64.exe   # Windows release
    rusty-rival-v1.0.13-linux-x86_64         # Linux release
    rusty-rival-v1.0.13-macos-aarch64        # macOS release
```

The script recognizes versioned binary names (`rusty-rival-vX.X.X-*`) automatically.

**Legacy naming (also supported):**

```
engines/
  v1.0.13/
    rusty-rival.exe   # Windows
    rusty-rival       # Linux/macOS
```

---

## Building Local Test Versions

When developing, you can create local test builds to compare against other versions.

### Naming Convention

Use the format `vNNN-description` where:
- `NNN` is a zero-padded version number (e.g., `012`, `027`)
- `description` is a short identifier for the changes

Examples:
- `v012-my-test-version`
- `v027-tablebase-fast`
- `v030-better-eval`

### Creating a Test Build

**Linux/macOS:**
```bash
# Build optimized for your CPU
RUSTFLAGS="-C target-cpu=native" RUST_MIN_STACK=4097152 cargo build --release

# Create version directory and copy binary
mkdir -p engines/v030-my-new-feature
cp target/release/rusty-rival engines/v030-my-new-feature/
```

**Windows (PowerShell):**
```powershell
# Build optimized for your CPU
$env:RUSTFLAGS="-C target-cpu=native"; $env:RUST_MIN_STACK=4097152; cargo build --release

# Create version directory and copy binary
mkdir engines\v030-my-new-feature
copy target\release\rusty-rival.exe engines\v030-my-new-feature\
```

**Windows (cmd.exe):**
```cmd
set RUSTFLAGS=-C target-cpu=native
set RUST_MIN_STACK=4097152
cargo build --release

mkdir engines\v030-my-new-feature
copy target\release\rusty-rival.exe engines\v030-my-new-feature\
```

---

## Running Competitions

### Basic Syntax

**Linux/macOS:**
```bash
./scripts/compete.py [engines...] [options]
```

**Windows (with venv activated):**
```cmd
python scripts/compete.py [engines...] [options]
```

### Engine Name Shorthand

You can use shorthand names instead of full names:

| Shorthand | Resolves to |
|-----------|-------------|
| `v1` | `v001-baseline` |
| `v12` | `v012-my-test-version` |
| `v1.0.13` | `v1.0.13` (semantic version) |
| `sf-2400` | Stockfish at 2400 Elo |

The script matches the shorthand against available engines:
- `v12` matches the first engine starting with `v012-`, `v12-`, or `v12.`
- Exact matches take priority

---

## Quick Start Examples

### Head-to-Head Match

**Linux/macOS:**
```bash
./scripts/compete.py v1.0.13 sf-2400 --games 100 --time 1.0
```

**Windows:**
```cmd
python scripts/compete.py v1.0.13 sf-2400 --games 100 --time 1.0
```

### Round-Robin League

Test multiple engines against each other:

**Linux/macOS:**
```bash
./scripts/compete.py v1.0.13 sf-2400 sf-2600 sf-2800 sf-3000 --games 100 --time 1.0
```

**Windows:**
```cmd
python scripts/compete.py v1.0.13 sf-2400 sf-2600 sf-2800 sf-3000 --games 100 --time 1.0
```

### Random Mode

Continuous random pairings from all active engines:

**Linux/macOS:**
```bash
./scripts/compete.py --random --games 100 --time 0.5
```

**Windows:**
```cmd
python scripts/compete.py --random --games 100 --time 0.5
```

### Gauntlet Mode

Test one engine against all others:

**Linux/macOS:**
```bash
./scripts/compete.py v1.0.13 --gauntlet --games 50 --time 0.5
```

**Windows:**
```cmd
python scripts/compete.py v1.0.13 --gauntlet --games 50 --time 0.5
```

### EPD Endgame Testing

Test with specific positions from an EPD file:

**Linux/macOS:**
```bash
./scripts/compete.py v1.0.13 sf-2800 --epd eet.epd --time 1.0
```

**Windows:**
```cmd
python scripts/compete.py v1.0.13 sf-2800 --epd eet.epd --time 1.0
```

---

## Competition Modes

### Head-to-Head Match (2 engines)

Direct match between two engines with alternating colors.

- Each opening position is played twice (once per side)
- Uses the built-in opening book (250 positions) by default
- Shows running score and Elo after each game

### Round-Robin League (3+ engines)

Full round-robin tournament between all specified engines.

- Creates all possible pairings (e.g., 4 engines = 6 pairings)
- Plays games in interleaved rounds across all pairings
- Shows league table after each round and after each game
- `--games` specifies games per pairing

### Gauntlet Mode

Test a single "challenger" engine against all active engines in the pool.

- Plays the challenger against every active engine
- Interleaved rounds: cycles through all opponents before repeating
- Each round = 1 game as white + 1 game as black per opponent
- Shows per-opponent statistics at the end

### Random Mode

Continuous random pairings from the active engine pool.

- Randomly selects two active engines for each match
- Each match = 2 games (one per side) with the same opening
- Updates Elo ratings after each game
- Shows standings after every game
- With `--weighted`: selection probability = 1/(games+1)

### EPD Mode

Play through positions from an EPD file sequentially.

- Each position played twice per pairing (once per side)
- Positions played sequentially (not random)
- **Elo ratings are NOT updated** (specialized positions would skew ratings)
- Shows standings after each position

**Included EPD files** (in `engines/epd/`):
- `eet.epd` - 100 positions (Eigenmann Endgame Test)
- `pet.epd` - 49 positions (Peter McKenzie pawn endgame test)

---

## Command Reference

### Options

| Option | Description |
|--------|-------------|
| `--games N`, `-g N` | Number of games/rounds/matches (default: 100) |
| `--time T`, `-t T` | Time per move in seconds (default: 1.0) |
| `--timelow T` | Minimum time per move (use with `--timehigh`) |
| `--timehigh T` | Maximum time per move (use with `--timelow`) |
| `--no-book` | Disable opening book (start from initial position) |
| `--gauntlet` | Enable gauntlet mode |
| `--random` | Enable random mode |
| `--weighted` | With `--random`: favor engines with fewer games |
| `--epd FILE` | EPD mode: play positions from file |

### Time Control

```bash
# Fixed time per move
python scripts/compete.py v1.0.13 sf-2400 --time 1.0

# Random time per match (only in random mode)
python scripts/compete.py --random --timelow 0.1 --timehigh 2.0
```

---

## Engine Directory Structure

```
engines/
  stockfish/
    stockfish-windows-x86-64-avx2.exe
    stockfish-ubuntu-x86-64-avx2
    stockfish-macos-m1-apple-silicon
  v1.0.13/
    rusty-rival-v1.0.13-windows-x86_64.exe
  v1.0.12/
    rusty-rival-v1.0.12-linux-x86_64
  v027-my-local-test/
    rusty-rival.exe          # Windows
    rusty-rival              # Linux/macOS
  epd/
    eet.epd
    pet.epd
  syzygy/                    # Optional: Syzygy tablebases
    KQvK.rtbw
    ...
```

### Engine Discovery

The script automatically discovers engines:

1. **Stockfish**: Looks in `engines/stockfish/` for known binary names
2. **Rusty Rival versions**: Looks in `engines/v*/` directories
   - First tries versioned names: `rusty-rival-v1.0.13-*`
   - Falls back to `rusty-rival.exe` (Windows) or `rusty-rival` (Unix)
3. **Virtual Stockfish engines**: Auto-creates `sf-1400` through `sf-3000` using UCI_LimitStrength

---

## Elo Rating System

The framework maintains persistent Elo ratings in `results/elo_ratings.json`.

### Rating Calculation

- **Initial rating**: Average of existing ratings (or 1500 if first engine)
- **K-factor**: 40 for provisional ratings (<30 games), 20 for established
- **Update**: Standard Elo formula after each game

### Provisional Ratings

Ratings are marked with `?` until an engine has played 30+ games:

```
v1.0.13: 1471 (26 games)?
v1.0.12: 1586 (492 games)
```

### When Ratings Are Updated

| Mode | Updates Elo? |
|------|--------------|
| Head-to-head | Yes |
| Round-robin | Yes |
| Gauntlet | Yes |
| Random | Yes |
| EPD | **No** (specialized positions would skew ratings) |

---

## Output Files

| Location | Description |
|----------|-------------|
| `results/competitions/*.pgn` | PGN files for each competition |
| `results/elo_ratings.json` | Persistent Elo ratings database |

PGN filenames include timestamp and engine names:
- `v1.0.13_vs_sf-2400_20260110_154230.pgn`
- `random_20260110_160000.pgn`
- `epd_eet_20260110_162000.pgn`

---

## Troubleshooting

### Python Not Found

Install Python:
- **Windows**: `winget install Python.Python.3.12`
- **macOS**: `brew install python`
- **Linux**: `sudo apt install python3`

### Virtual Environment Activation Fails

**Windows PowerShell execution policy error:**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Windows - script opens in Notepad:**
Use `&` operator or the batch file:
```powershell
& .\.venv\Scripts\Activate.ps1
# or
.\.venv\Scripts\activate.bat
```

### Engine Not Found

```
Error: Engine 'v99' not found
Available engines: v1.0.13, sf-2400, ...
```

Check that:
1. The engine directory exists in `engines/`
2. The binary is inside with the correct name
3. The binary has execute permissions (Linux/macOS: `chmod +x`)

### Binary Not Found

```
Error: Engine binary not found: engines/v1.0.13/rusty-rival
```

The script looks for binaries in this order:
1. `rusty-rival-v1.0.13-*` (versioned name)
2. `rusty-rival.exe` (Windows)
3. `rusty-rival` (Linux/macOS)

### Stockfish Not Found

Ensure the Stockfish binary is in `engines/stockfish/` with one of these names:
- `stockfish-windows-x86-64-avx2.exe`
- `stockfish-windows-x86-64.exe`
- `stockfish.exe`
- `stockfish-ubuntu-x86-64-avx2`
- `stockfish-linux-x86_64`
- `stockfish-macos-m1-apple-silicon`
- `stockfish`

### EPD File Not Found

```
Error: EPD file not found: custom.epd
```

Provide the full path or place the file in `engines/epd/`.

---

## Opening Book

The built-in opening book contains 250 balanced positions from major openings:

- Sicilian Defense, Italian Game, Ruy Lopez
- Queen's Gambit, King's Indian Defense
- French Defense, Caro-Kann
- English Opening, Slav Defense
- Nimzo-Indian, Scotch Game
- And more...

Each opening is played twice (once with each engine as white) to ensure fairness.

Disable with `--no-book` to start all games from the initial position.
