# Engine Competition Framework (compete.py)

A comprehensive engine vs engine testing harness with Elo tracking, multiple competition modes, and detailed statistics.

## Quick Start

```bash
# Head-to-head match between two engines
./scripts/compete.py v27 v24 --games 100 --time 1.0

# Round-robin league with multiple engines
./scripts/compete.py v27 v24 v19 v1 --games 50 --time 0.5

# Random continuous competition
./scripts/compete.py --random --games 100 --time 0.5

# EPD endgame testing
./scripts/compete.py v27 v24 --epd eet.epd --time 1.0
```

Engine names can be shorthand (`v27`, `v1`) or full (`v027-tablebase-fast`, `v001-baseline`).

---

## Competition Modes

### Head-to-Head Match (2 engines)

Direct match between two engines with alternating colors.

```bash
./scripts/compete.py v27 v24 --games 100 --time 1.0
```

- Each opening position is played twice (once per side)
- Uses the built-in opening book (250 positions) by default
- Shows running score and Elo after each game

### Round-Robin League (3+ engines)

Full round-robin tournament between all specified engines.

```bash
./scripts/compete.py v27 v24 v19 v1 --games 100 --time 0.5
```

- Creates all possible pairings (e.g., 4 engines = 6 pairings)
- Plays games in interleaved rounds across all pairings
- Shows league table after each round and after each game
- `--games` specifies games per pairing

### Gauntlet Mode

Test a single "challenger" engine against all active engines in the pool.

```bash
./scripts/compete.py v27 --gauntlet --games 50 --time 0.5
```

- Plays the challenger against every active engine
- Interleaved rounds: cycles through all opponents before repeating
- Each round = 1 game as white + 1 game as black per opponent
- Shows per-opponent statistics at the end

### Random Mode

Continuous random pairings from the active engine pool.

```bash
# Fixed time per move
./scripts/compete.py --random --games 100 --time 0.5

# Random time range per match
./scripts/compete.py --random --games 100 --timelow 0.1 --timehigh 1.0

# Weighted selection (engines with fewer games picked more often)
./scripts/compete.py --random --weighted --games 100 --time 0.5
```

- Randomly selects two active engines for each match
- Each match = 2 games (one per side) with the same opening
- Updates Elo ratings after each game
- Shows standings after every game
- With `--weighted`: selection probability = 1/(games+1)

### EPD Mode

Play through positions from an EPD file sequentially.

```bash
# Use EPD file from engines/epd/ directory
./scripts/compete.py v27 v24 --epd eet.epd --time 1.0

# Full path to EPD file
./scripts/compete.py v27 v24 --epd /path/to/positions.epd --time 0.5

# Round-robin with EPD positions
./scripts/compete.py v27 v24 v1 --epd pet.epd --time 0.5
```

- Each position played twice per pairing (once per side)
- Positions played sequentially (not random)
- **Elo ratings are NOT updated** (specialized positions would skew ratings)
- Shows standings after each position
- Useful for endgame testing with tablebase-enabled engines

**Included EPD files** (in `engines/epd/`):
- `eet.epd` - 100 positions (Eigenmann Endgame Test)
- `pet.epd` - 49 positions (Peter McKenzie pawn endgame test)

---

## Command Reference

### Engine Arguments

| Argument | Description |
|----------|-------------|
| `engine1 engine2 ...` | Engine names (shorthand or full) |

Shorthand resolution: `v27` -> `v027-tablebase-fast` (matches first engine starting with that prefix)

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
./scripts/compete.py v27 v24 --time 1.0

