#!/usr/bin/env python3
"""Verify docs/index.html still tells the truth about the source.

Every factual claim on the GitHub Pages site is derived from code in this
repo. Each claim pairs a SOURCE pattern (the fact, in tui/, bin/, helpers/ …)
with a PAGE pattern (how the site documents it, in docs/index.html):

  source pattern missing  -> the fact changed upstream; review the page copy
  page pattern missing    -> the copy was edited away; restore or reword it

Dynamic claims are derived from the source itself (roster size, table count,
mashup count), so the numbers on the page are checked against reality rather
than hard-coded twice.

Run:  python3 scripts/check_copy.py          (exit 0 = clean, 1 = drift)
CI runs this on every push/PR touching tui/, bin/, helpers/, config/, lua/,
bar_module/, BarWidget.qml or docs/index.html.
"""

from __future__ import annotations

import re
import sys
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAGE_FILE = ROOT / "docs" / "index.html"

SRC_TUI = "tui/src/main.rs"
SRC_DAEMON = "bin/tokengrid_daemon.py"
SRC_WRAPPER = "bin/agent_wrapper.sh"
SRC_BUILD = "bin/build_tui.sh"
SRC_ALIASES = "config/tokengrid_aliases.zsh"
SRC_GHOSTTY = "config/tokengrid.ghostty"
SRC_LUA = "lua/tokengrid-rules.lua"
SRC_QML = "BarWidget.qml"
SRC_PAYLOAD = "bar_module/tokengrid_bar.py"
SRC_METRICS = "helpers/metrics-collector.py"

_cache: dict[str, str] = {}


def read(rel: str) -> str:
    if rel not in _cache:
        path = ROOT / rel
        if not path.is_file():
            raise FileNotFoundError(rel)
        _cache[rel] = path.read_text(encoding="utf-8")
    return _cache[rel]


class Claim:
    def __init__(self, key: str, section: str, src: str, src_re: str,
                 page_re: str, note: str):
        self.key = key
        self.section = section
        self.src = src
        self.src_re = src_re
        self.page_re = page_re
        self.note = note


