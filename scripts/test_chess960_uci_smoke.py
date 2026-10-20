"""Tests for the raw UCI client in chess960_uci_smoke.py, with fake engines.

No real engine and no python-chess game is involved, but the module imports
python-chess, so run it with lichess-bot's virtualenv:

    ~/Work/rusty-rival/lichess-bot/.venv/bin/python -m unittest scripts/test_chess960_uci_smoke.py
"""
import contextlib
import importlib.util
import io
import json
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
        # wait() has reaped the engine once returncode is set
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
        self.assertIsNotNone(raw.p.returncode)

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

    def test_read_until_times_out_while_output_keeps_arriving(self):
        # Output is always waiting, as from an engine that streams info lines
        # faster than they are read, and the clock is scripted: only the
        # deadline can end the wait
        class Streaming:
            gets = 0

            def get(self, timeout=None):
                Streaming.gets += 1
                if Streaming.gets > 100:
                    raise AssertionError("read past the deadline")
                return "info string busy"

        clock = iter(float(tick) for tick in range(1000))
        raw = smoke.Raw.__new__(smoke.Raw)
        raw.q = Streaming()
        real_monotonic = smoke.time.monotonic
        smoke.time.monotonic = lambda: next(clock)
        try:
            with self.assertRaises(TimeoutError):
                raw.read_until("readyok", timeout=5)
        finally:
            smoke.time.monotonic = real_monotonic
        # Deadline at t=5; reads at t=1..4 succeed and t=5 stops before reading
        self.assertEqual(Streaming.gets, 4)


class FakeRaw:
    """Stands in for the raw-protocol prelude, so main() reaches python-chess."""

    lines = ["id name fake", "option name UCI_Chess960 type check default false"]

    def __init__(self, path):
        pass

    def send(self, line):
        pass

    def isready(self):
        pass

    def bestmove(self, go, timeout=30.0):
        return "e1g1", 1.0

    def read_until(self, token, timeout=30.0):
        return ["bestmove e1g1"]

    def close(self):
        pass


class FakeEngine:
    def __init__(self, configure_error=None, options_error=None):
        self.configure_error = configure_error
        self.options_error = options_error
        self.quits = 0
        self.plays = 0

    def configure(self, options):
        if self.configure_error:
            raise self.configure_error

    @property
    def options(self):
        if self.options_error:
            raise self.options_error
        return {"UCI_Chess960": None}

    def play(self, *args, **kwargs):
        self.plays += 1
        raise AssertionError("play after a failed setup")

    def quit(self):
        self.quits += 1


class DriverSetupTests(unittest.TestCase):
    """main() must quit the python-chess engine even when setting it up fails."""

    def run_main(self, engine):
        report = Path(tempfile.mkdtemp()) / "report.json"
        patches = [
            (smoke, "Raw", FakeRaw),
            (smoke, "legal_after", lambda *args: True),
            (smoke.time, "sleep", lambda seconds: None),
            (smoke.chess.engine.SimpleEngine, "popen_uci", lambda path: engine),
            (smoke.sys, "argv", ["chess960_uci_smoke.py", "--engine", "fake", "--json", str(report)]),
        ]
        for owner, name, value in patches:
            self.addCleanup(setattr, owner, name, getattr(owner, name))
            setattr(owner, name, value)
        with contextlib.redirect_stdout(io.StringIO()):
            status = smoke.main()
        return status, json.loads(report.read_text())

    def assert_failed_setup_cleaned_up(self, engine):
        status, report = self.run_main(engine)
        self.assertNotEqual(status, 0)
        self.assertEqual(engine.quits, 1)
        self.assertEqual(engine.plays, 0)
        driver = [c for c in report["checks"] if c["name"] == "python-chess driver"]
        self.assertEqual(len(driver), 1)
        self.assertFalse(driver[0]["ok"])
        self.assertGreater(report["failed"], 0)

    def test_engine_is_quit_when_configure_fails(self):
        self.assert_failed_setup_cleaned_up(FakeEngine(configure_error=RuntimeError("configure failed")))

    def test_engine_is_quit_when_reading_its_options_fails(self):
        self.assert_failed_setup_cleaned_up(FakeEngine(options_error=RuntimeError("options failed")))


if __name__ == "__main__":
    unittest.main()
