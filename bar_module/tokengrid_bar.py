#!/usr/bin/env python3
import sys
import time
import json
from pathlib import Path

# Add bin path to import daemon functions
sys.path.append(str(Path(__file__).resolve().parent.parent / "bin"))
try:
    import tokengrid_daemon
except ImportError:
    tokengrid_daemon = None

def render_bar():
    icon = "⚡"
    if tokengrid_daemon:
        summary = tokengrid_daemon.get_today_summary()
        tokens = summary["total_tokens"]
        cost = summary["total_cost"]
        
        # Format large numbers nicely
        if tokens >= 1_000_000:
            tok_str = f"{tokens / 1_000_000:.1f}M"
        elif tokens >= 1_000:
            tok_str = f"{tokens / 1_000:.1f}k"
        else:
            tok_str = str(tokens)

        text = f"{icon} {tok_str} | ${cost:.3f}"
        alt_text = f"TokenGrid: {tokens:,} tokens today (${cost:.4f})"
    else:
        text = f"{icon} TokenGrid"
        alt_text = "TokenGrid Daemon Disconnected"

    # Standard JSON payload supported by Omarchy / Waybar
    output = {
        "text": text,
        "tooltip": alt_text,
        "class": "tokengrid-active"
    }
    
    print(json.dumps(output))
    sys.stdout.flush()

if __name__ == "__main__":
    render_bar()
