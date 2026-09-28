# Rusty Rival v1.0.71

Pruning-margin calibration improves search with the v1.0.70 NNUE network and evaluation scale unchanged. Selected margins: reverse futility113cp/depth, alpha base81 and per-depth44, depth-one razor230, delta200. Sparse-null experiment excluded.

## Validation

- Independent matched non-PGO validation:6400 games at10+0.1,+6.41 Elo,95%CI[+0.75,+12.07].
- Production PGO qualification vs publicv1.0.70:SPRT H0=0/H1=5,alpha=beta0.05 acceptedH1 after5245 completed games (2622 complete pairs; one singleton preserved). Paired estimate+9.54 Elo; stopped-sample interval descriptive only.
- Separate1600-game10+0.1 confirmation:+14.99 Elo,95%CI[+4.25,+25.76],passes registered positive-point/lower>-5 gate.

These are separate results, not pooled, and do not predict an exact Lichess rating change. Colour-paired openings, fixed net/scale, no adjudication; one selected candidate, no retries/extensions. Alpha is per-test, not campaign-wide over adaptive research. Six-suite diagnostics include gains and losses; no Elo inference from EPD.

All272 Rust tests passed without exclusions (3 existing ignored), plus Python tools, clippy, compatibility and80-position identity. The root-fallback test retains all assertions with a measured570k node budget for this search tree. Versioned/public artifact checks follow before deployment.
