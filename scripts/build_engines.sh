#!/bin/bash
# Build all tagged engine versions for Docker
# This script is run inside the Docker build container

set -e

REPO_URL="https://github.com/chris-moreton/rusty-rival.git"
ENGINES_DIR="/engines"

# Tags to build (from v008 onwards - earlier versions have bugs)
TAGS=(
    "v008-make-unmake-fixed"
    "v009-unmake-opt"
    "v010-arrayvec-movelist"
    "v011-check-evasion"
    "v012-bishop-pair"
    "v013-knight-fork"
    "v014-rook-open-file"
    "v015-rook-mobility"
    "v016-queen-mobility"
    "v017-connected-rooks"
    "v018-killer-fix"
    "v019-capture-fix"
    "v020-remove-harmful-eval"
    "v021-wrong-color-bishop"
    "v022-king-activity"
    "v023-kpk-draw"
    "v024-connected-passed-pawns"
    "v026-tablebase-perf"
    "v027-tablebase-fast"
    "v028-dtz-progress"
    "v029-log-lmr"
    "v031-delta-pruning"
    "v032-history-lmr"
    "v033-trade-bonus"
    "v034-threat-lmr"
    "v035-pawn-push-ext"
    "v036-frac-ext"
    "v037-optimized"
)

# Clone repo once (full clone to access all tags)
echo "Cloning repository..."
git clone --depth 1 "$REPO_URL" /repo-base
cd /repo-base
git fetch --tags

for TAG in "${TAGS[@]}"; do
    echo "========================================"
    echo "Building $TAG..."
    echo "========================================"

    # Create build directory
    BUILD_DIR="/build-$TAG"
    rm -rf "$BUILD_DIR"

    # Clone at specific tag
    git clone --depth 1 --branch "$TAG" "$REPO_URL" "$BUILD_DIR" || {
        echo "Warning: Tag $TAG not found, skipping..."
        continue
    }

    cd "$BUILD_DIR"

    # Apply history.txt fix to prevent crash on startup
    sed -i 's/rl\.load_history("history\.txt")\.unwrap();/let _ = rl.load_history("history.txt");/' src/main.rs 2>/dev/null || true

    # Build
    cargo build --release || {
        echo "Warning: Build failed for $TAG, skipping..."
        cd /
        rm -rf "$BUILD_DIR"
        continue
    }

    # Copy binary
    mkdir -p "$ENGINES_DIR/$TAG"
    cp target/release/rusty-rival "$ENGINES_DIR/$TAG/"

    # Cleanup to save space
    cd /
    rm -rf "$BUILD_DIR"

    echo "Successfully built $TAG"
done

echo "========================================"
echo "All builds complete!"
ls -la "$ENGINES_DIR"
echo "========================================"
