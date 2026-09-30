# v1.0.72 — exact move-picker speedup

Research performed 29 September 2026; release prepared 30 September 2026.

[Notion research write-up](https://www.notion.so/3ebcc6c04bf5813da51cf1421cc7871f)

v1.0.72 changes only the move-selection implementation and its differential regression test, plus version and documentation, relative to v1.0.71. Net, evaluation scale, search parameters and move ordering remain unchanged. At a clock limit, faster search may naturally reach deeper and select a different move; fixed-work behavior is preserved.

## Confirmed candidate

Commit `7b58dd3317f93f1f3e2acac04450002cc1ac85cd`, branch `perf/v171-packed-pick`, based on the exact v1.0.71 source (`b0b085c5…`). Only `src/search.rs` changes: the move-selection scan and its differential regression test. The net, search parameters, move scores and tie order are unchanged.

The candidate packs each signed score and reversed original index into one 64-bit key. Taking the maximum preserves the first-highest-score choice and the exact `swap_remove` remainder order. This gives the compiler a different reduction to optimize inside search. There is no unsafe code in this change.

## Fresh confirmation measurements

| Build and workload | Paired NPS gain | Descriptive block t interval |
|---|---:|---:|
| Native non-PGO, bench depth 20, 8 ABBA blocks | +3.46% | +2.71% to +4.22% |
| x86-64-v3 matched PGO, bench depth 20, 8 blocks | +6.47% | +5.85% to +7.11% |
| Matched PGO, 40 held-out positions at depth 16, 4 blocks | +5.34% | +2.83% to +7.91% |
| x86-64-v2 non-PGO, bench depth 16, 8 blocks | +3.79% | +2.62% to +4.97% |

These are throughput gains (`baseline time / candidate time - 1`), not percentages of elapsed time saved. The PGO bench result corresponds to 6.08% less elapsed time. Intervals describe variation among a small number of shared-host blocks; they are not a guarantee across machines or workloads. Every complete measured block was retained.

## Exactness and checks

- Every depth-20 bench run: identical per-position counts, totaling **55,151,845 nodes**. Depth-16 portable runs: **10,766,745 nodes**.
- Held-out: all 40 positions have identical nodes, scores/bounds, PVs and best moves across 18 runs; **29,231,270 nodes** per complete position set.
- Additional 80 positions at one million nodes each: exact scores/PVs/best moves. Fixed-node totals are a budget by construction, not a separate proof of identical trees. These are PGO training positions and are used only for identity, not timing.
- Full release workspace tests: **273 passed, 0 failed, 3 pre-existing ignored**, no newly filtered test. Current-CI Clippy, formatting and cargo check passed. **17 Python tool tests** passed.
- Differential move-selection test: all lengths 1–256, ties, signed extremes and random scores; every selected move and every remaining array compared to the old implementation. The same test also passed in debug mode.
- Portable baseline/candidate library and NNUE tests passed.
- Claude independently reviewed the ordering proof and the confirmation arithmetic.

## Interpretation and limits

One Ryzen 9 5950X, CPU 6 pinned, one engine thread, Hash 128; its SMT sibling was not reserved. Benchmark processes have no Syzygy path configured. Other product work was allowed. All timed runs were serial and builds occurred between runs. Native and portable builds are non-PGO; release-style PGO builds were independently trained on the same frozen 80 positions. Results do not establish multi-thread speed, other-CPU speed or Elo.

The held-out set includes 23 STS, 8 WAC, 8 low-material calibration and 1 Arasan position. All group point estimates were positive, but small groups are diagnostic only. The 25 ms Arasan case is quantization-limited. A slow final WAC baseline run increased the last held-out block; it was retained.

The isolated non-inlined picker benchmark over sampled real lists showed only about +1.2% for packed keys, with mixed block signs. It does not reproduce the full-search gain, and is not proof of its mechanism. Build/inlining/branch context matters; the whole-search measurements are the primary evidence.


## Equivalence argument


For score s (signed32-bit), current array index i (0 <= i < MAX_MOVES=256), define K=s*2^32+(2^32-1-i). K fits signed64-bit even for i32::MIN/MAX. The implementation uses a left shift and bitwise OR to build exactly this value.

A score increase of one outweighs every possible index difference. For equal scores, a smaller current index gives a larger K. Therefore maximizing K selects exactly the first maximum used by the original strict-greater scan. The low32bits equal u32::MAX-i, so the inverse subtraction recovers precisely that index. Applying the unchanged swap_remove preserves the entire remaining sequence, including future tie-breaking.

No search parameters, score arithmetic, move generation, net, evaluation scale, node accounting, transposition-table contents, or public move-array representation change. No unsafe code or target-specific intrinsics are introduced. LLVM can lower the reduction differently for each target; speed claims apply only to measured builds and hardware.

Tests compare every pick and remaining list against the original scan for all lengths1..256, all-equal, ascending, descending, narrow tied random, alternating minimum/maximum, and full-width pseudo-random scores. This tests examples of the mathematical equivalence; it is not a proof over all possible lists by enumeration.

## Fresh-build repeat and rejected refinements

A second fresh matched PGO build pair reproduced **+6.44%** on the depth-20 bench (four ABBA blocks, descriptive t interval +5.17% to +7.73%) and **+4.84%** on the 40-position held-out set (+4.04% to +5.64%). Counts, scores, PVs and best moves remained exact. Neither prior result was replaced.

Twenty-six isolated variants were screened across move generation/attack queries, NNUE, SEE, move selection, prefetch and hashing. Most were flat or slower. One legality shortcut failed correctness before timing and was rejected. A contaminated SoA timing attempt was retained and wholly repeated; the clean repeat regressed 6.93%.

A short-list scalar special case (+0.82% bench, +0.11% held-out versus packed) remains unconfirmed. A four-lane packed refinement (+0.90%/+0.75%) passed correctness checks but is not included. A same-source old-versus-rebuilt packed comparison varied by −2.21% and −0.52% in its two blocks; that observation combines build/layout and host timing effects, and does not estimate general rebuild variance. The small refinement needs another matched-build comparison. Percentages from these comparisons are not added to the headline gain.

## Reproducible evidence

The [evidence directory](evidence/raw-speed-v1.0.72/) contains all warmup and measured rows for the six main comparisons, per-position node/result signatures, binary hashes, exact report values and an export manifest binding the original local JSON hashes. No timing rows were excluded. The reports retain their original approximate normal intervals; the table above uses small-sample t intervals descriptively.

Local bench invocation: `taskset -c 6 ENGINE bench depth 20` with Hash 128 configured through UCI as in the original harness. Native and PGO timings use 16 bench positions. Held-out runs use one-thread fresh-state UCI searches at depth 16; each timing pair runs A, B, B, A after warmups. PGO uses the existing `scripts/build_pgo.py` and frozen `scripts/pgo/positions.json`. Hardware/build limits above apply.

This is a speed release, not a new strength experiment or an Elo claim.