def build_claims() -> list[Claim]:
    claims = [
        # --- 03 / The TUI: screens and keys (tui/src/main.rs) ---
        Claim("tui-hub-title", "03 TUI", SRC_TUI,
              r'\.title\(" TokenGrid Telemetry Hub "\)',
              r"┌ TokenGrid Telemetry Hub",
              "hub screen title"),
        Claim("tui-daily-tokens", "03 TUI", SRC_TUI,
              r"Today's Total Tokens:",
              r"Today's Total Tokens: 412650",
              "Daily Overview totals line"),
        Claim("tui-monitored-agents", "03 TUI", SRC_TUI,
              r"Monitored Agents:",
              r"Monitored Agents: 19",
              "Daily Overview agent count"),
        Claim("tui-table-cols", "03 TUI", SRC_TUI,
              r'"Agent Name"',
              r"name, model, budget, tokens and cost today",
              "agent table columns"),
        Claim("tui-keys", "03 TUI", SRC_TUI,
              r"\[j/k: navigate, Enter: details, a: add, d: delete, s: settings, q: quit\]",
              r"<code>j/k</code> to move, Enter to open",
              "keybinding row"),
        Claim("tui-vim-footer", "03 TUI", SRC_TUI,
              r"Vim Keys: \[j/k\] Navigate",
              r"vim keys take you everywhere else",
              "footer key help"),
        Claim("tui-detail-back", "03 TUI", SRC_TUI,
              r"\[Esc/Enter: Back\]",
              r"Drill-down",
              "detail view + back key"),
        Claim("tui-detail-today", "03 TUI", SRC_TUI,
              r"TODAY \(measured\)",
              r"today's measured tokens",
              "detail: today (measured)"),
        Claim("tui-detail-week", "03 TUI", SRC_TUI,
              r"7-DAY \(measured\)",
              r"today and 7-day totals",
              "detail: 7-day (measured)"),
        Claim("tui-recent-8", "03 TUI", SRC_TUI,
              r"LIMIT 8",
              r"last eight runs",
              "detail: last 8 runs"),
        Claim("tui-add-form", "03 TUI", SRC_TUI,
              r"REGISTER NEW AI AGENT",
              r"add an agent",
              "add-agent form"),
        Claim("tui-delete-confirm", "03 TUI", SRC_TUI,
              r"Type the exact name to delete",
              r"delete one by typing its name",
              "delete confirmation"),
        Claim("tui-settings-path", "03 TUI", SRC_TUI,
              r"Active Database Path",
              r"settings for the DB path",
              "settings: DB path"),
        Claim("tui-settings-cap", "03 TUI", SRC_TUI,
              r"Global Daily Budget Cap: \$50\.00",
              r"the \$50 daily cap and webhook",
              "settings: global cap"),
        Claim("tui-settings-webhook", "03 TUI", SRC_TUI,
              r"Webhook Alerts URL",
              r"the \$50 daily cap and webhook",
              "settings: webhook alerts"),

        # --- 01 / Pipeline: provenance ---
        Claim("prov-column", "01 Pipeline", SRC_DAEMON,
              r"measurement_type TEXT NOT NULL DEFAULT 'unknown'",
              r"measurement_type",
              "provenance column"),
        Claim("prov-values", "01 Pipeline", SRC_DAEMON,
              r'measurement_type not in \{"measured", "estimated", "unknown"\}',
              r"tagged measured or estimated",
              "allowed provenance values"),
        Claim("prov-tui-filter", "03 TUI", SRC_TUI,
              r"measurement_type = 'measured'",
              r"Measured tokens, cost and agent count",
              "measured-only totals"),
        Claim("prov-wrapper", "01 Pipeline", SRC_WRAPPER,
              r"--log-estimate",
              r"writes an estimated baseline row",
              "wrapper logs estimates"),
        Claim("prov-measured-cli", "01 Pipeline", SRC_DAEMON,
              r'print\(f"Measured: ',
              r"Measured: 412650 tok \| \$1\.2841",
              "daemon measured output"),
        Claim("prov-estimated-cli", "01 Pipeline", SRC_DAEMON,
              r'print\(f"Estimated: ',
              r"Estimated: 96450 tok \| \$0\.0060",
              "daemon estimated output"),
        Claim("db-env", "01 Pipeline", SRC_DAEMON,
              r'"TOKENGRID_DB_PATH"',
              r"TOKENGRID_DB_PATH",
              "DB path override"),
        Claim("db-default-path", "01 Pipeline", SRC_DAEMON,
              r"\.local/share/omarchy/tokengrid/tokengrid\.db",
              r"\.local/share/omarchy/tokengrid/tokengrid\.db",
              "default DB path (outside plugin dir)"),
        Claim("db-tui-path", "01 Pipeline", SRC_TUI,
              r"\.local/share/omarchy/tokengrid/tokengrid\.db",
              r"kept outside the plugin dir",
              "TUI uses the same runtime DB path"),
        Claim("db-migrate", "01 Pipeline", SRC_DAEMON,
              r"io\.github\.cybercoretech\.tokengrid/data/tokengrid\.db",
              r"adopted on first run",
              "legacy DB migration retained"),
        Claim("roster-seed", "02 Coverage", SRC_DAEMON,
              r"INSERT OR IGNORE INTO agents",
              r"INSERT OR IGNORE",
              "idempotent roster seeding"),
        Claim("roster-budget", "02 Coverage", SRC_DAEMON,
              r"budget_limit REAL DEFAULT 10\.0",
              r"budget_limit REAL DEFAULT 10\.0",
              "default budget cap"),

        # --- surfaces + setup ---
        Claim("aliases-file", "01 Pipeline", SRC_ALIASES,
              r'_tokengrid_run "',
              r"config/tokengrid_aliases\.zsh",
              "shell wrapper file"),
        Claim("bar-payload", "01 Pipeline", SRC_PAYLOAD,
              r"get_today_summary",
              r"A bar payload puts today's tokens and dollars",
              "Waybar JSON payload"),
        Claim("bar-icon", "04 Setup", SRC_QML,
              r'barIcon: "󰓡"',
              r"Bar icon 󰓡",
              "bar widget icon"),
        Claim("launch-script", "04 Setup", SRC_QML,
              r"helpers/launch-tokengrid\.sh",
              r"helpers/launch-tokengrid\.sh",
              "launch helper"),
        Claim("ghostty-single", "04 Setup", SRC_GHOSTTY,
              r"gtk-single-instance = false",
              r"gtk-single-instance = ",
              "ghostty process isolation"),
        Claim("ghostty-title", "04 Setup", SRC_GHOSTTY,
              r"title = TokenGrid",
              r"title = TokenGrid",
              "ghostty window title"),
        Claim("rule-size", "04 Setup", SRC_LUA,
              r'size = "1100 650"',
              r'"1100 650"',
              "float window size"),
        Claim("build-release", "04 Setup", SRC_BUILD,
              r"cargo build --release",
              r"cargo build --release",
              "build command"),
        Claim("build-install", "04 Setup", SRC_BUILD,
              r"--install",
              r"build_tui\.sh --install",
              "install mode"),
        Claim("metrics-json", "02 Coverage", SRC_METRICS,
              r'"daily\.json"',
              r"daily, weekly and monthly JSON aggregates",
              "system metrics aggregates"),
        Claim("metrics-psutil", "02 Coverage", SRC_METRICS,
              r"import psutil",
              r"via psutil",
              "metrics sampling"),
        Claim("aliases-selfrelative", "01 Pipeline", SRC_ALIASES,
              r'TOKENGRID_WRAPPER="\$\{\$\{\(%\):-%x\}:A:h:h\}/bin/agent_wrapper\.sh',
              r"bin/agent_wrapper\.sh",
              "wrapper path is self-relative"),
        Claim("rule-title", "04 Setup", SRC_LUA,
              r'(?<!initial_)title = "TokenGrid"',
              r'title = <span class="s">"TokenGrid"</span>',
              "float rule matches ghostty title"),
    ]
    claims.extend(dynamic_claims())
    return claims


