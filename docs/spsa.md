# SPSA Tuning for Rusty Rival

This document describes SPSA (Simultaneous Perturbation Stochastic Approximation) tuning for optimizing engine parameters.

## Why SPSA?

Hand-tuning parameters failed in the v1.0.22 RC series - all release candidates lost to v1.0.21 despite implementing standard techniques (razoring, singular extensions, etc.). The problem wasn't the techniques themselves, but the parameter values.

SPSA solves this by:
- **Finding non-obvious interactions** - parameters affect each other; tuning one at a time misses this
- **Objective measurement** - game results don't lie, unlike intuition
- **Handling many parameters** - can tune 50+ values simultaneously

## How SPSA Works

1. **Start with current parameter values** (e.g., futility margins, LMR coefficients)

2. **Perturb all parameters simultaneously** - randomly add or subtract a small amount (δ) to each parameter

3. **Play games** between:
   - Version with +δ perturbations vs baseline, OR
   - Version with +δ vs version with -δ (more efficient)

4. **Measure results** - win/loss/draw rates converted to Elo difference

5. **Estimate gradient** - if +δ version won more, parameters should move in + direction

6. **Update parameters** - move in the direction that improved performance:
   ```
   θ_new = θ_old + a * gradient_estimate
   ```

7. **Repeat** thousands of times, decreasing step size (a) over time

## Parameters to Tune

### Search Parameters

| Parameter | Current Value | Description |
|-----------|---------------|-------------|
| `LMR_LEGAL_MOVES_BEFORE_ATTEMPT` | 4 | Moves before LMR kicks in |
| `LMR_MIN_DEPTH` | 3 | Minimum depth for LMR |
| LMR formula coefficients | 0.75, 2.5 | `floor(0.75 + ln(d)*ln(m)/2.5)` |
| `BETA_PRUNE_MARGIN_PER_DEPTH` | 200 | Reverse futility margin |
| `BETA_PRUNE_MAX_DEPTH` | 3 | Max depth for reverse futility |
| `ALPHA_PRUNE_MARGINS` | [128,192,256,320,384,448,512,576] | Futility margins by depth |
| `NULL_MOVE_MIN_DEPTH` | 4 | Min depth for null move |
| `NULL_MOVE_REDUCE_DEPTH_BASE` | 3 | Base reduction for null move |
| `SEE_PRUNE_MARGIN` | 20 | SEE pruning threshold multiplier |
| `SEE_PRUNE_MAX_DEPTH` | 6 | Max depth for SEE pruning |
| `LMP_MOVE_THRESHOLDS` | [0,8,12,16] | Late move pruning thresholds |
| `THREAT_EXTENSION_MARGIN` | 400 | Threshold for threat detection |
| Aspiration window sizes | [25,50,100,200,400,800] | Window widening sequence |

### Potential Future Parameters (if re-added)

| Parameter | Notes |
|-----------|-------|
| `RAZOR_MARGINS` | Tried [0,150,300], caused regression |
| `RAZOR_MAX_DEPTH` | Tried 2, caused regression |
| `SINGULAR_MIN_DEPTH` | Tried 8, caused regression |
| `SINGULAR_MARGIN` | Tried 3, caused regression |

### Evaluation Parameters

| Parameter | Current Value | Description |
|-----------|---------------|-------------|
| Piece values | 100,325,325,500,975 | P,N,B,R,Q |
| `BISHOP_VALUE_PAIR` | 50 | Bonus for bishop pair |
| `DOUBLED_PAWN_PENALTY` | 10 | Penalty per doubled pawn |
| `ISOLATED_PAWN_PENALTY` | 15 | Penalty per isolated pawn |
| Mobility weights | various | Per-piece mobility bonuses |
| King safety weights | various | Threat bonuses by piece type |
| `SPACE_BONUS_PER_SQUARE` | 2 | Space evaluation weight |

## Time Controls for Tuning

### Tradeoffs

| Time Control | Pros | Cons |
|--------------|------|------|
| Fast (1-5s/game) | More games, faster convergence | May optimize for bullet |
| Slow (60s+/game) | Realistic depths | Fewer games, higher variance |

### Recommendation

**5-10 seconds per game** (both sides combined):
- Fast enough for hundreds of games per iteration
- Deep enough that search parameters matter
- Parameters that help here usually help at longer time controls

Stockfish uses ~10s/game for SPSA tuning.

## Games Required

### Estimating Sample Size

| Factor | Typical Value |
|--------|---------------|
| Elo gain to detect | 3-5 Elo per iteration |
| Draw rate | 30-40% |
| Confidence level | 95% |
| Games per iteration | 100-200 |

### Total Games by Scope

