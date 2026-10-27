#!/usr/bin/env bash
# Trigger background metrics collection non-blocking
python3 "$(dirname "$0")/metrics-collector.py" &>/dev/null &

# Launch TokenGrid in an isolated Ghostty instance
exec ghostty --config-file="$HOME/.config/omarchy/plugins/io.github.cybercoretech.tokengrid/config/tokengrid.ghostty" -e tokengrid-tui
