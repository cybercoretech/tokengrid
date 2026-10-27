#!/usr/bin/env python3
import os
import sys
import sqlite3
from pathlib import Path

DB_PATH = Path.home() / "Devspace/Cybercore/ai/omarchy-plugins/tokengrid/data/tokengrid.db"

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
            cost_est REAL DEFAULT 0.0
        )
    """)

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

def log_event(agent_name, model_name, prompt_tokens, completion_tokens, cost_est=0.0):
    init_db()
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("""
        INSERT INTO token_usage (agent_name, model_name, prompt_tokens, completion_tokens, cost_est)
        VALUES (?, ?, ?, ?, ?)
    """, (agent_name, model_name, prompt_tokens, completion_tokens, cost_est))
    conn.commit()
    conn.close()

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
    if len(sys.argv) > 1 and sys.argv[1] == "--log":
        if len(sys.argv) >= 6:
            agent = sys.argv[2]
            model = sys.argv[3]
            p_tok = int(sys.argv[4])
            c_tok = int(sys.argv[5])
            cost = float(sys.argv[6]) if len(sys.argv) > 6 else 0.0
            log_event(agent, model, p_tok, c_tok, cost)
            print(f"[TokenGrid] Logged {p_tok + c_tok} tokens for {agent}")
    else:
        summary = get_today_summary()
        print(f"⚡ {summary['total_tokens']} tok | ${summary['total_cost']:.4f}")
        sys.stdout.flush()
