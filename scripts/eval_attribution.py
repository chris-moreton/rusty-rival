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
from typing import NamedTuple


class SearchResult(NamedTuple):
    score: int
    bestmove: str
    pv: str


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

    def new_game(self) -> None:
        """Reset engine heuristics before measuring an independent position."""
        self.send("ucinewgame")
        self.send("isready")
        self.read_until(lambda line: line == "readyok")


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


def parse_exact_score(
    lines: list[str], white_stm: bool, *, nodes: int | None = None, depth: int | None = None
) -> int:
    """Return the final exact score after validating the requested search limit."""
    score_lines = [
        line
        for line in lines
        if re.search(r"\bscore cp -?\d+", line)
        and " lowerbound" not in line
        and " upperbound" not in line
    ]
    if not score_lines:
        raise RuntimeError("search produced no centipawn score")
    score = re.search(r"\bscore cp (-?\d+)", score_lines[-1])
    if score is None:
        raise RuntimeError("could not parse final exact search score")
    if nodes is not None:
        searched_nodes = re.search(r"\bnodes (\d+)", score_lines[-1])
        if searched_nodes is None:
            raise RuntimeError("final search score did not report its node count")
        if int(searched_nodes.group(1)) < nodes:
            raise RuntimeError(
                f"final exact score was reported after {searched_nodes.group(1)} nodes; "
                f"expected at least {nodes}"
            )
    if depth is not None:
        searched_depth = re.search(r"\bdepth (\d+)", score_lines[-1])
        if searched_depth is None:
            raise RuntimeError("final search score did not report its depth")
        if int(searched_depth.group(1)) < depth:
            raise RuntimeError(
                f"final exact score was reported at depth {searched_depth.group(1)}; "
                f"expected at least {depth}"
            )
    stm_score = int(score.group(1))
    return stm_score if white_stm else -stm_score


def searched_score(
    engine: UciEngine, white_stm: bool, *, nodes: int | None = None, depth: int | None = None
) -> SearchResult:
    if (nodes is None) == (depth is None):
        raise RuntimeError(
            "searched_score requires exactly one of nodes or depth"
        )
    engine.send("setoption name Clear Hash")
    limit = f"nodes {nodes}" if nodes is not None else f"depth {depth}"
    engine.send(f"go {limit}")
    lines = engine.read_until(lambda line: line.startswith("bestmove "))
    score = parse_exact_score(lines, white_stm, nodes=nodes, depth=depth)
    exact_lines = [
        line
        for line in lines
        if re.search(r"\bscore cp -?\d+", line)
        and " lowerbound" not in line
        and " upperbound" not in line
    ]
    pv_match = re.search(r"\bpv (.+)$", exact_lines[-1])
    bestmove_parts = lines[-1].split()
    bestmove = bestmove_parts[1] if len(bestmove_parts) > 1 else ""
    return SearchResult(score, bestmove, pv_match.group(1) if pv_match else "")


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
    limits = parser.add_mutually_exclusive_group()
    limits.add_argument("--nodes", type=positive_int)
    limits.add_argument("--depth", type=positive_int)
    args = parser.parse_args()
    if args.nodes is None and args.depth is None:
        args.nodes = 200_000

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
                "rival_bestmove",
                "rival_pv",
                "stockfish_static_cp",
                "stockfish_search_cp",
                "stockfish_bestmove",
                "stockfish_pv",
                "raw_nnue_gap_cp",
                "search_gap_cp",
            ]
        )
        for label, position in load_positions(args.positions):
            white_stm = white_to_move(position)
            rival.new_game()
            stockfish.new_game()
            rival.send(position)
            stockfish.send(position)
            rr_nnue = rival_raw(rival, True)
            rr_hce = rival_raw(rival, False)
            rival.send("setoption name UseNNUE value true")
            rr_search = searched_score(rival, white_stm, nodes=args.nodes, depth=args.depth)
            sf_static = stockfish_static(stockfish)
            sf_search = searched_score(stockfish, white_stm, nodes=args.nodes, depth=args.depth)
            writer.writerow(
                [
                    label,
                    rr_nnue,
                    rr_hce,
                    rr_search.score,
                    rr_search.bestmove,
                    rr_search.pv,
                    sf_static,
                    sf_search.score,
                    sf_search.bestmove,
                    sf_search.pv,
                    rr_nnue - sf_static,
                    rr_search.score - sf_search.score,
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
