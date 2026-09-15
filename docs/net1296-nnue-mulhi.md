# NET-1296: AVX2 NNUE multiply-high normalization

Baseline v1.0.66 (`0d3e64ee62db57c4ade9f86f489f60bf454d3da8`).
[Pre-registered experiment](https://linear.app/netsensia/issue/NET-1296).

## Mechanism and exactness

For each clipped activation x in 0..255, SCReLU computes floor(x*x/255).
Replace the shift/add sequence with unsigned high16((x*x+1)*257).
The addition stays within u16, even at x=255. Signed multiply-high would be
incorrect; this uses `_mm256_mulhi_epu16`.

Proof: write n=x*x=255q+r, with 0<=r<255 and q<=255.
Then (n+1)*257 = 65536q + 257(r+1)-q. The latter term is at least 2
and less than 65536, so the upper 16 bits are exactly q.

The replacement uses two normalization instructions per perspective instead
of four, but increases pressure on integer vector multipliers. Timing decides
whether this improves whole-search throughput. No weights, clipping, signed
dot-product accumulation, scalar fallback or search decisions change.

## Registered validation and timing

Lane-isolated comparisons for all 256 clipped values, both perspectives,
negative weights and extreme clamp inputs; existing whole-vector scalar
oracle, full release tests, formatting and workspace Clippy.
80 distinct fresh-process searches at 100k nodes, checking every reported
iteration and final best/ponder move. Bench per-position nodes must match in
every timing run (total 1,772,650).

Primary: pipeline AVX2 (`-C target-cpu=x86-64-v3`), same rustc/settings,
separate baseline/candidate build targets, Threads 1, Hash 128 MB, CPU 2,
otherwise idle Ryzen 9 5950X. Warmup then 20 alternating ABBA/BAAB blocks.
Accept >=0.5% geometric-mean speed gain and positive Student-t 95% lower bound
on paired log ratios (19 df). A pass advances unchanged source to independent
native confirmation using the same gate. No pooling, exclusions or early stop.
No Elo gain inferred from timing.

## Results

| Build | Speed gain | Paired 95% interval | Gate |
| --- | ---: | ---: | --- |
| Pipeline AVX2 (primary) | +1.17% | +0.82% to +1.52% | Pass |
| Native (independent confirmation) | +1.08% | +0.47% to +1.70% | Pass |

Both runs retained every block. These are throughput measurements, not Elo.
232 release tests passed, 3 ignored; AVX2 workspace Clippy and formatting
passed. Every search trace and bench node count matched in both builds.
Disassembly shows `vpmulhuw` replacing the normalization shifts/adds.
CPU reserved only after the bot agent explicitly confirmed idle, graceful
shutdown, no ongoing games and no remaining bot/worker/engine processes.

## Reproduction and compact evidence

Build baseline and candidate with identical compiler flags in separate target
directories, keeping executable copies. Compiler: rustc 1.98.1
(48a229cea 2026-09-01). Native uses repository `.cargo/config.toml`; AVX2 uses
`RUSTFLAGS='-C link-args=-Wl,-z,stack-size=8388608 -C target-cpu=x86-64-v3'`.
Run `scripts/compare_search_speed.py BASELINE CANDIDATE --output run.json`
on an idle machine for each build pair. It rejects identical binary hashes.
The harness now skips duplicate FENs, yielding 80 distinct cases.

Raw outputs are archived on NET-1296. To reproduce the committed compact
`results/performance/net1296.json` (trace digests and raw timing blocks):

```sh
python3 - /tmp/net1296/comparison-native.json /tmp/net1296/comparison-avx2.json results/performance/net1296.json <<'PY'
import hashlib
import json
from pathlib import Path
import sys

out = {
    'baseline_commit': '0d3e64ee62db57c4ade9f86f489f60bf454d3da8',
    'rustc': '1.98.1 (48a229cea 2026-09-01)',
    'date': '2026-09-15',
    'runs': {},
}
for name, path in zip(('native', 'avx2'), sys.argv[1:3]):
    raw = Path(path).read_bytes()
    run = json.loads(raw)
    assert len(run['identity']) == 80 and len(run['blocks']) == 20
    assert all(item['equal'] and item['results'][0] == item['results'][1]
               for item in run['identity'])
    summary = {key: run[key] for key in (
        'binaries', 'cpu', 'nodes', 'depth', 'host', 'cpu_model', 'bench_nodes',
        'speed_percent', 'ci95_percent', 'accepted',
    )}
    summary['identity'] = [{
        'fen': item['fen'], 'equal': item['equal'],
        'trace_sha256': hashlib.sha256(
            json.dumps(item['results'][0], sort_keys=True).encode()).hexdigest(),
    } for item in run['identity']]
    summary['blocks'] = [{
        'runs': [{key: item[key] for key in ('arm', 'ms', 'nodes')}
                 for item in block['runs']],
        'log_speed_ratio': block['log_speed_ratio'],
    } for block in run['blocks']]
    summary['raw_result_sha256'] = hashlib.sha256(raw).hexdigest()
    out['runs'][name] = summary
Path(sys.argv[3]).write_text(json.dumps(out, indent=2) + '\n')
PY
```

Each trace digest hashes Python `json.dumps(trace, sort_keys=True).encode()`
with default separators/ASCII escaping. Raw-result hashes cover exact file
bytes. Update baseline/compiler/date metadata when adapting to another run.

