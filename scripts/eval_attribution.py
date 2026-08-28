#!/usr/bin/env python3
"""Compare Rival raw NNUE/HCE and searched scores with Stockfish.

The input is a tab-separated label and UCI ``position`` command. Both engines
are kept single-threaded and searched for identical node counts so the result
separates evaluator output from search as far as the public protocols allow.
"""

import argparse
import csv
import re
import subprocess
import sys
from pathlib import Path


class UciEngine:
    def __init__(self, path: str) -> None:
        self.process = subprocess.Popen(
            [path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        assert self.process.stdin is not None
        assert self.process.stdout is not None
        try:
            self.send("uci")
            self.read_until(lambda line: line == "uciok")
            self.send("setoption name Threads value 1")
            self.send("setoption name Hash value 64")
            self.send("isready")
            self.read_until(lambda line: line == "readyok")
        except BaseException:
            try:
                self.close()
            except Exception:
                pass
            raise

    def send(self, command: str) -> None:
        assert self.process.stdin is not None
        self.process.stdin.write(command + "\n")
        self.process.stdin.flush()

    def read_until(self, done) -> list[str]:
        assert self.process.stdout is not None
        lines = []
        for raw in self.process.stdout:
            line = raw.strip()
            lines.append(line)
            if done(line):
                return lines
        raise RuntimeError("engine exited before completing command")

    def close(self) -> None:
        if self.process.poll() is not None:
            return
        try:
            self.send("quit")
            self.process.wait(timeout=5)
        except (BrokenPipeError, subprocess.TimeoutExpired):
            if self.process.poll() is not None:
                return
            self.process.terminate()
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=2)


def load_positions(path: Path) -> list[tuple[str, str]]:
    positions = []
    for number, raw in enumerate(path.read_text().splitlines(), 1):
        if not raw or raw.startswith("#"):
            continue
        try:
            label, command = raw.split("\t", 1)
        except ValueError as exc:
            raise ValueError(f"{path}:{number}: expected label<TAB>position command") from exc
        if command.startswith("position startpos"):
            pass
        elif command.startswith("position fen "):
            fen_text = command.partition("position fen ")[2].partition(" moves ")[0]
            fen = fen_text.split()
            if len(fen) not in (4, 6) or fen[1] not in ("w", "b"):
                raise ValueError(f"{path}:{number}: malformed FEN position command")
        else:
            raise ValueError(f"{path}:{number}: expected a UCI position command")
        positions.append((label, command))
    return positions


def white_to_move(position_command: str) -> bool:
    if " startpos" in position_command:
        moves = position_command.partition(" moves ")[2].split()
        return len(moves) % 2 == 0
    fen_and_moves = position_command.partition(" fen ")[2]
    fen = fen_and_moves.partition(" moves ")[0].split()
    moves = fen_and_moves.partition(" moves ")[2].split()
    return len(fen) >= 2 and (fen[1] == "w") == (len(moves) % 2 == 0)


def rival_raw(engine: UciEngine, use_nnue: bool) -> int:
    engine.send(f"setoption name UseNNUE value {'true' if use_nnue else 'false'}")
    engine.send("eval")
    lines = engine.read_until(lambda line: "info string eval raw cp" in line)
    match = re.search(r"white_cp (-?\d+)", lines[-1])
    if match is None:
        raise RuntimeError("could not parse Rival raw evaluation")
    expected_evaluator = "nnue" if use_nnue else "hce"
    if not lines[-1].endswith(f"evaluator {expected_evaluator}"):
        raise RuntimeError(f"Rival did not activate {expected_evaluator}")
    return int(match.group(1))


def searched_score(engine: UciEngine, nodes: int, white_stm: bool) -> int:
    engine.send("setoption name Clear Hash")
    engine.send(f"go nodes {nodes}")
    lines = engine.read_until(lambda line: line.startswith("bestmove "))
    score_lines = [line for line in lines if re.search(r"\bscore cp -?\d+", line)]
    if not score_lines:
        raise RuntimeError("search produced no centipawn score")
    score = re.search(r"\bscore cp (-?\d+)", score_lines[-1])
    searched_nodes = re.search(r"\bnodes (\d+)", score_lines[-1])
    if score is None or searched_nodes is None:
        raise RuntimeError("final search score did not report its node count")
    if int(searched_nodes.group(1)) < nodes:
        raise RuntimeError(
            f"search stopped after {searched_nodes.group(1)} nodes; expected at least {nodes}"
        )
    stm_score = int(score.group(1))
    return stm_score if white_stm else -stm_score


def stockfish_static(engine: UciEngine) -> int:
    engine.send("eval")
    lines = engine.read_until(lambda line: line.startswith("Final evaluation"))
    match = re.search(r"Final evaluation\s+([+-]?\d+(?:\.\d+)?) \(white side\)", lines[-1])
    if match is None:
        raise RuntimeError("could not parse Stockfish static evaluation")
    return round(float(match.group(1)) * 100)


def positive_int(value: str) -> int:
    parsed = int(value)
    if parsed < 1:
        raise argparse.ArgumentTypeError("must be at least 1")
    return parsed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rival", required=True)
    parser.add_argument("--stockfish", required=True)
    parser.add_argument("--positions", type=Path, required=True)
    parser.add_argument("--nodes", type=positive_int, default=200_000)
    args = parser.parse_args()

    rival = None
    stockfish = None
    try:
        rival = UciEngine(args.rival)
        stockfish = UciEngine(args.stockfish)
        writer = csv.writer(sys.stdout)
        writer.writerow(
            [
                "label",
                "rival_nnue_cp",
                "rival_hce_cp",
                "rival_search_cp",
                "stockfish_static_cp",
                "stockfish_search_cp",
                "raw_nnue_gap_cp",
                "search_gap_cp",
            ]
        )
        for label, position in load_positions(args.positions):
            white_stm = white_to_move(position)
            rival.send(position)
            stockfish.send(position)
            rr_nnue = rival_raw(rival, True)
            rr_hce = rival_raw(rival, False)
            rival.send("setoption name UseNNUE value true")
            rr_search = searched_score(rival, args.nodes, white_stm)
            sf_static = stockfish_static(stockfish)
            sf_search = searched_score(stockfish, args.nodes, white_stm)
            writer.writerow(
                [
                    label,
                    rr_nnue,
                    rr_hce,
                    rr_search,
                    sf_static,
                    sf_search,
                    rr_nnue - sf_static,
                    rr_search - sf_search,
                ]
            )
            sys.stdout.flush()
    finally:
        try:
            if rival is not None:
                rival.close()
        finally:
            if stockfish is not None:
                stockfish.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
