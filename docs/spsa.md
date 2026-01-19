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

## SPSA Tooling Location

**All SPSA tooling is now consolidated in the chess-compete repository.** This includes:

- Master controller
- Worker implementation
- Build scripts
- Configuration files
- Parameter definitions

See: `chess-compete/compete/spsa/`

The rusty-rival path is configurable in `chess-compete/compete/spsa/config.toml` (default: `../rusty-rival`).

## Quick Start

```bash
# From chess-compete directory

# Start the master controller (generates iterations, monitors progress)
python -m compete.spsa.master

# Start workers on same or different machines (polls DB, builds engines, runs games)
python -m compete --spsa -c 6

# Workers automatically:
# - Poll database for pending iterations
# - Build engine binaries from parameters stored in DB
# - Run games between plus/minus perturbed versions
# - Report aggregate results
```

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

### Consolidated in chess-compete

All SPSA tooling lives in chess-compete repository:

```
chess-compete/
├── compete/
│   └── spsa/
│       ├── __init__.py       # Package exports
│       ├── master.py         # SPSA orchestration (iterations, gradients)
│       ├── worker.py         # Worker mode (builds, runs games)
│       ├── build.py          # Engine building with parameter injection
│       ├── config.toml       # SPSA hyperparameters, time control, paths
│       └── params.toml       # Current parameter values being tuned
├── engines/
│   └── spsa/                 # Built engines (spsa-plus, spsa-minus)
└── migrations/
    └── 002_spsa_iterations.sql
```

### Key Design Decisions

1. **No individual game saves** - SPSA mode does NOT write to the `games` table
   - Only aggregate results (wins/losses/draws) stored in `spsa_iterations`
   - Avoids polluting database with thousands of test games
   - Faster execution (no per-game DB writes)

2. **No engine registration** - SPSA engines are ephemeral
   - No `--init` needed, engines not added to `engines` table
   - Binaries loaded directly from path
   - Overwritten each iteration

3. **Workers build their own engines** - supports distributed cross-platform deployment
   - Parameters stored in database (`plus_parameters`, `minus_parameters`)
   - Each worker builds locally if engines don't exist
   - Works across different OSes (Windows, Linux, Mac)

4. **Random openings** - each game uses a random opening from `OPENING_BOOK`
   - Prevents overfitting to specific positions
   - Same book used by other compete modes

5. **Time variety** - use `timelow`/`timehigh` for random time per game
   - Consistent naming with other compete modes
   - Helps ensure parameters work across time controls

### Database Schema: spsa_iterations

```sql
CREATE TABLE spsa_iterations (
    id SERIAL PRIMARY KEY,
    iteration_number INTEGER NOT NULL,

    -- Engine binaries (paths, may be built by workers)
    plus_engine_path VARCHAR(500) NOT NULL,
    minus_engine_path VARCHAR(500) NOT NULL,

    -- Time control (consistent with other compete modes)
    timelow_ms INTEGER NOT NULL,
    timehigh_ms INTEGER NOT NULL,

    -- Game tracking
    target_games INTEGER NOT NULL DEFAULT 150,
    games_played INTEGER NOT NULL DEFAULT 0,
    plus_wins INTEGER NOT NULL DEFAULT 0,
    minus_wins INTEGER NOT NULL DEFAULT 0,
    draws INTEGER NOT NULL DEFAULT 0,

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- Parameter snapshot (for reproducibility and worker builds)
    base_parameters JSONB,
    plus_parameters JSONB,
    minus_parameters JSONB,
    perturbation_signs JSONB,

    -- Results (filled when complete)
    gradient_estimate JSONB,
    elo_diff NUMERIC(7, 2),

    -- Timestamps
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);
```

### Configuration

**config.toml** - SPSA hyperparameters and settings:

```toml
[time_control]
timelow = 0.25       # seconds
timehigh = 1.0       # seconds

[games]
games_per_iteration = 150
batch_size = 10

[spsa]
max_iterations = 500
a = 1.0              # Step size numerator
c = 1.0              # Perturbation size multiplier
A = 50               # Stability constant
alpha = 0.602        # Step size decay exponent
gamma = 0.101        # Perturbation decay exponent

[build]
rusty_rival_path = "../rusty-rival"   # Configurable path to rusty-rival source
engines_output_path = "engines/spsa"

[database]
poll_interval_seconds = 30
```

**params.toml** - Current parameter values:

```toml
[beta_prune_margin_per_depth]
value = 200.0
min = 50
max = 400
step = 20

[null_move_reduce_depth_base]
value = 3.0
min = 2
max = 5
step = 1

# ... more parameters
```

## Recommended Approach

### Phase 1: Infrastructure (COMPLETED)
1. ✅ Add parallel game support to chess-compete
2. ✅ Verify NPS consistency across parallel games
3. ✅ Establish baseline game throughput

### Phase 2: SPSA Infrastructure (COMPLETED)
1. ✅ Create `spsa_iterations` database table
2. ✅ Implement parameter definition (`params.toml`)
3. ✅ Implement engine building with parameter injection
4. ✅ Implement worker mode (`compete --spsa`)
5. ✅ Implement master controller
6. ✅ Add distributed worker support (workers build locally)

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