| Parameters | Iterations | Games/Iteration | Total Games |
|------------|------------|-----------------|-------------|
| 5 (focused) | 250 | 150 | 37,500 |
| 10 (moderate) | 500 | 150 | 75,000 |
| 20 (comprehensive) | 1,000 | 150 | 150,000 |

### Time Estimates

At 10s/game with parallelization:

| Parallel Games | Games/Hour | 150k Games |
|----------------|------------|------------|
| 1 | 360 | 17 days |
| 4 | 1,440 | 4 days |
| 8 | 2,880 | 2 days |

## Parallelization

### Why It Works

Rusty Rival is single-threaded, so on an 8-core machine:
- 1 game uses 1 core (the thinking engine), 7 cores idle
- 8 parallel games use 8 cores, each at ~85-95% NPS
- Total throughput: ~7x improvement

### NPS Impact

| Parallel Games | NPS per Game | Reason |
|----------------|--------------|--------|
| 1 | 100% | Full turbo boost |
| 4 | 95-98% | Minor cache pressure |
| 8 | 85-95% | Turbo reduction, shared L3 |

The slight per-game slowdown is vastly outweighed by throughput gains.

### Resource Considerations

- **Memory**: 8 games × 2 engines × 128MB hash = 2GB (usually fine)
- **L3 Cache**: Shared, some contention with many games
- **Turbo Boost**: Reduced when all cores loaded

## Implementation Architecture

### Decision: Build on chess-compete

After evaluating options (OpenBench, cutechess-cli, custom), we're building on chess-compete because:
- Already has parallel game execution (`-c N` flag)
- Already has Elo tracking and database
- Familiar codebase
- Can customize for our specific needs

### Key Design Decisions

1. **No individual game saves** - SPSA mode does NOT write to the `games` table
   - Only aggregate results (wins/losses/draws) stored in `spsa_iteration`
   - Avoids polluting database with thousands of test games
   - Faster execution (no per-game DB writes)

2. **No engine registration** - SPSA engines are ephemeral
   - No `--init` needed, engines not added to `engines` table
   - Binaries loaded directly from path
   - Overwritten each iteration (or use rotating names)

3. **Random openings** - each game uses a random opening from `OPENING_BOOK`
   - Prevents overfitting to specific positions
   - Same book used by other compete modes

4. **Time variety** - use `timelow`/`timehigh` for random time per game
   - Consistent naming with other compete modes
   - Helps ensure parameters work across time controls

### Distributed Worker Architecture

To scale across multiple machines, we use a master/worker pattern with the shared database:

```
┌─────────────────────────────────────────────────────────────────┐
│              MASTER MACHINE (rusty-rival repo)                   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  python -m spsa.master                                   │   │
│  │  - Reads spsa/params.toml (current parameter values)     │   │
│  │  - Generates perturbed engine_constants.rs               │   │
│  │  - Builds engine binaries (cargo build --release)        │   │
│  │  - Copies to shared location (engines/spsa-plus, etc.)   │   │
│  │  - Creates spsa_iteration record in database             │   │
│  │  - Monitors game count for current iteration             │   │
│  │  - When target_games reached: calculates gradient        │   │
│  │  - Updates spsa/params.toml with new values              │   │
│  │  - Creates next iteration                                │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Database (shared) + Engine binaries (shared path)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      spsa_iteration table                        │
│  - id, iteration_number                                          │
│  - plus_engine_path, minus_engine_path                          │
│  - target_games, games_played                                    │
│  - plus_wins, minus_wins, draws                                  │
│  - timelow_ms, timehigh_ms (for random time per game)           │
│  - status (pending/in_progress/complete)                         │
│  - parameter_snapshot (JSON of perturbed values)                │
│  - created_at, completed_at                                      │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌───────────────────┐ ┌───────────────────┐ ┌───────────────────┐
│  WORKER MACHINE 1 │ │  WORKER MACHINE 2 │ │  WORKER MACHINE N │
│  (chess-compete)  │ │  (chess-compete)  │ │  (chess-compete)  │
│                   │ │                   │ │                   │
│  compete --spsa   │ │  compete --spsa   │ │  compete --spsa   │
│  -c 8             │ │  -c 8             │ │  -c 8             │
│                   │ │                   │ │                   │
│  - Polls database │ │  - Polls database │ │  - Polls database │
│  - Loads engines  │ │  - Loads engines  │ │  - Loads engines  │
│    from path      │ │    from path      │ │    from path      │
│  - Random opening │ │  - Random opening │ │  - Random opening │
│  - Random time    │ │  - Random time    │ │  - Random time    │
│    (timelow-high) │ │    (timelow-high) │ │    (timelow-high) │
│  - Updates counts │ │  - Updates counts │ │  - Updates counts │
│  - NO game saves  │ │  - NO game saves  │ │  - NO game saves  │
└───────────────────┘ └───────────────────┘ └───────────────────┘
```

