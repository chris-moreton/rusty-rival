# NET-1291 arm E: retain a completed root fail-high on interruption

**Status: candidate validated and frozen; strength test pending.**

The [round 3 trace](net1291-round3-search-trace.md) showed Rival completing a
root fail-high for winning Kb3, then discarding it when its wider aspiration
retry was interrupted. Arm E tests a general response: retain a completed
root fail-high above the previous completed iteration's score if its retry
is interrupted. It has no position, material, move, depth or node selector.

A retained result is scoped to the current iteration and invalidated by a
later completed fail-low. The last complete iteration's exact state remains
separate. UCI reports the chosen result as `lowerbound` at its actual depth,
with that result's PV and ponder move. MultiPV is unchanged. The search tree,
pruning, evaluation and time budgets are unchanged; copying a root PV adds
some unmeasured overhead, which the game test includes.

Internal diagnostics/data generation that deliberately consume
`current_best` still receive the previous exact completed iteration. The new
move is exposed by `iterative_deepening`'s return value and the UCI result;
no lower bound is substituted for an exact training label.

## Frozen candidate and checks

- Parent: `da0baba`; candidate source and tests are committed with this report.
- Binary: `/home/chris/benchmark/sprt/rival-net1291E`.
- SHA256: `a3f61c54ae4dd0d5763ba0cf11560a9a3f4eac277889985d5663ae63ce460e4f`.
- Baseline: production `rival-v1.0.64`, SHA256
  `1c66b5718ef2e25b9b565003fc73267ec96d9a2910f383dd3a943cc2a139237f`.
- Bench: **1,772,650**, identical to baseline. This is a depth-limited bench;
  identical search work does not mean identical choices on interruption.
- Release tests: **230 passed, 3 ignored**. New tests cover retention without
  overwriting exact state, supersession by a completed iteration, reset at a
  terminal next root, and selecting the retained result's own ponder move.
- Required workspace Clippy, new integration-test Clippy, formatting and
  whitespace checks pass. An additional all-target Clippy check found old
  warnings in existing tests (constant/boolean assertions and other test
  style lints); those failures are archived. The new unit-test module was
  moved to the end of its file and its test rerun successfully.

Fresh FEN, one thread, Hash128:

| Check | Baseline | E |
| --- | --- | --- |
| R2-093 critical root, 1m nodes | Kb5 / +36 / completed d16 | Kb3 / +111 **lower bound**, d17 |
| Same root, 300k nodes | Kb5 / +36 / d13 | Identical |
| Same root, completed depth17 | Kb3 / +134 / 1,175,103 nodes | Identical |
| Same root, MultiPV2, 1m nodes | Kb5 / +61 / d15 | Identical |

All unchanged checks compare move/ponder, score, depth, seldepth, node count,
hashfull and PV, excluding time/NPS. The one-million-node result's final info
PV begins `Kb3 Rg1 Ka2 Rg4 Rb5`; bestmove and ponder agree with it. This fixes
the diagnosed budget-boundary choice without changing the search path.

## Registered game protocol

One frozen arm versus v1.0.64: Protocol S [0,+10], alpha/beta .05, 1+0.01,
4,000-game cap, concurrency12, one thread, Hash128, paired 8moves_v3 openings,
16 opening plies, random opening order seed1291. Standard resign/draw
adjudication and 30ms time margin match the prior NET-1291 experiments.
The bot is stopped only between games, and restarted after the measurement
window. No build runs during the match.

All six suites follow at 300k nodes, concurrency4, under label `net1291E`.
A nonpassing arm does not advance to 10+0.1, ladder or the reserved 100 rook
endings. Those confirmation positions remain unused.

The idea and protocol were recorded in NET-1291 before implementation.
NET-1149's rejected-experiment register and NET-1244's output bookkeeping
history were checked; no prior test of this policy was located. This does
not retry the previous nonpassing draw, TT-depth or training arms.

## Evidence

[Candidate patch](net1291/round4/candidate.patch),
[integration tests](net1291/round4/search_root_fallback.rs.txt),
[binary/source identity and UCI checks](net1291/round4/validation.json),
[validation runner](net1291/round4/validate.py.txt),
[match and suite command](net1291/round4/run-games.sh.txt),
[full release tests](net1291/round4/full-tests.log), and
[required Clippy](net1291/round4/clippy-required.log).

The archived scripts retain this investigation's local paths. The patch
covers tracked production source (including its unit test); add the separately
archived integration-test file when reconstructing the candidate. The
validation script refuses to overwrite a previously frozen match binary.