# Random time per match (only in random mode)
./scripts/compete.py --random --timelow 0.1 --timehigh 2.0
```

With `--timelow`/`--timehigh`, each match gets a randomly selected time control from that range.

---

## Engine Configuration

Engines are configured in `engines/engines.json`:

```json
{
  "v027-tablebase-fast": {
    "binary": "v027-tablebase-fast/rusty-rival",
    "active": true,
    "uci_options": {
      "SyzygyPath": "/path/to/syzygy"
    }
  },
  "sf-2000": {
    "binary": "/opt/homebrew/bin/stockfish",
    "active": true,
    "uci_options": {
      "UCI_LimitStrength": "true",
      "UCI_Elo": "2000"
    }
  },
  "maia-1500": {
    "binary": "/opt/homebrew/bin/lc0",
    "active": false,
    "uci_options": {
      "WeightsFile": "/path/to/maia-1500.pb.gz"
    }
  }
}
```

### Configuration Fields

| Field | Required | Description |
|-------|----------|-------------|
| `binary` | Yes | Path to executable (relative to `engines/` or absolute) |
| `active` | No | Whether engine participates in random/gauntlet modes (default: true) |
| `uci_options` | No | Dict of UCI options sent after initialization |

### Active vs Inactive Engines

- **Active engines** (`"active": true` or omitted): Included in random mode and gauntlet mode pools
- **Inactive engines** (`"active": false`): Can still be used in direct matches, but excluded from random pools

Use inactive status for:
- Older engine versions you want to preserve but not test regularly
- Engines with known bugs
- Engines at extreme strength levels that would skew random testing

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
v027-tablebase-fast: 1471 (26 games)?
v024-connected-passed-pawns: 1586 (492 games)
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

## Output

### Console Output

- Running score and results after each game
- League standings after each round (or each game in random/EPD mode)
- Final summary with Elo changes

### Files

| Location | Description |
|----------|-------------|
| `results/competitions/*.pgn` | PGN files for each competition |
| `results/elo_ratings.json` | Persistent Elo ratings database |

PGN filenames include timestamp and engine names:
- `v027-tablebase-fast_vs_v024-connected-passed-pawns_20260103_154230.pgn`
- `random_20260103_160000.pgn`
- `epd_eet_20260103_162000.pgn`

---

## Opening Book

The built-in opening book contains 250 balanced positions from major openings:

- Sicilian Defense (25 variations)
- Italian Game (15 variations)
- Ruy Lopez (20 variations)
- Queen's Gambit (20 variations)
- King's Indian Defense (15 variations)
- French Defense (15 variations)
- Caro-Kann (12 variations)
- English Opening (12 variations)
- Slav Defense (10 variations)
- Nimzo-Indian (12 variations)
- Scotch Game (8 variations)
- And more...

Each opening is played twice (once with each engine as white) to ensure fairness.

Disable with `--no-book` to start all games from the initial position.

---

## Examples

### Testing a New Engine Version

```bash
# Quick sanity check
./scripts/compete.py v27 v24 --games 10 --time 0.5

# Thorough head-to-head test
./scripts/compete.py v27 v24 --games 100 --time 1.0

# Test against multiple engines
./scripts/compete.py v27 v24 v19 v1 --games 50 --time 0.5

# Gauntlet against all active engines
./scripts/compete.py v27 --gauntlet --games 100 --time 0.5
```

### Endgame Testing (with tablebases)

```bash
# Test tablebase integration with endgame positions
./scripts/compete.py v27 v24 --epd eet.epd --time 1.0

# Pawn endgame test
./scripts/compete.py v27 v24 --epd pet.epd --time 0.5
```

### Continuous Rating Pool

```bash
# Run overnight with varied time controls
./scripts/compete.py --random --games 500 --timelow 0.25 --timehigh 2.0

# Weighted selection to balance game counts
./scripts/compete.py --random --weighted --games 200 --time 0.5
```

### Quick Smoke Test

```bash
# Fast 10-game match to verify engines work
./scripts/compete.py v27 v24 --games 10 --time 0.1 --no-book
```

---

## Troubleshooting

### Engine Not Found

```
Error: Engine 'v99' not found in engines/engines.json
Available engines: v001-baseline, v027-tablebase-fast, ...
```

Add the engine to `engines/engines.json` or check the name spelling.

### Binary Not Found

```
Error: Engine binary not found: engines/v99-missing/rusty-rival
```

Build the engine and copy the binary to the specified path.

### EPD File Not Found

```
Error: EPD file not found: custom.epd
Searched: /path/to/custom.epd and engines/epd/custom.epd
```

Provide the full path or place the file in `engines/epd/`.
