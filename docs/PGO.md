# Linux AVX2 profile-guided builds

The Linux x86-64 AVX2 release build generates a fresh profile from the checked-out
source, then builds that source using the profile. Other release assets and
ordinary `cargo build` use their existing optimization path. The legacy Docker
build is outside this release-asset path.

## Build locally

Use a clean, committed checkout and its exact `rust-toolchain.toml` compiler.
Install the matching `llvm-tools` component, then run:

```sh
rustup component add llvm-tools
mkdir -p /tmp/rival-pgo/study
python3 scripts/build_pgo.py \
  --positions scripts/pgo/positions.json \
  --work-dir /tmp/rival-pgo-build \
  --lock-file /tmp/rival-pgo/study/timing.lock \
  --output /tmp/rusty-rival-pgo
```

The example lock matches this study’s monitor; create its parent directory first or select an existing shared lock path.

The work directory must be absent or empty. The builder refuses stale profiles,
a dirty source tree, a compiler different from the pin, enabled NET365
diagnostics, and profile warnings indicating mismatched control flow or missing
engine records. A nonblocking lock prevents concurrent invocations using the
same lock file. Experiment drivers must use the same lock for builds and timing;
pass `--lock-file` when coordinating with an existing experiment.

The build uses the Linux GNU target explicitly, x86-64-v3 and the existing 8 MB
stack linker flag. Rust profile flags apply only to target builds. Native
mimalloc and zstd code is not instrumented by these Rust flags.

Training uses the frozen 80 positions in order: one process, one thread,
Hash 128, pondering disabled, one million requested nodes per position, with
`ucinewgame` between positions. Actual nodes and UCI output are retained, including
any search that ends before its node budget. Fixed-node searches do not promise
byte-identical profiles: runtime and library paths can still vary.

## Evidence and validation

The builder writes the raw and merged profile, build logs, training records,
profile counter dump and `provenance.json`. The release workflow archives the
merged profile and provenance as a separate CI artifact.

Three exact main-function records must have positive block counters: search,
quiescence and NNUE accumulator update. These are IR-PGO block counters, not
assumed function-entry counts. Functions without standalone records may be
inlined; their absence is not a coverage assertion.

The checked-in diagnostic canary report and log establish that the pinned
compiler reports deliberately missing and mismatched profile records. The
builder verifies their checksums, successful result and exact compiler identity.
Canary validation is compiler-specific and independent of engine source commits; regenerate it on every toolchain bump.
The canary binary is deliberately unsuitable for measurements or release.

The frozen JSON includes training and held-out positions and selection
provenance. Only training positions are searched while collecting a profile.
Workload changes require a new preregistered study; do not tune the workload
against observed acceptance results.

Acceptance is performed on the actual release-candidate AVX2 asset and the
previous public release. Local measurements alone do not qualify the CI binary.
Retain single-thread trace identity, bench identity, paired held-out timing and
separate 16-thread validation. Throughput is not an Elo estimate. The final
versioned asset is verified again after its build; changing the version creates
a new binary.

## Future comparisons and compiler changes

After adoption, source-change comparisons must train both arms afresh using the
same pinned toolchain, target flags and frozen workload. Never compare a newly
trained candidate with a stale profile or an ordinary unprofiled baseline.
Record both provenance files alongside timing results.

When changing the compiler:
1. Update the exact repository pin and CI toolchain selectors together.
2. Regenerate profiles from current source; LLVM profiles are compiler-specific.
3. Repeat the profile-only diagnostic canary: remove the main search record and
   change the main quiescence hash in a separate profile, build to a separate
   target directory, and retain the expected diagnostics and compiler identity.
4. Record the canary binary hash and remove its execute permissions. Never pass
   it to timing, calibration or release tooling.
5. Replace the canary evidence only after independent review, then repeat the
   local and public-binary gates with the new compiler.
