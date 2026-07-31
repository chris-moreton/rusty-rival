#!/bin/bash
# Time-control A/B match: like ab_match.sh, but configured to expose CLOCK bugs
# rather than playing strength (NET-339).
#
#   ./scripts/ab_time_match.sh <baseline-bin> <candidate-bin> [games] [tc] [threads]
#
# Differences from ab_match.sh, all deliberate — ab_match.sh could not have
# caught the v1.0.48 time forfeits:
#
#   * Threads defaults to 8, not 1. Overspend of the soft time limit scales with
#     thread count (measured 2.13x at 1 thread vs 2.88x at 8), and the Lichess bot
#     runs 8. A 1-thread match tests a materially less-affected engine.
#   * Hash 512 and Move Overhead 0, matching the bot's config.yml exactly.
#   * concurrency is FIXED AT 1. Running games in parallel starves the engines of
#     cores and produces time losses that are artefacts of the test box, not the
#     engine. A timing test cannot share a CPU.
#   * timemargin is small (100ms), not 200ms, so genuine overruns are not absorbed.
#   * Default TC is 60+0 — no increment. An increment replenishes the clock every
#     move and masks systematic overspend; the budget is recomputed from the time
#     REMAINING each move, so usage decays geometrically and a flag may never
#     arrive. Zero increment is both the realistic Lichess bullet control and the
#     condition under which overspend actually forfeits.
#
# The headline number here is the time-loss count, printed at the end. Elo is
# secondary — a strength gain is worthless if the engine flags.
set -u

BASE="${1:?usage: ab_time_match.sh <baseline-bin> <candidate-bin> [games] [tc] [threads]}"
CAND="${2:?need candidate binary}"
GAMES="${3:-200}"
TC="${4:-60+0}"
THREADS="${5:-8}"

BOOK="$HOME/git/chris-moreton/chess/chess-compete/openings/8moves_v3.pgn"
OUT="$HOME/git/chris-moreton/chess/rusty-rival/results/ab"
mkdir -p "$OUT"
STAMP=$(date +%Y%m%d-%H%M%S)
PGN="$OUT/abtime_${STAMP}.pgn"
LOG="$OUT/abtime_${STAMP}.log"

for f in "$BASE" "$CAND"; do
  [ -x "$f" ] || { echo "not executable: $f" >&2; exit 1; }
done
[ -f "$BOOK" ] || { echo "book not found: $BOOK" >&2; exit 1; }

echo "baseline : $BASE"
echo "candidate: $CAND"
echo "games=$GAMES tc=$TC threads=$THREADS concurrency=1 (fixed)"
echo "pgn      : $PGN"
echo

cutechess-cli \
  -engine name=base cmd="$BASE" proto=uci option.Threads="$THREADS" option.Hash=512 "option.Move Overhead=0" \
  -engine name=cand cmd="$CAND" proto=uci option.Threads="$THREADS" option.Hash=512 "option.Move Overhead=0" \
  -each tc="$TC" timemargin=100 \
  -openings file="$BOOK" format=pgn order=random plies=16 \
  -repeat -games 2 -rounds $((GAMES / 2)) \
  -concurrency 1 \
  -pgnout "$PGN" \
  -ratinginterval 20 2>&1 | tee "$LOG"

echo
echo "================ TIME LOSSES ================"
# cutechess reports these as "loses on time" in the result line for each game.
BASE_FLAGS=$(grep -c "base loses on time" "$LOG" || true)
CAND_FLAGS=$(grep -c "cand loses on time" "$LOG" || true)
echo "base flagged: $BASE_FLAGS"
echo "cand flagged: $CAND_FLAGS"
echo
grep -E "loses on time|Elo difference" "$LOG" | tail -20
echo
echo "PGN: $PGN"
echo "LOG: $LOG"
