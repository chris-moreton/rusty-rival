# epd-runner

Runs EPD test suites against any UCI engine, keeps every result in a JSON
store keyed by the exact engine binary, and prints suites × engines tables.
It is a regression check and a comparison view, not a strength measure:
games decide strength (see the Search Efficiency Audit page).

```
cargo build --release -p epd-runner
target/release/epd-runner suites
target/release/epd-runner run --engine target/release/rusty-rival --suites all --nodes 300000
target/release/epd-runner run --name sf-2800 --suites arasan18,wac --time 1
target/release/epd-runner table --nodes 300000 --engines rusty-rival:1.0.63,rusty-rival:1.0.64,ethereal
target/release/epd-runner run --engine BIN --suites quick --nodes 100000 --json
```

## Modes

* `--nodes N`: reproducible for an engine whose search is deterministic at
  the options used (one thread; rusty-rival is), so a second run is a cache
  hit and two binaries with the same behaviour produce identical records.
  An engine that is not deterministic at a node limit (several threads, or
  a search that depends on wall time) can differ between runs while the
  store still reports a cache hit. The regression mode.
* `--time S`: seconds per position. What the published suites are calibrated
  for, and only meaningful on an idle machine: the run refuses to start while
  a lichess-bot engine is running or the one-minute load is above 4 unless
  `--allow-busy` is given. Concurrency defaults to 1.
* `--depth D`.

One fresh engine process per position, `Threads` and `Hash` from the run
arguments (defaults 1 and 128), extra UCI options from the registry entry or
`--option NAME=VALUE`.

## Engines

`--engine PATH` reads the engine's identity from its UCI `id name` line:
`Rusty Rival 1.0.64` is stored as family `rusty-rival`, label `1.0.64`.
`--name NAME` takes an entry from `epd/engines.toml`, whose `name` becomes the
label (for example `sf-2800`, Stockfish with `UCI_Elo` 2800). The binary's
sha256 is always part of the record, so a rebuilt binary with the same
version string is a different engine, and the UCI options are part of it
too: the same binary with other options is another column, shown as
`sha8#opthash` under the header. A selector takes `#hash` suffixes that all
have to match the binary sha8 or the options hash (`-` for no options), so
`family:label#sha8#opthash` names one binary with one option set.

## Store

`epd/results/<family>/<label>-<sha8>.json`, one file per engine binary:

```
{ "engine": { "name": "Rusty Rival 1.0.64", "family": "rusty-rival", "label": "1.0.64",
              "version": "1.0.64", "sha256": "...", "options": {}, "bench": 1772650 },
  "runs": [ { "suite": { "name": "arasan18", "sha256": "...", "positions": 250 },
              "mode": "nodes", "budget": 300000, "threads": 1, "hash_mb": 128,
              "host": { "hostname": "...", "cpu": "..." }, "date": "2026-09-11T20:00:00Z",
              "summary": { "solved": 58, "total": 246, "median_solve_nodes": 41000,
                           "mean_depth": 11.2, "nps": 1900000, "errors": 0 },
              "positions": [ { "id": "arasan18.1", "bm": ["g4"], "best": "g2g4", "solved": true,
                               "solved_at": { "depth": 9, "nodes": 12345, "ms": 8 },
                               "score_cp": 35, "depth": 14, "nodes": 300012, "ms": 160 } ] } ] }
```

The cache key of a run is (binary sha256, suite sha256, mode, budget,
threads, hash MB) and, in time mode, the host CPU model. `run` skips a suite
whose key is already in the store unless `--force`, which replaces it.
`solved_at` is the first `info` line at which the correct move became the
PV move and stayed there until `bestmove`. Graded suites (STS) also record
`points` per position and `points`/`max_points` in the summary; `total`
counts positions without a resolution error.

Records for released rusty-rival versions and for the peers are committed;
experiment binaries can be left untracked.

## Terminal view

```
target/release/epd-runner tui
```

Rows are suites, columns the selected engines, cells the percent solved
(or counts with `p`) at the current budget; the best and worst cell in a
row are green and red, `—` is a missing run. `b`/`B` cycle the budgets in
the store for the current mode, `m` moves to the next mode, `e` opens the
engine picker (space toggles one engine, `f` a whole family, `a`/`n`
all/none), Enter opens the suite with one row per position and every
engine's move and solve point (`d` keeps only the positions the engines
disagree on), and wide tables scroll sideways with the cursor. `r` queues
`epd-runner run` for the missing cell under the cursor and `R` for every
missing cell in the column; runs go one at a time, only for engines whose
`engines.toml` entry is the same binary and options as the column, on this
CPU for time budgets and for the suite revision on disk, and time-mode
runs ask first. The selection, the percent switch and the budget persist in
`epd/tui-state.toml`.

## Diff and check

