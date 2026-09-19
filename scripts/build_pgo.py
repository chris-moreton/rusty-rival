#!/usr/bin/env python3
"""Build Linux AVX2 with a fresh, same-source single-thread PGO profile.

This script deliberately refuses a nonempty work directory. Profile generation,
training, merge and profile use are one operation, using the pinned compiler.
"""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import shutil
import subprocess
import tempfile
import threading
import time
import tomllib

TARGET = 'x86_64-unknown-linux-gnu'
BASE_FLAGS = '-C link-args=-Wl,-z,stack-size=8388608 -C target-cpu=x86-64-v3'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def load_training(path, source=None):
    data = json.loads(path.read_text())
    training = data['training']
    heldout = data['heldout']
    keys = lambda rows: {' '.join(row['fen'].split()[:4]) for row in rows}
    if len(training) != 80 or len(keys(training)) != 80:
        raise ValueError('Expected 80 distinct frozen training positions')
    if keys(training) & keys(heldout):
        raise ValueError('Training and held-out positions overlap')
    if keys(training) & set(data['excluded_all_16_bench_keys']):
        raise ValueError('Training overlaps bench')
    if source is not None:
        source_text = (source / 'src/uci_bench.rs').read_text()
        match = re.search(r'const BENCH_FENS:\s*\[&str;\s*(\d+)\]\s*=\s*\[(.*?)\];', source_text, re.S)
        if match is None:
            raise ValueError('Cannot parse current BENCH_FENS; review exclusions before profiling')
        fens = re.findall(r'"([^"]+)"', match[2])
        if len(fens) != int(match[1]) or any(len(fen.split()) != 6 for fen in fens):
            raise ValueError('Current bench FEN parser did not recover the declared list')
        if keys(training) & {' '.join(fen.split()[:4]) for fen in fens}:
            raise ValueError('Training overlaps CURRENT source bench')
    for row in training:
        if len(row['fen'].split()) != 6:
            raise ValueError('Training FEN must have six fields')
    return training


def check_runner_isa(cpuinfo):
    rows = re.findall(r'^flags\s*:\s*(.*)$', cpuinfo, re.M)
    required = {'avx', 'avx2', 'bmi1', 'bmi2', 'fma', 'movbe', 'f16c', 'xsave',
                'pni', 'ssse3', 'sse4_1', 'sse4_2', 'popcnt', 'cx16', 'lahf_lm'}
    if not rows:
        raise RuntimeError('Training requires a Linux x86-64-v3 host with CPU feature flags')
    for index, row in enumerate(rows):
        flags = set(row.split())
        missing = sorted(required - flags)
        if not flags.intersection({'abm', 'lzcnt'}):
            missing.append('abm/lzcnt')
        if missing:
            raise RuntimeError(f'Training runner CPU {index} lacks x86-64-v3 features: {missing}')
    return {'logical_cpus_checked': len(rows), 'required_flags': sorted(required), 'lzcnt_aliases': ['abm', 'lzcnt']}

def train(binary, positions, profile_dir, expected_version, output):
    env = dict(os.environ, LLVM_PROFILE_FILE=str(profile_dir / 'train-%p-%m.profraw'))
    proc = subprocess.Popen([str(binary)], stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                            text=True, bufsize=1, env=env)
    lines = queue.Queue()
    def reader():
        try:
            for line in proc.stdout:
                lines.put(line.strip())
        finally:
            lines.put(None)
    thread = threading.Thread(target=reader, daemon=True)
    thread.start()
    def send(command):
        proc.stdin.write(command + '\n')
        proc.stdin.flush()
    def until(prefix):
        captured = []
        deadline = time.monotonic() + 120
        while True:
            try:
                line = lines.get(timeout=max(0.001, deadline - time.monotonic()))
            except queue.Empty:
                raise TimeoutError('Engine did not send ' + prefix) from None
            if line is None:
                raise RuntimeError('Engine exited before ' + prefix)
            captured.append(line)
            if re.search(r'\b(error|invalid|illegal|panicked)\b', line, re.I):
                raise RuntimeError('Engine rejected training command: ' + line)
            if line.startswith(prefix):
                return captured
            if time.monotonic() >= deadline:
                raise TimeoutError('Engine did not send ' + prefix)
    records = []
    try:
        send('uci')
        uci = until('uciok')
        if 'id name Rusty Rival ' + expected_version not in uci:
            raise RuntimeError('Training binary has wrong UCI version')
        send('setoption name Threads value 1')
        send('setoption name Hash value 128')
        send('setoption name Ponder value false')
        send('isready')
        until('readyok')
        for i, row in enumerate(positions):
            send('ucinewgame')
            send('position fen ' + row['fen'])
            send('isready')
            until('readyok')
            start = time.monotonic()
            send('go nodes 1000000')
            captured = until('bestmove')
            info = next(line for line in reversed(captured) if ' nodes ' in line)
            records.append({'index': i, 'fen': row['fen'],
                            'requested_nodes': 1000000,
                            'reported_nodes': int(re.search(r' nodes (\d+)', info)[1]),
                            'wall_seconds': time.monotonic() - start,
                            'lines': captured})
            output.write_text(json.dumps(records, indent=2) + '\n')
            print(f'Training {i + 1}/{len(positions)}', flush=True)
    finally:
        if proc.poll() is None:
            try:
                send('quit')
                proc.wait(timeout=15)
            except (BrokenPipeError, subprocess.TimeoutExpired):
                proc.kill()
                proc.wait()
        proc.stdin.close()
        thread.join(timeout=5)
        proc.stdout.close()
    if proc.returncode != 0:
        raise RuntimeError(f'Training engine exited {proc.returncode}')
    return records



