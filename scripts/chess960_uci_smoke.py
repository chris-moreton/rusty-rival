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
import random
import subprocess
import sys
import time

import chess
import chess.engine

# Chess960 start numbers: the extremes, the standard start (518), and a spread
START_NUMBERS = [0, 1, 100, 255, 356, 518, 700, 833, 959]


class Raw:
    """Minimal UCI client, for the checks that need the text of the protocol."""

    def __init__(self, path: str):
        self.p = subprocess.Popen([path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
        self.send("uci")
        self.lines = self.read_until("uciok")

    def send(self, line: str) -> None:
        assert self.p.stdin is not None
        self.p.stdin.write(line + "\n")
        self.p.stdin.flush()

    def read_until(self, token: str, timeout: float = 30.0) -> list[str]:
        assert self.p.stdout is not None
        deadline = time.monotonic() + timeout
        out: list[str] = []
        while time.monotonic() < deadline:
            line = self.p.stdout.readline()
            if not line:
                break
            out.append(line.rstrip("\n"))
            if line.startswith(token):
                return out
        raise TimeoutError(f"no {token!r} within {timeout}s; got {out[-5:]}")

    def bestmove(self, go: str, timeout: float = 30.0) -> tuple[str, float]:
        start = time.monotonic()
        self.send(go)
        lines = self.read_until("bestmove", timeout)
        return lines[-1].split()[1], (time.monotonic() - start) * 1000.0

    def isready(self) -> None:
        self.send("isready")
        self.read_until("readyok")

    def close(self) -> None:
        self.send("quit")
        self.p.wait(timeout=10)


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
    check("uci advertises UCI_Chess960", any(l.startswith("option name UCI_Chess960 type check") for l in raw.lines))
    name = next((l for l in raw.lines if l.startswith("id name")), "")
    check("uci id name", bool(name), name)

    raw.send("setoption name UCI_Chess960 value true")
    raw.send("ucinewgame")
    raw.isready()
    # Shredder-FEN start, king-takes-rook castles both sides, as lichess-bot sends them
    raw.send("position fen 1rk3r1/pppppppp/8/8/8/8/PPPPPPPP/1RK3R1 w GBgb - 0 1 moves c1g1 c8b8")
    raw.isready()
    mv, _ = raw.bestmove("go depth 1")
    check("position accepts king-takes-rook castles (960 on)", len(mv) >= 4, mv)
    # Standard spelling is accepted as well
    raw.send("position fen r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves e1g1 e8c8")
    raw.isready()
    mv, _ = raw.bestmove("go depth 1")
    check("position accepts e1g1-style castles (960 on)", len(mv) >= 4, mv)
    # bestmove spelling: option on -> king takes rook
    raw.send("position fen 6kr/8/8/8/8/8/8/6KR w Hh - 0 1")
    mv, _ = raw.bestmove("go depth 1 searchmoves g1h1")
    check("bestmove spells an in-place castle as g1h1 (960 on)", mv == "g1h1", mv)
    raw.send("position fen r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
    mv, _ = raw.bestmove("go depth 1 searchmoves e1g1")
    check("bestmove spells a standard castle as e1h1 (960 on)", mv == "e1h1", mv)
    raw.send("setoption name UCI_Chess960 value false")
    mv, _ = raw.bestmove("go depth 1 searchmoves e1g1")
    check("bestmove spells a standard castle as e1g1 (960 off)", mv == "e1g1", mv)
    raw.send("position fen r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves e1h1")
    raw.isready()
    mv, _ = raw.bestmove("go depth 1")
    check("position accepts e1h1 even with 960 off", len(mv) >= 4, mv)

    # Clocked budgets (the lichess-bot path): hard cap is the engine's, +250 ms grace
    raw.send("setoption name UCI_Chess960 value true")
    raw.send("position fen bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9")
    mv, ms = raw.bestmove("go wtime 1000 btime 1000 movestogo 1")
    check("clocked go 1000ms movestogo 1 returns in time", ms < 1000 + 250, f"{mv} in {ms:.0f} ms")
    mv, ms = raw.bestmove("go wtime 10000 btime 10000 winc 100 binc 100", timeout=20)
    check("clocked go 10000ms returns in time", ms < 10000 + 250, f"{mv} in {ms:.0f} ms")
    # Ponder then ponderhit still produces a bestmove
    raw.send("position fen bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR b HFhf - 2 9")
    raw.send("go ponder wtime 3000 btime 3000")
    time.sleep(0.3)
    start = time.monotonic()
    raw.send("ponderhit")
    lines = raw.read_until("bestmove", 10)
    ms = (time.monotonic() - start) * 1000
    check("ponderhit yields bestmove", lines[-1].startswith("bestmove"), f"{lines[-1]} after {ms:.0f} ms")
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
