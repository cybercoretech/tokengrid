#!/usr/bin/env bash
set -e

PROJECT_DIR="/home/raven/Devspace/Cybercore/ai/omarchy-plugins/tokengrid"
cd "$PROJECT_DIR/tui"

echo "[TokenGrid] Building Rust TUI binary..."
cargo build --release

# Locate binary in global cargo-target or local target directory
if [ -f "/home/raven/.cargo-target/release/tokengrid-tui" ]; then
    cp /home/raven/.cargo-target/release/tokengrid-tui "$PROJECT_DIR/bin/tokengrid-tui"
elif [ -f "target/release/tokengrid-tui" ]; then
    cp target/release/tokengrid-tui "$PROJECT_DIR/bin/tokengrid-tui"
else
    echo "Error: tokengrid-tui binary not found in target directories."
    exit 1
fi

chmod +x "$PROJECT_DIR/bin/tokengrid-tui"

echo "[TokenGrid] TUI built successfully -> $PROJECT_DIR/bin/tokengrid-tui"