def dynamic_claims() -> list[Claim]:
    """Claims whose numbers are derived from source, never hard-coded twice."""
    claims: list[Claim] = []

    roster = re.search(r"INITIAL_AGENTS = \[(.*?)\]", read(SRC_DAEMON), re.S)
    if roster:
        agents = re.findall(r'"([^"]+)"', roster.group(1))
        n = len(agents)
        claims.append(Claim(
            "roster-count", "02 Coverage", SRC_DAEMON,
            r"INITIAL_AGENTS = \[",
            rf"{n} CLI AI agents",
            f"roster is {n} agents — lede must say {n}"),
        )
        claims.append(Claim(
            "wrapper-stat", "01 Pipeline", SRC_ALIASES,
            r'_tokengrid_run "',
            rf"<b>{n}</b><span>AGENT WRAPPERS</span>",
            f"page stat must show {n} agent wrappers"),
        )

    tables = len(re.findall(r"CREATE TABLE IF NOT EXISTS", read(SRC_DAEMON)))
    claims.append(Claim(
        "table-stat", "01 Pipeline", SRC_DAEMON,
        r"CREATE TABLE IF NOT EXISTS",
        rf"<b>{tables}</b><span>SQLITE TABLES</span>",
        f"page stat must show {tables} SQLite tables"),
    )

    page = read("docs/index.html")
    mashups = len(re.findall(r"id: 'tokengrid-", page))
    claims.append(Claim(
        "mashup-count", "06 Themes", "docs/index.html",
        r"id: 'tokengrid-",
        rf"{mashups} signature mashups",
        f"mashup ids found: {mashups} — heading must match"),
    )
    claims.append(Claim(
        "mashup-comment", "06 Themes", "docs/index.html",
        r"id: 'tokengrid-",
        rf"{mashups} TokenGrid mixes \+ ",
        f"config comment must say {mashups} mixes"),
    )
    return claims


def cross_checks() -> list[str]:
    """Consistency checks between source files (not page copy)."""
    errors: list[str] = []
    roster = re.search(r"INITIAL_AGENTS = \[(.*?)\]", read(SRC_DAEMON), re.S)
    agents = set(re.findall(r'"([^"]+)"', roster.group(1))) if roster else set()
    if agents:
        wrappers = set(re.findall(r'_tokengrid_run "([^"]+)"', read(SRC_ALIASES)))
        if agents != wrappers:
            only_roster = sorted(agents - wrappers)
            only_wrappers = sorted(wrappers - agents)
            errors.append(
                "roster vs wrappers mismatch:\n"
                f"      daemon INITIAL_AGENTS only: {only_roster or '—'}\n"
                f"      aliases only:               {only_wrappers or '—'}\n"
                "      → every rostered agent needs an alias and vice versa")

        try:
            desc = read("plugin.json")
        except FileNotFoundError:
            desc = ""
        count = re.search(r"(\d+) CLI AI agents", desc)
        if not count:
            errors.append("plugin.json description lacks an agent count "
                          f"(roster is {len(agents)})")
        elif int(count.group(1)) != len(agents):
            errors.append(f"plugin.json says {count.group(1)} CLI AI agents, "
                          f"roster has {len(agents)}")

    errors.extend(personal_path_check())
    return errors