### Repo Responsibilities

**rusty-rival** (master logic):
- `spsa/master.py` - orchestration loop
- `spsa/params.toml` - current parameter values
- `spsa/config.toml` - SPSA hyperparameters and time controls
- `spsa/build.py` - generates `engine_constants.rs` and builds
- Owns the parameter definitions and tuning state

**chess-compete** (worker logic):
- `compete/spsa/worker.py` - polls and runs games
- `--spsa` CLI flag to enter worker mode
- Database tables (`spsa_iteration`)
- Game execution infrastructure (already exists)

### Database Schema: spsa_iteration

```sql
CREATE TABLE spsa_iteration (
    id INTEGER PRIMARY KEY,
    iteration_number INTEGER NOT NULL,

    -- Engine binaries (paths to shared location)
    plus_engine_path VARCHAR(255) NOT NULL,
    minus_engine_path VARCHAR(255) NOT NULL,

    -- Time control (consistent with other compete modes)
    timelow_ms INTEGER NOT NULL,      -- e.g., 250 (0.25s)
    timehigh_ms INTEGER NOT NULL,     -- e.g., 1000 (1.0s)

    -- Game tracking
    target_games INTEGER NOT NULL DEFAULT 150,
    games_played INTEGER NOT NULL DEFAULT 0,
    plus_wins INTEGER NOT NULL DEFAULT 0,
    minus_wins INTEGER NOT NULL DEFAULT 0,
    draws INTEGER NOT NULL DEFAULT 0,

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending/in_progress/complete

    -- Parameter snapshot (for reproducibility)
    base_parameters JSON,      -- θ values before perturbation
    plus_parameters JSON,      -- θ + δ values
    minus_parameters JSON,     -- θ - δ values
    perturbation_signs JSON,   -- +1 or -1 for each parameter

    -- Results (filled when complete)
    gradient_estimate JSON,    -- Calculated gradient
    elo_diff REAL,            -- Plus vs minus Elo difference

    -- Timestamps
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);
```

### Worker Mode Flow

When `compete --spsa -c 8` runs:

```python
while True:
    # 1. Find work
    iteration = db.query("""
        SELECT * FROM spsa_iteration
        WHERE status IN ('pending', 'in_progress')
        AND games_played < target_games
        ORDER BY iteration_number DESC
        LIMIT 1
    """)

    if not iteration:
        sleep(10)  # No work, wait and retry
        continue

    # 2. Mark as in_progress if pending
    if iteration.status == 'pending':
        db.execute("UPDATE spsa_iteration SET status='in_progress' WHERE id=?", iteration.id)

    # 3. Play a batch of games
    plus_wins, minus_wins, draws = 0, 0, 0

    for _ in range(batch_size):  # e.g., 10 games per batch
        # Random opening
        opening_fen, opening_name = random.choice(OPENING_BOOK)

        # Random time within range
        time = random.uniform(iteration.timelow_ms, iteration.timehigh_ms) / 1000

        # Alternate colors
        if random.choice([True, False]):
            result = play_game(iteration.plus_engine_path, iteration.minus_engine_path, time, opening_fen)
            if result == "1-0": plus_wins += 1
            elif result == "0-1": minus_wins += 1
            else: draws += 1
        else:
            result = play_game(iteration.minus_engine_path, iteration.plus_engine_path, time, opening_fen)
            if result == "1-0": minus_wins += 1
            elif result == "0-1": plus_wins += 1
            else: draws += 1

    # 4. Update results atomically (no individual game saves!)
    db.execute("""
        UPDATE spsa_iteration
        SET games_played = games_played + ?,
            plus_wins = plus_wins + ?,
            minus_wins = minus_wins + ?,
            draws = draws + ?
        WHERE id = ?
    """, batch_size, plus_wins, minus_wins, draws, iteration.id)

    # 5. Continue (master will mark complete when target reached)
```

### Master Mode Flow