def audit_hot_counts(text):
    records = {}
    pattern = r'^  (.+):\n    Hash: (.*?)\n    Counters: (.*?)\n    Block counts: \[(.*?)\]'
    for match in re.finditer(pattern, text, re.M):
        name, hash_value, number, values = match.groups()
        if '11rusty_rival' not in name:
            continue
        for suffix in ['6search6search', '7quiesce7quiesce', '23update_accumulator_from']:
            if name.endswith(suffix):
                counts = [int(value) for value in values.split(',') if value.strip()]
                records.setdefault(suffix, []).append({
                    'symbol': name, 'hash': hash_value, 'counter_count': int(number),
                    'first_block_counter': counts[0] if counts else 0,
                    'maximum_block_counter': max(counts, default=0)})
    for suffix in ['6search6search', '7quiesce7quiesce', '23update_accumulator_from']:
        matches = records.get(suffix, [])
        if not matches or not any(row['maximum_block_counter'] > 0 for row in matches):
            raise RuntimeError('No positive profile counters for main engine function ' + suffix)
    return records

def profile_warnings(log):
    mismatch = [line for line in log.splitlines() if 'hash mismatch' in line.lower() or 'control flow change detected' in line.lower()]
    missing = [line for line in log.splitlines() if re.search(r'(no profile data|profile data.*missing|missing.*profile data)', line, re.I)]
    engine_missing = [line for line in missing if '11rusty_rival' in line]
    return {'hash_mismatch': mismatch, 'missing_function_count': len(missing), 'missing_function_lines': missing, 'engine_missing_function_lines': engine_missing}

def validate_profile_warnings(report):
    if report['hash_mismatch']:
        raise RuntimeError('PGO control-flow hash mismatch')
    if report['engine_missing_function_lines']:
        raise RuntimeError('Engine function missing profile data')


