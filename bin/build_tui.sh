#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
MANIFEST="$PROJECT_DIR/tui/Cargo.toml"
INSTALL_DIR="$HOME/.config/omarchy/plugins/io.github.cybercoretech.tokengrid"

MODE="${1:-build}"
if [[ "$MODE" != "build" && "$MODE" != "--install" ]]; then
    echo "Usage: $0 [--install]" >&2
    exit 2
fi

echo "[TokenGrid] Building TUI..."
cargo build --release --manifest-path "$MANIFEST"

TARGET_DIR="$(
    cargo metadata --manifest-path "$MANIFEST" --no-deps --format-version 1 |
    python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])'
)"
BUILT_BIN="$TARGET_DIR/release/tokengrid-tui"

if [[ ! -x "$BUILT_BIN" ]]; then
    echo "[TokenGrid] ERROR: expected executable not found: $BUILT_BIN" >&2
    exit 1
fi

install -D -m 0755 "$BUILT_BIN" "$PROJECT_DIR/bin/tokengrid-tui"
echo "[TokenGrid] Build artifact updated: $PROJECT_DIR/bin/tokengrid-tui"

if [[ "$MODE" == "--install" ]]; then
    if [[ ! -d "$INSTALL_DIR" ]]; then
        echo "[TokenGrid] ERROR: plugin install directory does not exist: $INSTALL_DIR" >&2
        echo "[TokenGrid] Install the plugin first; no installed files were changed." >&2
        exit 1
    fi

    install -D -m 0755 "$BUILT_BIN" "$INSTALL_DIR/bin/tokengrid-tui"
    echo "[TokenGrid] Installed TUI: $INSTALL_DIR/bin/tokengrid-tui"
fi
