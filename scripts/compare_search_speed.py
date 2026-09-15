#!/usr/bin/env python3
"""Compare node-identical engines with fixed-node searches and paired bench timing.

Run only on an idle machine. Both executables must use the same build settings.
Results include binary hashes, raw timings, and a paired confidence interval;
they measure throughput, not Elo. Example:
  python3 scripts/compare_search_speed.py BASE CAND --output /tmp/comparison.json
"""

import argparse
import hashlib
import json
import math
import os
import platform
from pathlib import Path
import queue
import re
import statistics
import subprocess
import threading


class Engine:
    def __init__(self, path):
        self.proc = subprocess.Popen(
            [str(path)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, text=True, bufsize=1,
        )
        self.lines = queue.Queue()
        def read():
            for line in self.proc.stdout:
                self.lines.put(line.strip())
            self.lines.put(None)
        threading.Thread(target=read, daemon=True).start()
        self.send('uci')
        self.until('uciok')
        self.send('setoption name Threads value 1')
        self.send('setoption name Hash value 128')
        self.send('isready')
        self.until('readyok')

    def send(self, command):
        self.proc.stdin.write(command + '\n')
        self.proc.stdin.flush()

    def until(self, prefix):
        lines = []
        while True:
            line = self.lines.get(timeout=120)
            if line is None:
                raise RuntimeError('Engine exited before ' + prefix)
            lines.append(line)
            if line.startswith(prefix):
                return lines

    def close(self):
        if self.proc.poll() is None:
            self.send('quit')
            try:
                self.proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                self.proc.wait()


def search(path, fen, nodes):
    engine = Engine(path)
    try:
        engine.send('position fen ' + fen)
        engine.send('go nodes ' + str(nodes))
        lines = engine.until('bestmove')
        info = [line for line in lines if line.startswith('info depth ')]
        if not info:
            raise RuntimeError('Search produced no depth information')
        # Timing and occupancy are not search identity. Preserve scores, bounds,
        # PV, depth, seldepth, and node counts from every reported iteration.
        normalized = [re.sub(r'\s+(?:time|nps|hashfull|tbhits)\s+\d+', '', line)
                      for line in info]
        return {'info': normalized, 'bestmove': lines[-1]}
    finally:
        engine.close()


def bench(path, depth):
    engine = Engine(path)
    try:
        engine.send('bench depth ' + str(depth))
        engine.send('isready')
        lines = engine.until('readyok')
        def number(label):
            line = next(line for line in lines if line.startswith(label))
            return int(line.split(':', 1)[1].strip().split()[0].replace(',', ''))
        return {'ms': number('Time '), 'nodes': number('Nodes searched'),
                'positions': [line for line in lines if line.startswith('Position ')]}
    finally:
        engine.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('baseline', type=Path)
    parser.add_argument('candidate', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cpu', type=int, default=2)
    parser.add_argument('--nodes', type=int, default=100000)
    parser.add_argument('--depth', type=int, default=12)
    parser.add_argument('--blocks', type=int, default=20)
    args = parser.parse_args()
    if args.blocks != 20:
        parser.error('Use 20 blocks: the confidence interval uses t(19) = 2.093.')
    os.sched_setaffinity(0, {args.cpu})
    root = Path(__file__).resolve().parents[1]
    paths = [args.baseline.resolve(), args.candidate.resolve()]
    result = {'binaries': [{'path': str(p), 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}
                           for p in paths], 'cpu': args.cpu, 'nodes': args.nodes,
              'depth': args.depth, 'host': platform.uname()._asdict(),
              'cpu_model': next(line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines()
                                if line.startswith('model name')),
              'identity': [], 'blocks': []}
    if result['binaries'][0]['sha256'] == result['binaries'][1]['sha256']:
        raise RuntimeError('Identical binaries: verify build provenance before timing')
    def save():
        args.output.write_text(json.dumps(result, indent=2) + '\n')
    # Include all bench positions plus 64 evenly spaced independent suite cases.
    source = (root / 'src/uci_bench.rs').read_text().split('const BENCH_FENS:')[1].split('];', 1)[0]
    fens = re.findall(r'"([^"]+)"', source)
    seen = {' '.join(fen.split()[:4]) for fen in fens}
    for suite in ['arasan18', 'eet', 'wac', 'sts']:
        # Legacy EPD comments contain Latin-1 names; FEN fields are ASCII.
        entries = [line for line in (root / f'epd/suites/{suite}.epd').read_text(encoding='latin-1').splitlines()
                   if line.strip() and not line.startswith('#')]
        for i in range(16):
            start = i * (len(entries) - 1) // 15
            for offset in range(len(entries)):
                fen = ' '.join(entries[(start + offset) % len(entries)].split()[:4])
                if fen not in seen:
                    seen.add(fen)
                    fens.append(fen + ' 0 1')
                    break
            else:
                raise RuntimeError('Not enough distinct positions in ' + suite)
    for i, fen in enumerate(fens):
        records = [search(path, fen, args.nodes) for path in paths]
        result['identity'].append({'fen': fen, 'equal': records[0] == records[1], 'results': records})
        save()
        if records[0] != records[1]:
            raise RuntimeError(f'Search divergence on position {i + 1}: {fen}')
        if (i + 1) % 16 == 0:
            print(f'Identity: {i + 1}/{len(fens)} positions match', flush=True)
    reference = bench(paths[0], args.depth)
    warm = bench(paths[1], args.depth)
    if (reference['nodes'], reference['positions']) != (warm['nodes'], warm['positions']):
        raise RuntimeError('Bench divergence')
    result['bench_nodes'] = reference['nodes']
    for i in range(args.blocks):
        order = [0, 1, 1, 0] if i % 2 == 0 else [1, 0, 0, 1]
        runs = []
        for arm in order:
            record = bench(paths[arm], args.depth)
            if (record['nodes'], record['positions']) != (reference['nodes'], reference['positions']):
                raise RuntimeError('Bench divergence during timing')
            runs.append({'arm': arm, **record})
        times = [statistics.mean(r['ms'] for r in runs if r['arm'] == arm) for arm in [0, 1]]
        result['blocks'].append({'runs': runs, 'log_speed_ratio': math.log(times[0] / times[1])})
        save()
        print(f'Block {i + 1}/20: speed {(times[0] / times[1] - 1) * 100:+.2f}%', flush=True)
    ratios = [block['log_speed_ratio'] for block in result['blocks']]
    mean = statistics.mean(ratios)
    error = 2.093 * statistics.stdev(ratios) / math.sqrt(len(ratios))
    result['speed_percent'] = 100 * math.expm1(mean)
    result['ci95_percent'] = [100 * math.expm1(mean - error), 100 * math.expm1(mean + error)]
    result['accepted'] = result['speed_percent'] >= 0.5 and result['ci95_percent'][0] > 0
    save()
    print(json.dumps({key: result[key] for key in ['speed_percent', 'ci95_percent', 'accepted']}, indent=2))


if __name__ == '__main__':
    main()
