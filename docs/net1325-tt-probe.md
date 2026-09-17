# NET-1325: reuse TT probe words for replacement metadata

Baseline v1.0.67 (`29129306325f9b23c169382c66419bd1b20d6645`).
[Registered experiment](https://linear.app/netsensia/issue/NET-1325).

## Change

Main search called `probe` and then `entry_meta` for the same slot, loading
three relaxed atomic words twice. Disassembly confirms both load sequences.
`probe_with_meta` returns the checked entry and advisory replacement metadata
from one set of three loads. Search consumes both; existing probe callers retain
the original API. The 24-byte entry, full checksum key, generation width, score
encoding and replacement rules stay the same.

This is not an atomic snapshot: concurrent writers can still interleave with
loads. Only a checksum-accepted entry supplies scores/moves/evaluation. Metadata
remains advisory on misses or checksum failure, just as before. SMP observations
can differ because the former second read could see a later writer.

## Results

| Build/workload | Throughput change | Paired 95% interval | Gate |
| --- | ---: | ---: | --- |
| Pipeline AVX2, one thread | +0.73% | +0.60% to +0.87% | Pass |
| Native, independent confirmation | +0.92% | +0.75% to +1.08% | Pass |
| AVX2, 16-thread fixed time | +0.82% | +0.39% to +1.26% | Pass |

234 release tests pass, 3 ignored; formatting and workspace Clippy pass.
Focused tests cover empty, matching, foreign and each individually corrupted
entry word, signed scores, all bound types and full-width generations.
Both single-thread builds match all 80 valid distinct 100k-node search traces
and every bench per-position node count (total 1,772,650) in every run.

## Method and limits

Ryzen 9 5950X, rustc 1.98.1. Independent target directories and binary hashes.
AVX2 RUSTFLAGS: `-C link-args=-Wl,-z,stack-size=8388608 -C target-cpu=x86-64-v3`;
native uses repository `.cargo/config.toml`. One warmup per engine, 20 alternating
ABBA/BAAB blocks, CPU2, Threads1, Hash128MB. Student-t(19) interval on block
log ratios. Registered single-thread gate: >=0.5% and positive lower bound.

The separate SMP check uses physical CPUs0..15, Threads16, Hash128MB, two
non-mating positions, 2000ms/search, fresh engine per run, 20 ABBA/BAAB blocks.
Block ratios use aggregate final-reported nodes per reported search time.
All bestmoves must be legal and searches complete within the registered time
bounds. Registered SMP gate: nonnegative point estimate and lower 95% bound
above -1%. Search trees are not expected identical under SMP. This is a small
workload and Hash128MB; it does not prove behavior at every hash size or Elo.
No exclusions, tuning, pooling or early stopping; CPU samples retained.

NET-1324 exploratory profiling motivated this candidate. No claim that removing
loads must improve speed: repeated reads can hit cache. Neither the profiles nor
these throughput measurements establish an Elo gain.

Reproduce single-thread runs with `scripts/compare_search_speed.py BASE CAND
--output RESULT.json`; reserve CPUs and run builds before timing. Raw evidence,
SMP/monitor scripts, patch, disassembly, build/test logs and compiler hashes are
attached to NET-1325. `results/performance/net1325.json` contains compact records:
trace SHA256 hashes use Python `json.dumps(trace, sort_keys=True).encode()`;
raw hashes cover exact archived JSON bytes. CPU reservation came only after
explicit idle/no-process confirmation; bot remained stopped throughout.

## Provenance and observed SMP records (review clarification)

The AVX2 and SMP baseline intentionally reuse the **v1.0.67 baseline** built for
NET-1322, not that experiment's rejected candidate. Every tracked baseline-source
file was checked against commit 2912930's Git blob IDs. The source tree ID,
compiler, exact flags/build command, build-log digest and candidate commit are
recorded in `build_provenance`; executable SHA256 values remain in each run.
The native baseline was independently built from the same verified source.
No binary, timing sample, or engine code changed during this clarification.

Each compact SMP run now includes observed aggregate `nodes` and `time_ms`, plus
per-position `searches` with node count, reported search time, wall-clock duration
and final `bestmove`. `position_index` selects the FEN from `runs.smp.fens`.
`nps` is recomputable as 1000 * aggregate nodes / aggregate reported time_ms;
`movetime_ms` remains the requested limit, not measured elapsed time. Full UCI
lines and CPU samples remain in the NET-1325 raw evidence attachment.
