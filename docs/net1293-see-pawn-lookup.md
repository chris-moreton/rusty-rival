# NET-1293: SEE pawn recapture lookup

Baseline: v1.0.65, `61e6ae5947943f6b2b56a82e011e71dc1e097f81`.
Experiment registered before timing in [NET-1293](https://linear.app/netsensia/issue/NET-1293).

## Change

SEE previously scanned every friendly pawn until it found the first pawn
attacking the exchange square. It now intersects the friendly pawns with the
opposite colour's pawn-attack mask at that square and selects the lowest set
bit. This preserves the selected pawn, promotion encoding, subsequent attacker
order, and legality checks. In particular, it does not change SEE's existing
behaviour when the first recapturer is pinned.

The initial full-capacity move-list hypothesis was rejected by disassembly:
the compiler already removes that temporary. Fresh release disassembly shows
the pawn loop does survive, and the candidate replaces it with a lookup.

## Validation

- 231 release tests passed, 3 ignored, including a new comparison with the
  forward pawn generator over both colours, every origin/destination square,
  promotion ranks, and multiple-attacker pawn sets.
- `cargo fmt --all -- --check` and `cargo clippy --workspace -- -D warnings` pass.
- Each build pair runs 80 fresh-process searches at 100,000 nodes: the 16 bench
  FENs and 16 evenly spaced positions from each of Arasan, EET, WAC, and STS.
  Every reported iteration's depth, seldepth, nodes, score/bound and PV, plus
  the final best/ponder move, must match.
- The 16-position depth-12 bench must match per-position and total nodes in
  every timing run. Signature: **1,772,650**.

## Timing protocol

Ryzen 9 5950X, Linux, rustc 1.98.1; one thread, Hash 128, process affinity CPU 2.
The Lichess agent confirmed the current game had finished and all bot/worker/
engine processes had exited before builds or searches started. Builds and tests
finished before timing. Baseline and candidate use the same compiler settings.

Each pair gets an untimed warmup, then 20 blocks alternating ABBA and BAAB.
A block's speed ratio is mean baseline milliseconds / mean candidate milliseconds.
The reported gain is the geometric mean of the 20 block ratios, with a Student-t
95% interval on log ratios (19 degrees of freedom). The registered acceptance
gate is at least +0.5% and a positive lower confidence bound. All blocks count;
there is no outlier removal or early stopping.

| Build | Speed gain | Paired 95% interval | Result |
| --- | ---: | ---: | --- |
| Native (`.cargo/config.toml`) | +1.26% | +0.66% to +1.87% | Pass |
| Portable AVX2 (release pipeline flags) | +0.97% | +0.55% to +1.39% | Pass |

The AVX2 run is an independent portability check, specified before its timing;
it is not pooled with the native run. These are throughput measurements, not
an Elo estimate or a playing-strength test.

## Reproduction

Build the baseline and candidate from their source revisions with identical
flags, keeping separate executable copies. Use separate target directories or
force recompilation when changing source roots: Cargo reused a stale artifact
in a shared target directory during setup, caught by identical binary hashes
before any measurements. The harness rejects identical input hashes.

```sh
python3 scripts/compare_search_speed.py BASELINE CANDIDATE --output result.json
```

Native: repository `.cargo/config.toml`. Portable AVX2:

```sh
RUSTFLAGS='-C link-args=-Wl,-z,stack-size=8388608 -C target-cpu=x86-64-v3' cargo build --release --locked --bin rusty-rival
```

Run the harness only on an idle host after coordinating with the bot agent.

The [committed evidence](../results/performance/net1293.json) records binary hashes, host, all 80 FENs and matching trace digests per build, and every timing block. Full raw transcripts are in `/tmp/net1293/comparison.json` and `/tmp/net1293/comparison-avx2.json` on the experiment host; their hashes are recorded in the evidence.
