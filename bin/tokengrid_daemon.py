#!/usr/bin/env python3
import os
import sys
import sqlite3
from pathlib import Path

DB_PATH = Path(
    os.environ.get(
        "TOKENGRID_DB_PATH",
        str(
            Path.home()
            / ".config/omarchy/plugins/io.github.cybercoretech.tokengrid/data/tokengrid.db"
        ),
    )
).expanduser()

INITIAL_AGENTS = [
    "gemini", "claude", "crush", "cursor-agent", "codex", "copilot", 
    "oh-my-pi", "pi", "opencode", "grok", "ori", "kimi", "muse", 
    "antigravity-cli", "qwen", "devin", "hermes", "hermes-agent", "kilocode"
]

def init_db():
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    
    # Token usage history table
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS token_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            agent_name TEXT NOT NULL,
            model_name TEXT,
            prompt_tokens INTEGER DEFAULT 0,
            completion_tokens INTEGER DEFAULT 0,
            cost_est REAL DEFAULT 0.0,
            measurement_type TEXT NOT NULL DEFAULT 'unknown'
        )
    """)

    # Add provenance to existing databases without rewriting usage records.
    columns = {
        row[1] for row in cursor.execute("PRAGMA table_info(token_usage)")
    }
    if "measurement_type" not in columns:
        cursor.execute(
            "ALTER TABLE token_usage "
            "ADD COLUMN measurement_type TEXT NOT NULL DEFAULT 'unknown'"
        )

    # Agents configuration table
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS agents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            agent_name TEXT UNIQUE NOT NULL,
            model_default TEXT DEFAULT 'default',
            budget_limit REAL DEFAULT 10.0
        )
    """)

    # Pre-seed initial 17+ agents immediately
    for agent in INITIAL_AGENTS:
        cursor.execute("INSERT OR IGNORE INTO agents (agent_name, model_default, budget_limit) VALUES (?, 'default', 10.0)", (agent,))

    conn.commit()
    conn.close()

def log_event(
    agent_name, model_name, prompt_tokens, completion_tokens,
    cost_est=0.0, measurement_type="measured"
):
    if measurement_type not in {"measured", "estimated", "unknown"}:
        raise ValueError("Invalid measurement_type")
    if prompt_tokens < 0 or completion_tokens < 0 or cost_est < 0:
        raise ValueError("Tokens and cost must not be negative")

    init_db()
    with sqlite3.connect(DB_PATH, timeout=10) as conn:
        conn.execute("""
            INSERT INTO token_usage (
                agent_name, model_name, prompt_tokens, completion_tokens,
                cost_est, measurement_type
            ) VALUES (?, ?, ?, ?, ?, ?)
        """, (
            agent_name, model_name, prompt_tokens, completion_tokens,
            cost_est, measurement_type
        ))

def get_today_summary():
    init_db()
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("""
        SELECT 
            SUM(prompt_tokens + completion_tokens),
            SUM(cost_est)
        FROM token_usage
        WHERE date(timestamp) = date('now')
    """)
    row = cursor.fetchone()
    conn.close()
    
    total_tokens = row[0] if row[0] is not None else 0
    total_cost = row[1] if row[1] is not None else 0.0
    return {"total_tokens": total_tokens, "total_cost": total_cost}

if __name__ == "__main__":
    init_db()
    if len(sys.argv) > 1 and sys.argv[1] in ("--log", "--log-estimate"):
        if len(sys.argv) not in (6, 7):
            print(
                "Usage: tokengrid_daemon.py [--log|--log-estimate] "
                "AGENT MODEL PROMPT_TOKENS COMPLETION_TOKENS [COST]",
                file=sys.stderr,
            )
            sys.exit(2)

        agent = sys.argv[2]
        model = sys.argv[3]
        p_tok = int(sys.argv[4])
        c_tok = int(sys.argv[5])
        cost = float(sys.argv[6]) if len(sys.argv) == 7 else 0.0
        kind = "estimated" if sys.argv[1] == "--log-estimate" else "measured"
        log_event(agent, model, p_tok, c_tok, cost, kind)
        print(f"[TokenGrid] Logged {kind} usage for {agent}")
    else:
        init_db()
        with sqlite3.connect(DB_PATH) as conn:
            row = conn.execute("""
                SELECT
                    COALESCE(SUM(CASE WHEN measurement_type='measured'
                        THEN prompt_tokens + completion_tokens ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN measurement_type='measured'
                        THEN cost_est ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN measurement_type='estimated'
                        THEN prompt_tokens + completion_tokens ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN measurement_type='estimated'
                        THEN cost_est ELSE 0 END), 0)
                FROM token_usage WHERE date(timestamp)=date('now')
            """).fetchone()
        print(f"Measured: {row[0]} tok | ${row[1]:.4f}")
        print(f"Estimated: {row[2]} tok | ${row[3]:.4f}")
        sys.stdout.flush()
