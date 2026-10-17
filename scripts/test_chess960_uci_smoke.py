"""Tests for the raw UCI client in chess960_uci_smoke.py, with fake engines.

No real engine and no python-chess game is involved, but the module imports
python-chess, so run it with lichess-bot's virtualenv:

    ~/Work/rusty-rival/lichess-bot/.venv/bin/python -m unittest scripts/test_chess960_uci_smoke.py
"""
import importlib.util
import os
import stat
import sys
import tempfile
import time
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("chess960_uci_smoke", HERE / "chess960_uci_smoke.py")
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)


def fake_engine(directory: Path, body: str) -> Path:
    path = directory / "engine"
    path.write_text(f"#!{sys.executable}\nimport sys, time\n{body}\n")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)
    return path


def alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    # A zombie still answers kill(0); a reaped child does not
    status = Path(f"/proc/{pid}/status")
    return status.exists() and "State:\tZ" not in status.read_text()


class RawClientTests(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp())
        self.spawned = []
        real_popen = smoke.subprocess.Popen

        def tracking_popen(*args, **kwargs):
            proc = real_popen(*args, **kwargs)
            self.spawned.append(proc)
            return proc

        smoke.subprocess.Popen = tracking_popen
        self.addCleanup(setattr, smoke.subprocess, "Popen", real_popen)

    def test_silent_engine_handshake_times_out_and_is_killed(self):
        engine = fake_engine(self.dir, "time.sleep(60)")
        start = time.monotonic()
        with self.assertRaises(TimeoutError):
            smoke.Raw(str(engine), handshake_timeout=0.5)
        self.assertLess(time.monotonic() - start, 10)
        self.assertEqual(len(self.spawned), 1)
        self.assertFalse(alive(self.spawned[0].pid))
        self.assertIsNotNone(self.spawned[0].returncode)

    def test_engine_that_exits_before_uciok_is_reported_and_reaped(self):
        engine = fake_engine(self.dir, "print('id name broken', flush=True)")
        with self.assertRaises(RuntimeError):
            smoke.Raw(str(engine), handshake_timeout=5)
        self.assertIsNotNone(self.spawned[0].returncode)

    def test_engine_ignoring_quit_is_killed_on_close(self):
        body = (
            "for line in sys.stdin:\n"
            "    if line.strip() == 'uci':\n"
            "        print('id name fake', flush=True); print('uciok', flush=True)\n"
            "    if line.strip() == 'quit':\n"
            "        time.sleep(60)\n"
        )
        engine = fake_engine(self.dir, body)
        raw = smoke.Raw(str(engine), handshake_timeout=5)
        self.assertIn("uciok", raw.lines)
        start = time.monotonic()
        original_wait = raw.p.wait

        def short_wait(timeout=None):
            return original_wait(timeout=0.5 if timeout else None)

        raw.p.wait = short_wait
        raw.close()
        self.assertLess(time.monotonic() - start, 10)
        self.assertFalse(alive(raw.p.pid))

    def test_read_until_times_out_on_a_quiet_engine(self):
        body = (
            "for line in sys.stdin:\n"
            "    if line.strip() == 'uci':\n"
            "        print('uciok', flush=True)\n"
        )
        engine = fake_engine(self.dir, body)
        raw = smoke.Raw(str(engine), handshake_timeout=5)
        try:
            with self.assertRaises(TimeoutError):
                raw.read_until("readyok", timeout=0.5)
        finally:
            raw.p.kill()
            raw.p.wait()


if __name__ == "__main__":
    unittest.main()
