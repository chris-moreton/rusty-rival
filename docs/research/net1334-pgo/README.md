# NET-1334: Linux AVX2 PGO local acceptance

These are local build results, not a release-binary qualification or an Elo estimate.
The release candidate must independently regenerate its profile in CI and pass
public-binary validation before a final release is accepted.

## Results

All accepted runs completed their whole-run host CPU guard. Invalid attempts
were retained separately and never pooled, trimmed or used for speed inference.

| Gate | Result | 95% interval | Criterion |
|---|---:|---:|---|
| Primary, 20 paired blocks | +4.921% | +4.701% to +5.142% | >=3%, lower bound >0 |
| Held-out 40 positions, 20 paired blocks | +5.685% | +5.594% to +5.777% | >=3%, lower bound >0 |
| 16-thread baseline vs itself, 10 blocks | -0.111% | -0.699% to +0.479% | Contains0; half-width<=0.75 percentage points |
| 16-thread candidate, 20 blocks | +6.303% | +5.807% to +6.802% | Non-inferiority: lower bound >-1% |

The baseline-only half-width was0.589 percentage points. The candidate point
estimate is reported separately from its non-inferiority decision. Single-thread
and multi-thread results are not pooled or converted to Elo.

All80 primary normalized fixed-node search traces match, as do the40 held-out
traces throughout the paired runs. Bench remains1,772,650 nodes. The clean-tree
production builder smoke also passed80 identities against public v1.0.68 and the
same bench. Full release tests:234 passed,3 ignored;17 Python tests, formatting
and clippy passed after review fixes. No production search/evaluation code changed.

## Frozen design and scope

Source bd521c645f96a6638e8dbbac8ad2bb249e7d5e1e, pinned Rust1.98.1, Linux GNU
x86-64-v3, LTO and one codegen unit. Exact binary, compiler and profile identities
are in `summary.json` and the archived provenance.

Training is80 frozen positions,1M nodes each, one thread, Hash128, pondering off.
The40 held-out positions include8 low-material cases and are disjoint from training;
training also excludes all80 identity positions and all16 literal bench keys.
The selector, inputs, thresholds and hashes were frozen before candidate timing.

Single-thread timing uses CPU2 with sibling18 monitored. The 16-thread stage uses
physical cores0–15, Hash512 requested (384MiB actual with current TT rounding),
pondering off and eight baseline-qualified positions. Qualification required three
successful baseline trials per position, then chose the first two qualifying
low-material and first six other positions in frozen SHA order. Each search uses
2s movetime. Two warmups precede ABBA/BAAB blocks; there is2s idle after each arm's
eight searches, none between its searches. Rates pool total reported nodes over
total reported milliseconds. There is no deterministic multi-thread identity claim.
Ponder-on bot operation is outside this timing scope.

## Contamination and environmental controls

The unchanged guard uses a10s preflight and2s host-counter samples. Mean sibling
and other-CPU use must each be<=3%; peak sibling use<=20%, peak other-CPU use<=50%.
Any contaminated window invalidates the entire run. The monitor fails closed on
sampling errors. `/proc/stat` is host-wide; a namespace-local process scan is not
used as proof of idle CPUs.

Earlier attempts encountered desktop usage rescans, Chrome bursts, an accidental
diagnostic rustfmt run, scheduled database backup, JetBrains Toolbox and updatedb.
One SMP launch also failed before any engine ran because the system interpreter
lacked python-chess; the existing chess-test virtual environment fixed the launcher.
Failures and their monitor records are listed in `summary.json` and archived.
No threshold was relaxed and no bad block was dropped.

Usage-bar transcript rescans were deferred through their existing locks. During
accepted AA5 and AB6, Toolbox was temporarily paused and two Chrome application
scopes were limited to5% CPU each. For AB6 the hourly agent-farm snapshot timer was
also deferred. Each control had a restoration marker and automatic deadline.
After the local window, Chrome quotas were verified unlimited again, Toolbox was
running, the snapshot timer was active and scan locks were released. The bot
remained stopped on v1.0.68. These controls and restoration records are archived.

## Reproducibility

`study-evidence.tar.gz` contains the accepted raw records and CPU evidence,
invalid-run monitor records, frozen harnesses/selector, training/profile evidence,
canary evidence and clean production-builder verification. It contains no engines.
`checksums.json` lists the exact member bytes and archive checksum.

Archive recipe: members sorted by path, original bytes unchanged, USTAR regular
files with mode0644, uid/gid/mtime0 and empty owner names; gzip filename empty,
mtime0, compression level9. The manifest records zlib version. To verify content,
SHA-256 each extracted regular file and compare its path with the manifest.
Byte-identical recompression additionally depends on the recorded compression
implementation. No JSON reserialization or concatenated aggregate digest is used.
