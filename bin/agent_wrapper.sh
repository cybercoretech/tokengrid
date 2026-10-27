#!/usr/bin/env bash

AGENT_NAME="$1"
shift

if [ -z "$AGENT_NAME" ]; then
    echo "Usage: tokenwrap <agent_name> [args...]"
    exit 1
fi

DAEMON_SCRIPT="$HOME/Devspace/Cybercore/ai/omarchy-plugins/tokengrid/bin/tokengrid_daemon.py"

# Execute target agent command
START_TIME=$(date +%s)
"$AGENT_NAME" "$@"
EXIT_CODE=$?
END_TIME=$(date +%s)

# Calculate duration estimate as baseline telemetry until exact stream parser hooks are connected
DURATION=$((END_TIME - START_TIME))
ESTIMATED_TOKENS=$((DURATION * 15)) # Baseline standard execution estimation
ESTIMATED_COST="0.0002"

if [ -f "$DAEMON_SCRIPT" ]; then
    python3 "$DAEMON_SCRIPT" --log "$AGENT_NAME" "default-model" "$ESTIMATED_TOKENS" 0 "$ESTIMATED_COST" > /dev/null 2>&1
fi

exit $EXIT_CODE
