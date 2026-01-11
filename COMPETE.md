# Engine Competition Framework (compete.py)

A comprehensive engine vs engine testing harness with Elo tracking, multiple competition modes, and detailed statistics.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Setup](#setup)
3. [Competition Modes](#competition-modes)
4. [Command Reference](#command-reference)
5. [Engine Management](#engine-management)
6. [Elo Rating System](#elo-rating-system)
7. [Web Dashboard](#web-dashboard)
8. [Troubleshooting](#troubleshooting)

---

## Quick Start

Once setup is complete, here are the most common commands:

### Head-to-Head Match

```bash
# Linux/macOS
./scripts/compete.py v1.0.13 sf-2400 --games 100 --time 1.0

# Windows
python scripts/compete.py v1.0.13 sf-2400 --games 100 --time 1.0
```

### Round-Robin League (3+ engines)

```bash
./scripts/compete.py v1.0.13 sf-2400 sf-2600 sf-2800 --games 50 --time 1.0
```

### Gauntlet Mode (test one engine against all active engines)

```bash
./scripts/compete.py v1.0.13 --gauntlet --games 50 --time 0.5
```

### Random Mode (continuous random pairings)

```bash
./scripts/compete.py --random --games 100 --time 0.5
```

### EPD Testing (specific positions, Elo not updated)

```bash
./scripts/compete.py v1.0.13 sf-2800 --epd eet.epd --time 1.0
```

**Engine name shorthand:** Use `v12` instead of `v012-my-feature`, `sf-2400` for Stockfish at Elo 2400.

---

## Setup

### Prerequisites

- Python 3.10+
- PostgreSQL database
- At least one chess engine (Stockfish or Rusty Rival)

### Step 1: Python Environment

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

If PowerShell gives an execution policy error:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Step 2: Database Setup

The framework requires a PostgreSQL database for Elo tracking and game history.

#### 2a. Create the Database

Run this SQL to create the schema:

```sql
CREATE TABLE engines (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    binary_path VARCHAR(500),
    active BOOLEAN DEFAULT TRUE,
    initial_elo INTEGER DEFAULT 1500,
    uci_options JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    white_engine_id INTEGER NOT NULL REFERENCES engines(id),
    black_engine_id INTEGER NOT NULL REFERENCES engines(id),
    result VARCHAR(10) NOT NULL,
    white_score NUMERIC(2,1) NOT NULL,
    black_score NUMERIC(2,1) NOT NULL,
    date_played DATE NOT NULL,
    time_control VARCHAR(50),
    opening_name VARCHAR(100),
    opening_fen TEXT,
    pgn TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE elo_ratings (
    id SERIAL PRIMARY KEY,
    engine_id INTEGER NOT NULL UNIQUE REFERENCES engines(id),
    elo NUMERIC(7,2) NOT NULL,
    games_played INTEGER DEFAULT 0,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_games_white_engine ON games(white_engine_id);
CREATE INDEX idx_games_black_engine ON games(black_engine_id);
CREATE INDEX idx_games_date ON games(date_played);
CREATE INDEX idx_engines_active ON engines(active);
```

#### 2b. Configure Environment

Create a `.env` file in the project root:

```
DATABASE_URL=postgresql://username:password@hostname:5432/database_name
```

Example for local development:
```
DATABASE_URL=postgresql://postgres:password@localhost:5432/rusty_rival
```

#### 2c. Add Engines to Database

Engines are auto-created with `active=false` when first used. To include an engine in random/gauntlet modes, set `active=true`:

```sql
-- Add and activate Stockfish engines
INSERT INTO engines (name, active, initial_elo) VALUES
    ('sf-1400', true, 1400),
    ('sf-1600', true, 1600),
    ('sf-1800', true, 1800),
    ('sf-2000', true, 2000),
    ('sf-2200', true, 2200),
    ('sf-2400', true, 2400),
    ('sf-2600', true, 2600),
    ('sf-2800', true, 2800),
    ('sf-3000', true, 3000),
    ('sf-full', true, 3500);

-- Add Rusty Rival version
INSERT INTO engines (name, active, initial_elo) VALUES
    ('v1.0.13', true, 2600);

-- Create initial Elo ratings
INSERT INTO elo_ratings (engine_id, elo, games_played)
SELECT id, initial_elo, 0 FROM engines;
```

### Step 3: Install Engines

#### Stockfish

Download from [stockfishchess.org](https://stockfishchess.org/download) and place in `engines/stockfish/`:

```
engines/
  stockfish/
    stockfish-windows-x86-64-avx2.exe   # Windows
    stockfish-ubuntu-x86-64-avx2        # Linux
    stockfish-macos-m1-apple-silicon    # macOS Apple Silicon
```

The script auto-creates virtual Stockfish engines: `sf-1400`, `sf-1600`, `sf-1800`, `sf-2000`, `sf-2200`, `sf-2400`, `sf-2600`, `sf-2800`, `sf-3000`, and `sf-full`.

#### Rusty Rival

Use `--init` to download from GitHub releases automatically:

```bash
# Download rusty-rival (auto-detects platform, sets permissions)
python scripts/compete.py --init rusty v1.0.15
```

This downloads the correct binary for your platform to `engines/v1.0.15/` and handles macOS quarantine removal and Unix executable permissions.

Manual download is also supported - get from [GitHub releases](https://github.com/chris-moreton/rusty-rival/releases):

```
engines/
  v1.0.13/
    rusty-rival-v1.0.13-windows-x86_64.exe   # Windows
    rusty-rival-v1.0.13-linux-x86_64         # Linux
    rusty-rival-v1.0.13-macos-aarch64        # macOS
```

Legacy naming (`rusty-rival.exe` / `rusty-rival`) is also supported.

#### Java Rival

Use `--init` to download from GitHub releases:

```bash
# Download java-rival JAR (cross-platform)
python scripts/compete.py --init java 38
```

This downloads to `engines/java-rival-38.0.0/rivalchess-v38.0.0.jar`.

---

## Competition Modes

### Head-to-Head Match (2 engines)

Direct match between two engines with alternating colors.

- Each opening played twice (once per side)
- Uses built-in opening book (250 positions) by default
- Shows running score and Elo after each game
- Elo ratings **are updated**

```bash
./scripts/compete.py v1.0.13 sf-2400 --games 100 --time 1.0
```

### Round-Robin League (3+ engines)

Full round-robin tournament between all specified engines.

- Creates all possible pairings (4 engines = 6 pairings)
- Plays games interleaved across all pairings
- Shows league table after each game
- `--games` specifies games per pairing
- Elo ratings **are updated**

```bash
./scripts/compete.py v1.0.13 sf-2400 sf-2600 sf-2800 --games 50 --time 1.0
```

### Gauntlet Mode

Test a single "challenger" engine against all **active** engines in the database.

- Plays the challenger against every active engine
- Interleaved rounds: cycles through all opponents before repeating
- Each round = 1 game as white + 1 game as black per opponent
- Shows per-opponent statistics at the end
- Elo ratings **are updated**

```bash
./scripts/compete.py v1.0.13 --gauntlet --games 50 --time 0.5
```

**Important:** Only engines with `active=true` in the database are included as opponents.

### Random Mode

Continuous random pairings from all **active** engines.

- Randomly selects two active engines for each match
- Each match = 2 games (one per side) with the same opening
- Shows standings after every game
- Elo ratings **are updated**
- **Live updates**: Re-checks active engines before each match, so you can enable/disable engines without restarting

```bash
./scripts/compete.py --random --games 100 --time 0.5

# Weighted: favor engines with fewer games played
./scripts/compete.py --random --weighted --games 100 --time 0.5
```

**Important:** Only engines with `active=true` in the database are included. Use `--enable` and `--disable` to control which engines participate (changes take effect on the next match).

### EPD Mode

Play through positions from an EPD file sequentially.

- Each position played twice per pairing (once per side)
- Positions played in order (not random)
- Elo ratings **are NOT updated** (specialized positions would skew ratings)

```bash
./scripts/compete.py v1.0.13 sf-2800 --epd eet.epd --time 1.0
```

**Included EPD files** (in `engines/epd/`):
- `eet.epd` - 100 positions (Eigenmann Endgame Test)
- `pet.epd` - 49 positions (Peter McKenzie pawn endgame test)

---

## Command Reference

### Syntax

```bash
# Linux/macOS
./scripts/compete.py [engines...] [options]

# Windows
python scripts/compete.py [engines...] [options]
```

### Options

| Option | Description |
|--------|-------------|
| `--games N`, `-g N` | Number of games/rounds (default: 100) |
| `--time T`, `-t T` | Time per move in seconds (default: 1.0) |
| `--timelow T` | Minimum time per move (use with `--timehigh`) |
| `--timehigh T` | Maximum time per move (use with `--timelow`) |
| `--no-book` | Disable opening book (start from initial position) |
| `--gauntlet` | Gauntlet mode: test one engine against all active engines |
| `--random` | Random mode: continuous random pairings |
| `--weighted` | With `--random`: favor engines with fewer games played |
| `--epd FILE` | EPD mode: play positions from file |
| `--list` | List all engines with their active status |
| `--enable ENGINE...` | Enable one or more engines |
| `--disable ENGINE...` | Disable one or more engines |
| `--init TYPE VERSION` | Download engine from GitHub releases (see below) |

### Time Control Examples

```bash
# Fixed time per move (1 second)
python scripts/compete.py v1.0.13 sf-2400 --time 1.0

# Random time per match (only in random mode)
python scripts/compete.py --random --timelow 0.1 --timehigh 2.0
```

### Engine Name Shorthand

You can use shorthand names instead of full names:

| Shorthand | Resolves to |
|-----------|-------------|
| `v1` | `v001-baseline` |
| `v12` | `v012-my-test-version` |
| `v1.0.13` | `v1.0.13` (semantic version) |
| `sf-2400` | Stockfish at 2400 Elo |

The script matches shorthand against available engines. Exact matches take priority.

---

## Engine Management

### Directory Structure

```
engines/
  stockfish/
    stockfish-windows-x86-64-avx2.exe
    stockfish-ubuntu-x86-64-avx2
  v1.0.13/
    rusty-rival-v1.0.13-windows-x86_64.exe
  v027-my-local-test/
    rusty-rival.exe          # Windows
    rusty-rival              # Linux/macOS
  java-rival-37.0.0/
    rivalchess-v37.0.0.jar   # Java engine (cross-platform)
  epd/
    eet.epd
    pet.epd
  syzygy/                    # Optional: Syzygy tablebases
    KQvK.rtbw
```

### Engine Discovery

The script automatically discovers engines:

1. **Stockfish**: Looks in `engines/stockfish/` for known binary names
2. **Rusty Rival versions**: Looks in `engines/v*/` directories
   - First tries versioned names: `rusty-rival-v1.0.13-*`
   - Falls back to `rusty-rival.exe` (Windows) or `rusty-rival` (Unix)
3. **Java engines**: Looks in `engines/java-rival-*/` for `.jar` files
   - Runs with `java -jar` (cross-platform, no wrapper script needed)
   - Example: `engines/java-rival-37.0.0/rivalchess-v37.0.0.jar` → `java-rival-37`
4. **Virtual Stockfish engines**: Auto-creates `sf-1400` through `sf-3000` using UCI_LimitStrength

### Enabling and Disabling Engines

Control which engines are included in random and gauntlet modes:

```bash
# List all engines with their status
python scripts/compete.py --list

# Disable engines (won't be selected in random/gauntlet mode)
python scripts/compete.py --disable sf-1400 sf-full java-rival-36

# Enable engines
python scripts/compete.py --enable java-rival-37 v1.0.13
```

Example output from `--list`:
```
Engine                         Status          Elo    Games
----------------------------------------------------------
sf-3000                        active         2991      510
sf-2800                        active         2705     1472
v1.0.13                        active         2408      143
java-rival-37                  disabled       1500        0
```

**Note:** Changes take effect immediately. In random mode, the engine list is refreshed before each match, so you can enable/disable engines while a competition is running.

### Building Local Test Versions

When developing, create local builds for comparison testing.

**Naming convention:** `vNNN-description` (e.g., `v030-better-eval`)

**Linux/macOS:**
```bash
RUSTFLAGS="-C target-cpu=native" RUST_MIN_STACK=4097152 cargo build --release
mkdir -p engines/v030-my-new-feature
cp target/release/rusty-rival engines/v030-my-new-feature/
```

**Windows (PowerShell):**
```powershell
$env:RUSTFLAGS="-C target-cpu=native"; $env:RUST_MIN_STACK=4097152; cargo build --release
mkdir engines\v030-my-new-feature
copy target\release\rusty-rival.exe engines\v030-my-new-feature\
```

---

## Elo Rating System

Elo ratings are stored in the PostgreSQL database (`elo_ratings` table).

### Rating Calculation

- **Initial rating**: Based on the `initial_elo` column in the `engines` table (default: 1500)
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
| EPD | **No** |

---

## Web Dashboard

A Flask-based web interface displays head-to-head statistics and Elo ratings.

### Running the Dashboard

**Linux/macOS:**
```bash
source .venv/bin/activate
python -m web.app
```

**Windows:**
```cmd
.venv\Scripts\activate.bat
python -m web.app
```

The dashboard runs at `http://localhost:5000`.

### Features

- **H2H Grid**: Head-to-head scores between all engine pairs
- **Elo Rankings**: Engines sorted by current rating
- **Performance Colors**: Green (overperforming), Red (underperforming), White (expected)
- **Tooltips**: Hover for detailed stats
- **Active Filter**: Add `?all=1` to URL to show inactive engines

---

## Troubleshooting

### Python Not Found

- **Windows**: `winget install Python.Python.3.12`
- **macOS**: `brew install python`
- **Linux**: `sudo apt install python3`

### Virtual Environment Activation Fails

**Windows PowerShell execution policy error:**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Windows - script opens in Notepad:**
```powershell
& .\.venv\Scripts\Activate.ps1
```

### Engine Not Found

```
Error: Engine 'v99' not found
```

Check:
1. Engine directory exists in `engines/`
2. Binary is inside with correct name
3. Binary has execute permissions (Linux/macOS: `chmod +x`)

### Stockfish Not Found

Ensure the binary is in `engines/stockfish/` with a recognized name:
- `stockfish-windows-x86-64-avx2.exe`
- `stockfish-ubuntu-x86-64-avx2`
- `stockfish-macos-m1-apple-silicon`
- `stockfish.exe` / `stockfish`

### EPD File Not Found

Provide the full path or place the file in `engines/epd/`.

---

## Opening Book

The built-in opening book contains 250 balanced positions from major openings (Sicilian, Italian, Ruy Lopez, Queen's Gambit, King's Indian, French, Caro-Kann, English, etc.).

Each opening is played twice (once with each engine as white) for fairness.

Disable with `--no-book` to start all games from the initial position.
