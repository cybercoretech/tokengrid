#!/usr/bin/env python3
import os
import json
import time
import psutil
from datetime import datetime, timedelta

DATA_DIR = os.path.expanduser("~/.local/share/omarchy/tokengrid/metrics")
os.makedirs(DATA_DIR, exist_ok=True)

DAILY_FILE = os.path.join(DATA_DIR, "daily.json")
WEEKLY_FILE = os.path.join(DATA_DIR, "weekly.json")
MONTHLY_FILE = os.path.join(DATA_DIR, "monthly.json")

def get_system_metrics():
    cpu = psutil.cpu_percent(interval=1)
    mem = psutil.virtual_memory().percent
    disk = psutil.disk_usage('/').percent
    net = psutil.net_io_counters()
    return {
        "timestamp": datetime.now().isoformat(),
        "cpu_percent": cpu,
        "memory_percent": mem,
        "disk_percent": disk,
        "bytes_sent": net.bytes_sent,
        "bytes_recv": net.bytes_recv
    }

def update_aggregates(new_metric):
    now = datetime.now()
    
    # Helper to update JSON metric logs
    def update_log(filepath, max_entries, key_group_fn):
        data = []
        if os.path.exists(filepath):
            try:
                with open(filepath, "r") as f:
                    data = json.load(f)
            except Exception:
                data = []
        
        data.append(new_metric)
        # Keep recent history trimmed
        if len(data) > max_entries:
            data = data[-max_entries:]
            
        with open(filepath, "w") as f:
            json.dump(data, f, indent=2)

    # Update daily (last 24 hours / hourly points), weekly (last 7 days), monthly (last 30 days)
    update_log(DAILY_FILE, 24, lambda dt: dt.strftime("%Y-%m-%d %H:00"))
    update_log(WEEKLY_FILE, 168, lambda dt: dt.strftime("%Y-%m-%d"))
    update_log(MONTHLY_FILE, 720, lambda dt: dt.strftime("%Y-%m-%d"))

if __name__ == "__main__":
    metric = get_system_metrics()
    update_aggregates(metric)
    print(json.dumps(metric, indent=2))
