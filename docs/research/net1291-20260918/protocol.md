# EPD resource investigation, 18 September 2026

Diagnosis only. No production engine source changes or parameter selection. PGO work remains preserved on perf/linux-avx2-pgo and is not an arm in this comparison.

## Baselines
- Public v1.0.68 AVX2, SHA256 fb4c0ef1d088a2d25abdf890acc277cafbb9cb30c90c999e200e3131e29b748b.
- Stash source 13e0a81a4af7fb6af43405228535c5058ee71657, default release build.
- Ethereal source 0e47e9b67f345c75eb965d9fb3e2493b6a11d09a, public HCE build, GCC, default PGO make target.
- Stockfish 19 public Linux universal package, source included, for confirmation only.

## Comparison
Full 100-position EET at 1 and 5 seconds, fresh process per position, Threads1, requested Hash128, concurrency4, same host/default affinity. Tablebases unset. New binaries have new records, never overwrite historical records. No builds during runs. Host desktop activity remains possible: these are coarse diagnostic screens, NOT tightly controlled speed or strength measurements. Compare differing moves at longer and fixed-node budgets before causal conclusions. NPS, depth and cp scales are not interchangeable across engines.

Hash128 actually allocates96MiB in Rival. Report equal requested budget as deployed behavior; isolate actual hash-capacity sensitivity before claiming an evaluation cause.

## Truth and diagnosis
Audit all EET <=7-piece roots with exact tablebase results, recording full responses and expected moves. Other EPD best moves and peer majority are hypotheses. A tablebase reached along one PV proves that leaf/continuation only, not root minimax. Forced-root moves use the same reference engine and equal budget, with fresh game/TT state. Preselected historical exemplars before 5-second results: EET049,060,081 (defensive king move, attacking king route, quiet rook retreat). Archive root and forced PVs, then identify earliest concrete divergence. Long reference searches remain fallible.

## Generalisation and acceptance
Do not inspect the previously reserved100-position NET1291 holdout during design. No constants selected from EPD counts. Any intervention first needs a general mechanism expressed without suite IDs, then independent game-derived diagnosis positions, deterministic controls, all-six-suite gain AND loss lists, untouched confirmation set and paired game testing at short and longer controls. Trading one EPD success for another failure is not accepted evidence of progress. Correctness bugs and strength hypotheses reported separately. No Elo inferred from suite score.

## History correction
NET1291 arms A-D/pilot D nonpassing; no repeat proposed. Root fail-high retention eventually shipped v1.0.65, commit530f8a5, after later recorded [0,+5] match6266 games and scaling check. Linear description ends at earlier inconclusive run; comments/commit supersede it. The broader rook gap remains unresolved.
