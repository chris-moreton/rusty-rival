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
version string is a different engine.

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

## Suites

`epd/suites/*.epd`; sources and terms in `epd/suites/NOTICE`. A line is four
FEN fields (a full FEN is tolerated) followed by `;`-separated opcodes. `bm`
and `am` are SAN lists resolved against the engine crate's move generator;
STS grades are read from `c0` ("Rd4=10, Bd5=8") or the `c7`/`c8` pair. The
test `tests/suites.rs` resolves every target of every shipped suite and
cross-checks the STS moves against their `c9` UCI form.
