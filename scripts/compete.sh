#!/bin/bash
# Engine vs Engine competition wrapper for cutechess-cli
# Usage: ./scripts/compete.sh <engine1> <engine2> [games] [time_per_move]
#
# Examples:
#   ./scripts/compete.sh v1-baseline v2-weak-queen          # 100 games, 1s/move
#   ./scripts/compete.sh v1-baseline v2-weak-queen 50       # 50 games, 1s/move
#   ./scripts/compete.sh v1-baseline v2-weak-queen 100 0.5  # 100 games, 0.5s/move

set -e

ENGINE_DIR="$(dirname "$0")/../engines"
RESULTS_DIR="$(dirname "$0")/../results/competitions"

# Arguments
ENGINE1="${1:?Usage: $0 <engine1> <engine2> [games] [time_per_move]}"
ENGINE2="${2:?Usage: $0 <engine1> <engine2> [games] [time_per_move]}"
GAMES="${3:-100}"
TIME_PER_MOVE="${4:-1}"

# Validate engines exist
if [[ ! -x "$ENGINE_DIR/$ENGINE1/rusty-rival" ]]; then
    echo "Error: Engine not found or not executable: $ENGINE_DIR/$ENGINE1/rusty-rival"
    exit 1
fi

if [[ ! -x "$ENGINE_DIR/$ENGINE2/rusty-rival" ]]; then
    echo "Error: Engine not found or not executable: $ENGINE_DIR/$ENGINE2/rusty-rival"
    exit 1
fi

# Create results directory if needed
mkdir -p "$RESULTS_DIR"

# Generate timestamp for unique filenames
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
PGN_FILE="$RESULTS_DIR/${ENGINE1}_vs_${ENGINE2}_${TIMESTAMP}.pgn"

echo "========================================"
echo "Engine Competition"
echo "========================================"
echo "Engine 1: $ENGINE1"
echo "Engine 2: $ENGINE2"
echo "Games:    $GAMES"
echo "Time:     ${TIME_PER_MOVE}s per move"
echo "PGN:      $PGN_FILE"
echo "========================================"
echo ""

cutechess-cli \
    -engine name="$ENGINE1" cmd="$ENGINE_DIR/$ENGINE1/rusty-rival" \
    -engine name="$ENGINE2" cmd="$ENGINE_DIR/$ENGINE2/rusty-rival" \
    -each proto=uci tc=0/0:${TIME_PER_MOVE} \
    -games "$GAMES" \
    -pgnout "$PGN_FILE" \
    -recover \
    -repeat \
    -resign movecount=3 score=500 \
    -draw movenumber=40 movecount=8 score=10

echo ""
echo "========================================"
echo "Competition complete!"
echo "PGN saved to: $PGN_FILE"
echo "========================================"
