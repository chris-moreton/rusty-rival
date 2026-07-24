#!/bin/bash
# A/B match between two rusty-rival binaries via cutechess-cli.
#
#   ./scripts/ab_match.sh <baseline-bin> <candidate-bin> [games] [tc] [concurrency]
#
# Defaults to a fast TC suitable for detecting ~10 Elo in a couple of hours.
# Prints the cutechess result line; feed the PGN to ordo for a rating if wanted.
set -u

BASE="${1:?usage: ab_match.sh <baseline-bin> <candidate-bin> [games] [tc] [concurrency]}"
CAND="${2:?need candidate binary}"
GAMES="${3:-1000}"
TC="${4:-10+0.1}"
CONC="${5:-4}"

BOOK="$HOME/git/chris-moreton/chess/chess-compete/openings/8moves_v3.pgn"
OUT="$HOME/git/chris-moreton/chess/rusty-rival/results/ab"
mkdir -p "$OUT"
STAMP=$(date +%Y%m%d-%H%M%S)
PGN="$OUT/ab_${STAMP}.pgn"

for f in "$BASE" "$CAND"; do
  [ -x "$f" ] || { echo "not executable: $f" >&2; exit 1; }
done
[ -f "$BOOK" ] || { echo "book not found: $BOOK" >&2; exit 1; }

echo "baseline : $BASE"
echo "candidate: $CAND"
echo "games=$GAMES tc=$TC concurrency=$CONC"
echo "pgn      : $PGN"

# -repeat pairs colours so each opening is played both ways: halves colour-bias variance.
# 1 thread + fixed hash each, so the comparison isolates the code change.
cutechess-cli \
  -engine name=base cmd="$BASE" proto=uci option.Threads=1 option.Hash=128 \
  -engine name=cand cmd="$CAND" proto=uci option.Threads=1 option.Hash=128 \
  -each tc="$TC" timemargin=200 \
  -openings file="$BOOK" format=pgn order=random plies=16 \
  -repeat -games 2 -rounds $((GAMES / 2)) \
  -concurrency "$CONC" \
  -draw movenumber=40 movecount=8 score=10 \
  -resign movecount=3 score=600 \
  -pgnout "$PGN" \
  -ratinginterval 50 2>&1 | tee "$OUT/ab_${STAMP}.log"

echo
echo "PGN: $PGN"
