#!/usr/bin/env python3
"""Chess960 UCI smoke test for rusty-rival (NET-1762).

Drives the engine the way lichess-bot does, through python-chess with
``chess960=True`` (Shredder-FEN start positions, king-takes-rook castling,
``UCI_Chess960`` set per game), and over the raw protocol for the cases
python-chess hides: both castle spellings on ``position ... moves``, the
spelling of ``bestmove`` with the option on and off, and the clocked
``go`` budgets the bot relies on.

Needs python-chess (lichess-bot's virtualenv has it):

    ~/Work/rusty-rival/lichess-bot/.venv/bin/python scripts/chess960_uci_smoke.py \
        --engine target/release/rusty-rival [--json report.json]

Exit status 0 means every check passed; the report lists each one.
"""

from __future__ import annotations

import argparse
import json
import queue
import random
import subprocess
import sys
import threading
import time

import chess
import chess.engine

# Chess960 start numbers: the extremes, the standard start (518), and a spread
START_NUMBERS = [0, 1, 100, 255, 356, 518, 700, 833, 959]


class Raw:
    """Minimal UCI client, for the checks that need the text of the protocol.

    Output is read by a thread into a queue so that every wait has a deadline,
    including a wait on an engine that prints nothing at all."""

    def __init__(self, path: str, handshake_timeout: float = 30.0):
        self.p = subprocess.Popen([path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
        self.q: queue.Queue[str | None] = queue.Queue()
        threading.Thread(target=self._pump, daemon=True).start()
        try:
            self.send("uci")
            self.lines = self.read_until("uciok", handshake_timeout)
        except BaseException:
            # A failed handshake must not leave the engine running: the caller
            # never gets an object to close
            self.p.kill()
            self.p.wait()
            raise

    def _pump(self) -> None:
        assert self.p.stdout is not None
        try:
            for line in self.p.stdout:
                self.q.put(line.rstrip("\n"))
        finally:
            self.q.put(None)

    def send(self, line: str) -> None:
        assert self.p.stdin is not None
        self.p.stdin.write(line + "\n")
        self.p.stdin.flush()

    def read_until(self, token: str, timeout: float = 30.0) -> list[str]:
        deadline = time.monotonic() + timeout
        out: list[str] = []
        while True:
            remaining = deadline - time.monotonic()
            # Checked before reading, so an engine that keeps printing cannot
            # hold the wait open past its deadline
            if remaining <= 0:
                raise TimeoutError(f"no {token!r} within {timeout}s; got {out[-5:]}")
            try:
                line = self.q.get(timeout=remaining)
            except queue.Empty:
                raise TimeoutError(f"no {token!r} within {timeout}s; got {out[-5:]}") from None
            if line is None:
                raise RuntimeError(f"engine closed its output before {token!r}; got {out[-5:]}")
            out.append(line)
            if line.startswith(token):
                return out

    def bestmove(self, go: str, timeout: float = 30.0) -> tuple[str, float]:
        start = time.monotonic()
        self.send(go)
        lines = self.read_until("bestmove", timeout)
        return lines[-1].split()[1], (time.monotonic() - start) * 1000.0

    def isready(self) -> None:
        self.send("isready")
        self.read_until("readyok")

    def close(self) -> None:
        try:
            self.send("quit")
            self.p.wait(timeout=10)
        except (subprocess.TimeoutExpired, OSError):
            self.p.kill()
            self.p.wait()


def legal_after(fen: str, moves: list[str], bestmove: str, moves_960: bool, bestmove_960: bool) -> bool:
    """True if `bestmove` is legal in the position reached from `fen` by `moves`.

    The moves and the bestmove may use different castling spellings (the
    option can change between them), so each is parsed on a board of the
    matching kind; python-chess rejects an unparsable or illegal move."""
    board = chess.Board(fen, chess960=moves_960)
    for move in moves:
        board.push_uci(move)
    reached = chess.Board(board.fen(shredder=True), chess960=bestmove_960)
    try:
        return reached.parse_uci(bestmove) in reached.legal_moves
    except ValueError:
        return False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--engine", required=True)
    parser.add_argument("--json", help="write the per-check report here")
    parser.add_argument("--nodes", type=int, default=4000, help="nodes per move in the self-play games")
    parser.add_argument("--plies", type=int, default=40, help="maximum plies per self-play game")
    parser.add_argument("--seed", type=int, default=1762)
    args = parser.parse_args()

    checks: list[dict] = []

    def check(name: str, ok: bool, detail: str = "") -> None:
        checks.append({"name": name, "ok": bool(ok), "detail": detail})
        print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))

    # ---- raw protocol -----------------------------------------------------
    raw = Raw(args.engine)
    try:
        check("uci advertises UCI_Chess960", any(l.startswith("option name UCI_Chess960 type check") for l in raw.lines))
        name = next((l for l in raw.lines if l.startswith("id name")), "")
        check("uci id name", bool(name), name)

        raw.send("setoption name UCI_Chess960 value true")
        raw.send("ucinewgame")
        raw.isready()
        # Shredder-FEN start, king-takes-rook castles both sides, as lichess-bot sends them
        fen = "1rk3r1/pppppppp/8/8/8/8/PPPPPPPP/1RK3R1 w GBgb - 0 1"
        raw.send(f"position fen {fen} moves c1g1 c8b8")
        raw.isready()
        mv, _ = raw.bestmove("go depth 1")
        check("position accepts king-takes-rook castles (960 on)", legal_after(fen, ["c1g1", "c8b8"], mv, True, True), mv)
        # Standard spelling is accepted as well
        fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"
        raw.send(f"position fen {fen} moves e1g1 e8c8")
        raw.isready()
        mv, _ = raw.bestmove("go depth 1")
        check("position accepts e1g1-style castles (960 on)", legal_after(fen, ["e1g1", "e8c8"], mv, False, True), mv)
        # bestmove spelling: option on -> king takes rook
        raw.send("position fen 6kr/8/8/8/8/8/8/6KR w Hh - 0 1")
        mv, _ = raw.bestmove("go depth 1 searchmoves g1h1")
        check("bestmove spells an in-place castle as g1h1 (960 on)", mv == "g1h1", mv)
        raw.send(f"position fen {fen}")
        mv, _ = raw.bestmove("go depth 1 searchmoves e1g1")
        check("bestmove spells a standard castle as e1h1 (960 on)", mv == "e1h1", mv)
        raw.send("setoption name UCI_Chess960 value false")
        mv, _ = raw.bestmove("go depth 1 searchmoves e1g1")
        check("bestmove spells a standard castle as e1g1 (960 off)", mv == "e1g1", mv)
        raw.send(f"position fen {fen} moves e1h1")
        raw.isready()
        mv, _ = raw.bestmove("go depth 1")
        check("position accepts e1h1 even with 960 off", legal_after(fen, ["e1h1"], mv, True, False), mv)

        # Clocked budgets (the lichess-bot path): hard cap is the engine's, +250 ms grace
        raw.send("setoption name UCI_Chess960 value true")
        fen = "bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9"
        raw.send(f"position fen {fen}")
        mv, ms = raw.bestmove("go wtime 1000 btime 1000 movestogo 1")
        check("clocked go 1000ms movestogo 1 returns in time", ms < 1000 + 250 and legal_after(fen, [], mv, True, True), f"{mv} in {ms:.0f} ms")
        mv, ms = raw.bestmove("go wtime 10000 btime 10000 winc 100 binc 100", timeout=20)
        check("clocked go 10000ms returns in time", ms < 10000 + 250 and legal_after(fen, [], mv, True, True), f"{mv} in {ms:.0f} ms")
        # Ponder then ponderhit still produces a bestmove
        fen = "bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR b HFhf - 2 9"
        raw.send(f"position fen {fen}")
        raw.send("go ponder wtime 3000 btime 3000")
        time.sleep(0.3)
        start = time.monotonic()
        raw.send("ponderhit")
        lines = raw.read_until("bestmove", 10)
        ms = (time.monotonic() - start) * 1000
        mv = lines[-1].split()[1] if len(lines[-1].split()) > 1 else ""
        check("ponderhit yields bestmove", legal_after(fen, [], mv, True, True), f"{lines[-1]} after {ms:.0f} ms")
    finally:
        raw.close()

    # ---- python-chess, as lichess-bot drives it ---------------------------
    rng = random.Random(args.seed)
    engine = chess.engine.SimpleEngine.popen_uci(args.engine)
    engine.configure({"Threads": 1, "Hash": 64})
    check("python-chess sees UCI_Chess960 as a managed option", "UCI_Chess960" in engine.options)
    castles = 0
    plies = 0
    try:
        numbers = START_NUMBERS + [rng.randrange(960) for _ in range(6)]
        for number in numbers:
            board = chess.Board.from_chess960_pos(number)
            for _ in range(args.plies):
                if board.is_game_over(claim_draw=True):
                    break
                result = engine.play(board, chess.engine.Limit(nodes=args.nodes), game=number)
                assert result.move is not None
                if result.move not in board.legal_moves:
                    raise RuntimeError(f"illegal {result.move} in {board.fen()} (960 #{number})")
                if board.is_castling(result.move):
                    castles += 1
                board.push(result.move)
                plies += 1
            check(f"960 start #{number} self-play legal", True, f"{len(board.move_stack)} plies, {board.fen(shredder=True)}")
        # A standard game through the same driver, chess960=False
        board = chess.Board()
        standard_castles = 0
        for _ in range(args.plies):
            if board.is_game_over(claim_draw=True):
                break
            result = engine.play(board, chess.engine.Limit(nodes=args.nodes), game="standard")
            assert result.move is not None and result.move in board.legal_moves
            if board.is_castling(result.move):
                standard_castles += 1
            board.push(result.move)
        check("standard self-play legal (960 off)", True, f"{len(board.move_stack)} plies, {standard_castles} castles")
        # Resume a 960 game from a mid-game Shredder FEN with castles already played
        board = chess.Board("1rk3r1/pppppppp/8/8/8/8/PPPPPPPP/1RK3R1 w GBgb - 0 1", chess960=True)
        board.push_uci("c1g1")
        board.push_uci("c8b8")
        result = engine.play(board, chess.engine.Limit(depth=4), game="resume")
        check("resume after king-takes-rook castles", result.move in board.legal_moves, str(result.move))
    except Exception as exc:  # noqa: BLE001 - report, then fail
        check("python-chess driver", False, repr(exc))
    finally:
        engine.quit()
    check("self-play exercised castling", castles >= 3, f"{castles} castles in {plies} plies")

    failed = [c for c in checks if not c["ok"]]
    report = {
        "engine": args.engine,
        "python_chess": chess.__version__,
        "id_name": name,
        "checks": checks,
        "failed": len(failed),
    }
    if args.json:
        with open(args.json, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=2)
    print(f"{len(checks) - len(failed)}/{len(checks)} checks passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