```python
while iteration_number < max_iterations:
    # 1. Generate perturbed parameters
    base_params = read_params_toml()
    signs = {p: random.choice([-1, +1]) for p in base_params}

    plus_params = {p: v + signs[p] * delta[p] for p, v in base_params.items()}
    minus_params = {p: v - signs[p] * delta[p] for p, v in base_params.items()}

    # 2. Build engines (in rusty-rival repo)
    build_engine(plus_params, "engines/spsa-plus")
    build_engine(minus_params, "engines/spsa-minus")

    # 3. Create iteration record
    db.execute("""
        INSERT INTO spsa_iteration (
            iteration_number, plus_engine_path, minus_engine_path,
            timelow_ms, timehigh_ms, target_games,
            base_parameters, plus_parameters, minus_parameters, perturbation_signs
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, iteration_number, plus_path, minus_path,
         config.timelow_ms, config.timehigh_ms, config.games_per_iteration,
         json.dumps(base_params), json.dumps(plus_params), json.dumps(minus_params), json.dumps(signs))

    # 4. Wait for workers to complete games
    while True:
        row = db.query("SELECT * FROM spsa_iteration WHERE iteration_number = ?", iteration_number)
        if row.games_played >= row.target_games:
            break
        print(f"Iteration {iteration_number}: {row.games_played}/{row.target_games} games")
        sleep(30)

    # 5. Calculate gradient and update parameters
    score = (row.plus_wins + row.draws * 0.5) / row.games_played
    elo_diff = -400 * log10(1/score - 1) if 0.01 < score < 0.99 else 0

    gradient = {}
    for param, sign in signs.items():
        gradient[param] = sign * elo_diff / (2 * delta[param])

    # 6. Update parameters: θ_new = θ_old + a_k * gradient
    a_k = a / (iteration_number + A) ** alpha  # Decreasing step size
    new_params = {p: base_params[p] + a_k * gradient[p] for p in base_params}

    write_params_toml(new_params)

    # 7. Mark complete and continue
    db.execute("""
        UPDATE spsa_iteration
        SET status='complete', gradient_estimate=?, elo_diff=?, completed_at=NOW()
        WHERE id=?
    """, json.dumps(gradient), elo_diff, row.id)

    iteration_number += 1
```

### Master Configuration File

```toml
# spsa/config.toml

[time_control]
timelow = 0.25       # seconds (consistent with compete CLI naming)
timehigh = 1.0       # seconds

[games]
games_per_iteration = 150
batch_size = 10      # games per worker batch

[spsa]
max_iterations = 500
a = 1.0              # Step size numerator
c = 0.5              # Perturbation size base
A = 50               # Stability constant
alpha = 0.602        # Step size decay exponent
gamma = 0.101        # Perturbation decay exponent

[build]
rusty_rival_path = "../rusty-rival"
engines_path = "../chess-compete/engines"
```

### File Structure

```
rusty-rival/
├── spsa/
│   ├── __init__.py
│   ├── master.py          # SPSA master controller
│   ├── build.py           # Engine building (modifies engine_constants.rs)
│   ├── params.toml        # Current parameter values (updated each iteration)
│   └── config.toml        # SPSA hyperparameters, time controls
├── src/
│   └── engine_constants.rs  # Modified by build.py
└── ...

chess-compete/
├── compete/
│   ├── spsa/
│   │   ├── __init__.py
│   │   └── worker.py      # Worker mode implementation
│   └── cli.py             # Add --spsa flag
├── engines/
│   ├── spsa-plus/         # Built by master, used by workers
│   │   └── rusty-rival
│   └── spsa-minus/
│       └── rusty-rival
└── ...
```

## Recommended Approach

### Phase 1: Infrastructure (COMPLETED)
1. ✅ Add parallel game support to chess-compete
2. ✅ Verify NPS consistency across parallel games
3. ✅ Establish baseline game throughput

### Phase 2: SPSA Infrastructure
1. Create `spsa_iteration` database table
2. Implement parameter definition (`params.toml`)
3. Implement engine building with parameter injection
4. Implement worker mode (`compete --spsa`)
5. Implement master controller

### Phase 3: Focused Tuning
1. **Start small**: 5-10 most impactful parameters
   - Fewer parameters = fewer games needed for convergence
   - Easier to validate results
   - Recommended initial set:
     - LMR coefficients (2 params)
     - Futility margins (2-3 params)
     - Null move depth (1-2 params)
     - SEE pruning (1-2 params)
2. Run ~50,000 games across distributed workers
3. Validate improvement with longer time control matches

### Phase 4: Expand and Iterate
1. Add more parameters incrementally
2. Re-tune periodically as engine changes
3. Consider evaluation parameters

### Phase 5: Revisit Failed Techniques
1. Re-add razoring with SPSA-tuned margins
2. Re-add singular extensions with tuned thresholds
3. Test each addition independently

## References

- [SPSA Algorithm](https://www.jhuapl.edu/SPSA/)
- [Chess Programming Wiki - Automated Tuning](https://www.chessprogramming.org/Automated_Tuning)
- [Stockfish SPSA Tuning](https://github.com/official-stockfish/Stockfish/wiki/Regression-Tests)
- [OpenBench](https://github.com/AndyGrant/OpenBench)

## Related Issues

- Issue #44: SPSA tuning implementation
- See also: `docs/search.md` - Failed Improvements Log
