#!/usr/bin/env bash
# Trigger background metrics collection non-blocking
python3 "$(dirname "$0")/metrics-collector.py" &>/dev/null &

# Resolve absolute path to binary inside the plugin directory
PLUGIN_DIR="$HOME/.config/omarchy/plugins/io.github.cybercoretech.tokengrid"
BIN_PATH="$PLUGIN_DIR/bin/tokengrid-tui"

# Launch Ghostty using the explicit binary executable path
exec ghostty --config-file="$PLUGIN_DIR/config/tokengrid.ghostty" -e "$BIN_PATH"