def validate_canary_log(canary, log):
    warnings = profile_warnings(log)
    missing = warnings['missing_function_lines']
    mismatch = warnings['hash_mismatch']
    search = [line for line in missing
              if re.search(r'11rusty_rival6search6search\s+Hash\s*=', line)]
    quiesce = [line for line in mismatch
               if re.search(r'11rusty_rival7quiesce7quiesce\s+Hash\s*=', line)]
    derived = {
        'all_missing_count': len(missing),
        'all_mismatch_count': len(mismatch),
        'engine_missing_lines': warnings['engine_missing_function_lines'],
        'engine_mismatch_lines': [line for line in mismatch if '11rusty_rival' in line],
        'missing_search_diagnostics': search,
        'mismatched_quiesce_diagnostics': quiesce,
        'passed': bool(search and quiesce and canary.get('exit_code') == 0
                       and 'Finished `release` profile' in log),
    }
    for key, value in derived.items():
        if canary.get(key) != value:
            raise RuntimeError('Diagnostic canary metadata disagrees with hashed log: ' + key)
    if not derived['passed']:
        raise RuntimeError('Diagnostic canary log lacks successful positive controls')
    return derived


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--positions', type=Path, required=True)
    parser.add_argument('--work-dir', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--lock-file', type=Path, default=Path(tempfile.gettempdir()) / 'rusty-rival-performance.lock')
    args = parser.parse_args()
    if "RIVAL_NET365_DIAGNOSTIC" in os.environ:
        raise RuntimeError("Unset RIVAL_NET365_DIAGNOSTIC before profiling")
    source, positions, work = args.source.resolve(), args.positions.resolve(), args.work_dir.resolve()
    lock = args.lock_file.open('a')
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    status = subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=source, text=True)
    if status:
        raise RuntimeError('PGO requires a clean committed source tree: ' + status)
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip()
    describe = subprocess.check_output(['git', 'describe', '--always', '--tags'], cwd=source, text=True).strip()
    runner_isa = check_runner_isa(Path("/proc/cpuinfo").read_text())
    rows = load_training(positions, source)
    compiler = subprocess.check_output(['rustc', '-Vv'], cwd=source, text=True)
    pin_file = source / 'rust-toolchain.toml'
    if not pin_file.exists():
        raise RuntimeError('Pin first: rust-toolchain.toml must specify an exact release')
    pinned = tomllib.loads(pin_file.read_text())['toolchain']['channel']
    if not re.fullmatch(r'\d+\.\d+\.\d+', pinned):
        raise RuntimeError('Pin first: toolchain channel must be an exact release')
    if not re.search(r'^release: ' + re.escape(pinned) + r'$', compiler, re.M):
        raise RuntimeError('Active compiler differs from pinned release compiler')
    canary_file = source / 'scripts/pgo/diagnostic-canary.json'
    canary_log = source / 'scripts/pgo/diagnostic-canary.log'
    canary = json.loads(canary_file.read_text())
    if not canary.get('passed') or canary['compiler'] != compiler:
        raise RuntimeError('Diagnostic canary must pass for this exact compiler; regenerate after toolchain changes')
    if sha(canary_log) != canary['log_sha256']:
        raise RuntimeError('Diagnostic canary log checksum mismatch')
    validate_canary_log(canary, canary_log.read_text())
    if work.exists() and any(work.iterdir()):
        raise ValueError('PGO work directory must be empty (no stale profiles)')
    work.mkdir(parents=True, exist_ok=True)
    profiles = work / 'profiles'
    profiles.mkdir()
    version = tomllib.loads((source / 'Cargo.toml').read_text())['package']['version']
    manifest = {'version': version, 'compiler': compiler, 'target': TARGET,
                'base_rustflags': BASE_FLAGS, 'runner_isa': runner_isa, 'positions_sha256': sha(positions),
                'training_threads': 1, 'training_hash_mb': 128,
                'training_nodes_per_position': 1000000, 'diagnostic_canary_sha256': sha(canary_file),
                'diagnostic_canary_log_sha256': sha(canary_log), 'builds': []}
    manifest.update(commit=commit, git_describe=describe, git_status=status,
                    native_dependencies='mimalloc and zstd C/C++ code receive no Rust PGO instrumentation or profile-use')
    sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], cwd=source, text=True).strip())
    host = re.search(r'^host: (.+)$', compiler, re.M)[1]
    profdata = sysroot / 'lib/rustlib' / host / 'bin/llvm-profdata'
    if not profdata.is_file():
        raise RuntimeError('Missing llvm-profdata: install rustup component llvm-tools for the pinned toolchain')
    def save():
        (work / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')
    def build(name, flag):
        flags = BASE_FLAGS + ' ' + flag
        target_dir = work / name
        command = ['cargo', 'build', '--locked', '--release', '--target', TARGET,
                   '--bin', 'rusty-rival', '--manifest-path', str(source / 'Cargo.toml')]
        manifest['builds'].append({'name': name, 'command': command, 'rustflags': flags})
        save()
        env = dict(os.environ, RUSTFLAGS=flags, CARGO_TARGET_DIR=str(target_dir))
        env.pop('LLVM_PROFILE_FILE', None)
        env.pop('CARGO_ENCODED_RUSTFLAGS', None)
        with (work / (name + '.log')).open('w') as log:
            subprocess.run(command, cwd=source, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
        return target_dir / TARGET / 'release/rusty-rival'
    instrumented = build('instrumented', '-C profile-generate=' + str(profiles))
    manifest['instrumented_sha256'] = sha(instrumented)
    save()
    train(instrumented, rows, profiles, version, work / 'training.json')
    raw = sorted(profiles.glob('*.profraw'))
    if not raw or any(p.stat().st_size == 0 for p in raw):
        raise RuntimeError('No complete raw profile generated')
    merged = work / 'merged.profdata'
    subprocess.run([str(profdata), 'merge', '-o', str(merged), *map(str, raw)], check=True)
    manifest['raw_profile_sha256'] = {p.name: sha(p) for p in raw}
    manifest['merged_profile_sha256'] = sha(merged)
    counts_text = subprocess.check_output([str(profdata), 'show', '--all-functions', '--counts', str(merged)], text=True)
    (work / 'profile-counts.txt').write_text(counts_text)
    manifest['hot_profile_counters'] = audit_hot_counts(counts_text)
    manifest['profile_count_semantics'] = 'Positive IR-PGO block counters; not assumed entry counts or compared to engine node totals. Absent standalone symbols do not prove absence of inlined profile coverage.'
    save()
    binary = build('optimized', '-C profile-use=' + str(merged) + ' -C llvm-args=-pgo-warn-missing-function')
    log = (work / 'optimized.log').read_text()
    manifest['profile_warnings'] = profile_warnings(log)
    save()
    validate_profile_warnings(manifest['profile_warnings'])
    if b'__llvm_profile_' in binary.read_bytes():
        raise RuntimeError('Final binary still contains instrumentation')
    manifest['optimized_sha256'] = sha(binary)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(binary, args.output)
    save()
    print('PGO binary:', args.output, flush=True)

if __name__ == '__main__':
    main()