```
epd-runner diff rusty-rival:1.0.63 rusty-rival:1.0.64 --suites arasan18 --nodes 300000
epd-runner diff target/release/rusty-rival ~/benchmark/sprt/rival-v1.0.64 --suites all --nodes 300000
epd-runner check --engine target/release/rusty-rival --baseline rusty-rival:1.0.64 \
    --suites bratko-kopec,wac --nodes 100000 --max-drop 2
epd-runner check --engine BIN --baseline epd/results/rusty-rival/1.0.64-<sha8>.json --suites quick --nodes 100000 --exact
```

Each side of `diff` is an engine binary (run, or served from the cache),
`@name` for a registry entry, a results file, or a store selector
(`family:label[#hash]`); `--option NAME=VALUE` applies to any side that is
run. `check` takes the candidate as `--engine PATH` or `--name NAME` and the
baseline in the same forms as a `diff` side. The output lists,
per suite, the positions solved by one side and not the other with each
side's move and solve point, the two summary lines and the net change;
`--json` gives the same as data. `check` runs the candidate binary against a
baseline and exits 1 when any suite's solved count fell by more than
`--max-drop` (default 0), or, with `--exact`, when any position played a
different move or searched a different node count, which is the test for a
change that claims to be node-identical. A comparison with an errored or
missing position on either side always fails. Both compare only runs of the
same suite revision.

## Continuous integration

The `EPD regression` job in `.github/workflows/build.yml` builds the engine
and the runner, runs `check` on Bratko-Kopec and WAC at 100k nodes against
the committed rusty-rival 1.0.64 record, posts the delta as a comment on the
pull request (one comment, updated on each run), and fails when a suite's
solved count drops by more than two or when the comparison is incomplete (a
position that errored, or is missing on one side). A search change legitimately
moves these counts; a pull request that intends one updates the baseline by
running the suites for the new binary, as with the bench signature.

## Calibration (NET-1288, 12 September 2026)

Fourteen engines (rusty-rival 1.0.59 to 1.0.64, Ethereal 14.40, Stash
v37.25, Berserk 20260524, Obsidian dev-16.15, Stockfish dev-20260726 full and
capped at 2600, 2800 and 3000) ran every suite at 300k and 1M nodes and at
1 s per position (5 s on arasan18, bratko-kopec and eet), one thread, 128 MB,
time mode at concurrency 4 on an otherwise idle Ryzen 5950X with the bot
stopped. `scripts/calibrate.py` orders the engines per suite and budget and
counts violations of 22 orderings known from games: the Rival series where a
gain was measured (59 < 60 < 61 < 62 < 63), the capped Stockfish rungs,
every peer and sf-2800 or above over Rival, full Stockfish over all, Rival
over sf-2600.

| suite | 300k nodes | 1M nodes | 1 s | 5 s |
|---|---|---|---|---|
| eet (100) | 4 | 2 | **1** | **1** |
| arasan18 (250) | 5 | 4 | 3 | 3 |
| sts (1500) | 4 | 4 | 5 | – |
| bratko-kopec (24) | 8 | 6 | 5 | 8 |
| wac (300) | 10 | 13 | 9 | – |

Violations out of 22 pairs. What the numbers say:

- **Node budgets do not compare engine families.** Ethereal and Stash, far
  stronger than Rival in games, score below it at every node budget because
  their nodes are cheap; the NNUE-heavy Berserk and Obsidian sit next to full
  Stockfish. Node mode is a same-binary-family regression check and nothing
  else, which is what the CI gate uses it for.
- **Time budgets order the families roughly and the Stockfish rungs
  correctly** on eet and arasan18, and eet at 1 s or 5 s is the only
  suite-and-budget pair with a single violation, an adjacent Rival pair
  inside sampling noise. wac and bratko-kopec saturate above 90% for every
  strong engine and order nothing; sts orders the peers but places capped
  Stockfish below Rival at every budget.
- **No suite resolves the Rival release gains.** Six releases spanning about
  70 Elo at 10+0.1 land within one or two sigma of each other everywhere
  (arasan18's 250 positions give ±3 points; the whole series spans 5). Only
  arasan18 and eet at 5 s show the series trending upward. Suites of this
  size cannot rank changes worth 10 to 25 Elo; games do that.

Chosen gate: the CI job keeps bratko-kopec + wac at 100k nodes as a fast,
deterministic crash-and-blunder check with a threshold. For a look at where
an engine stands against the peers, read the time-mode eet and arasan18
columns, and treat everything else as informational.

## Suites

`epd/suites/*.epd`; sources and terms in `epd/suites/NOTICE`. A line is four
FEN fields (a full FEN is tolerated) followed by `;`-separated opcodes. `bm`
and `am` are SAN lists resolved against the engine crate's move generator;
STS grades are read from `c0` ("Rd4=10, Bd5=8") or the `c7`/`c8` pair. The
test `tests/suites.rs` resolves every target of every shipped suite and
cross-checks the STS moves against their `c9` UCI form.