SCAN_GLOBS = [
    "*.md", "*.json", "*.zsh", "*.sh", "*.py", "*.rs", "*.lua", "*.qml",
    "config/*", "helpers/*", "bin/*", "bar_module/*", "lua/*",
    "scripts/*", "src/*", "tui/src/*", "docs/*.html", "docs/*.css",
]
PERSONAL_PATH = re.compile(r"/home/|Devspace")


def personal_path_check() -> list[str]:
    """No machine-specific paths in distributable files (site + plugin)."""
    errors: list[str] = []
    seen: set[Path] = set()
    self_path = Path(__file__).resolve()
    for pattern in SCAN_GLOBS:
        for path in sorted(ROOT.glob(pattern)):
            if not path.is_file() or path in seen:
                continue
            seen.add(path)
            if path.name.endswith(".pre-telemetry") or path.resolve() == self_path:
                continue  # the checker itself holds the detection pattern
            try:
                text = path.read_text(encoding="utf-8")
            except (UnicodeDecodeError, OSError):
                continue  # binary (compiled TUI, caches)
            for lineno, line in enumerate(text.splitlines(), 1):
                if PERSONAL_PATH.search(line):
                    rel = path.relative_to(ROOT)
                    errors.append(f"{rel}:{lineno} contains a personal path: "
                                  f"{line.strip()[:80]}")
    return errors


class _Balance(HTMLParser):
    VOID = {"meta", "link", "br", "img", "input", "hr", "source"}

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.stack: list[str] = []
        self.errors: list[str] = []

    def handle_starttag(self, tag: str, attrs) -> None:
        if tag not in self.VOID:
            self.stack.append(tag)

    def handle_endtag(self, tag: str) -> None:
        if tag in self.VOID:
            return
        if self.stack and self.stack[-1] == tag:
            self.stack.pop()
        else:
            self.errors.append(f"stray </{tag}> (stack tail: {self.stack[-3:]})")


def html_balance_check() -> list[str]:
    parser = _Balance()
    parser.feed(read("docs/index.html"))
    errors = list(parser.errors)
    if parser.stack:
        errors.append(f"unclosed tags: {parser.stack}")
    return errors


def main() -> int:
    try:
        read("docs/index.html")
    except FileNotFoundError:
        print("FAIL docs/index.html not found")
        return 1

    claims = build_claims()
    print(f"TokenGrid docs copy check — {len(claims)} claims")

    passed = failed = 0
    for c in claims:
        try:
            src_text = read(c.src)
        except FileNotFoundError:
            print(f"FAIL {c.key:<22} [{c.section}] source file missing: {c.src}")
            failed += 1
            continue

        if not re.search(c.src_re, src_text):
            print(f"FAIL {c.key:<22} [{c.section}] {c.note}")
            print(f"       source fact gone: {c.src}")
            print(f"       pattern: {c.src_re}")
            print(f"       → upstream changed; review the copy in docs/index.html ({c.section})")
            failed += 1
            continue

        if not re.search(c.page_re, read("docs/index.html")):
            print(f"FAIL {c.key:<22} [{c.section}] {c.note}")
            print(f"       page copy missing in docs/index.html")
            print(f"       pattern: {c.page_re}")
            print(f"       → copy was edited away; restore it or update scripts/check_copy.py")
            failed += 1
            continue

        print(f"ok   {c.key:<22} [{c.section}] {c.note}")
        passed += 1

    cross_errors = cross_checks() + html_balance_check()
    if cross_errors:
        for err in cross_errors:
            print(f"FAIL cross-check: {err}")
        failed += len(cross_errors)
    else:
        print("ok   cross-checks           roster↔wrappers, plugin.json count, "
              "no personal paths, html balance")
        passed += 1

    print(f"\n{passed} passed, {failed} failed")
    if failed:
        print("drift detected — the site copy no longer matches the source.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
