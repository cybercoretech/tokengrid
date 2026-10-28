use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame, Terminal,
};
use rusqlite::Connection;

#[derive(Clone)]
struct AgentRow {
    id: i64,
    name: String,
    model: String,
    budget: f64,
    tokens_today: i64,
    cost_today: f64,
}

struct AgentDetailData {
    name: String,
    model: String,
    budget: f64,
    today_tokens: i64,
    today_cost: f64,
    week_tokens: i64,
    week_cost: f64,
    total_requests: i64,
    recent_requests: Vec<(String, String, i64, i64, f64, String)>,
}

#[derive(Clone, PartialEq)]
enum AppState {
    Normal,
    Detail(String),
    Settings,
    ConfirmDelete {
        id: i64,
        name: String,
        input: String,
    },
    Adding {
        name: String,
        model: String,
        budget: String,
        active_field: usize, // 0: Name, 1: Model, 2: Budget
    },
}

struct App {
    state: AppState,
    agents: Vec<AgentRow>,
    selected_index: usize,
    should_quit: bool,
}

impl App {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut app = Self {
            state: AppState::Normal,
            agents: Vec::new(),
            selected_index: 0,
            should_quit: false,
        };
        app.refresh_data()?;
        Ok(app)
    }

    fn refresh_data(&mut self) -> Result<(), Box<dyn Error>> {
        let db_path = get_db_path();
        let conn = Connection::open(&db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS token_usage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                agent_name TEXT NOT NULL,
                model_name TEXT,
                prompt_tokens INTEGER DEFAULT 0,
                completion_tokens INTEGER DEFAULT 0,
                cost_est REAL DEFAULT 0.0,
                measurement_type TEXT NOT NULL DEFAULT 'unknown'
            )",
            [],
        )?;

        let has_measurement_type: bool = {
            let mut columns = conn.prepare("PRAGMA table_info(token_usage)")?;
            let rows = columns.query_map([], |row| row.get::<_, String>(1))?;
            let mut found = false;
            for column in rows {
                if column? == "measurement_type" {
                    found = true;
                    break;
                }
            }
            found
        };
        if !has_measurement_type {
            conn.execute(
                "ALTER TABLE token_usage ADD COLUMN measurement_type TEXT NOT NULL DEFAULT 'unknown'",
                [],
            )?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS agents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_name TEXT UNIQUE NOT NULL,
                model_default TEXT DEFAULT 'default',
                budget_limit REAL DEFAULT 10.0
            )",
            [],
        )?;

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM agents")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        if count == 0 {
            let default_agents = vec![
                "gemini",
                "claude",
                "crush",
                "cursor-agent",
                "codex",
                "copilot",
                "oh-my-pi",
                "pi",
                "opencode",
                "grok",
                "ori",
                "kimi",
                "muse",
                "antigravity-cli",
                "qwen",
                "devin",
                "hermes",
                "hermes-agent",
                "kilocode",
            ];
            for a in default_agents {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO agents (agent_name, model_default, budget_limit) VALUES (?1, 'default', 10.0)",
                    rusqlite::params![a],
                );
            }
        }

        let mut stmt = conn.prepare(
            "SELECT a.id, a.agent_name, a.model_default, a.budget_limit,
                    COALESCE(SUM(CASE WHEN t.measurement_type = 'measured' AND date(t.timestamp) = date('now') THEN (t.prompt_tokens + t.completion_tokens) ELSE 0 END), 0) as tok_sum,
                    COALESCE(SUM(CASE WHEN t.measurement_type = 'measured' AND date(t.timestamp) = date('now') THEN t.cost_est ELSE 0.0 END), 0.0) as cost_sum
             FROM agents a
             LEFT JOIN token_usage t ON a.agent_name = t.agent_name
             GROUP BY a.id, a.agent_name, a.model_default, a.budget_limit
             ORDER BY a.agent_name ASC",
        )?;

        let agent_iter = stmt.query_map([], |row| {
            Ok(AgentRow {
                id: row.get(0)?,
                name: row.get(1)?,
                model: row.get(2)?,
                budget: row.get(3)?,
                tokens_today: row.get(4)?,
                cost_today: row.get(5)?,
            })
        })?;

        self.agents.clear();
        for agent in agent_iter {
            if let Ok(item) = agent {
                self.agents.push(item);
            }
        }

        if !self.agents.is_empty() && self.selected_index >= self.agents.len() {
            self.selected_index = self.agents.len() - 1;
        }

        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode) {
        let mut confirmed_delete: Option<i64> = None;
        match &mut self.state {
            AppState::Normal => match code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    self.should_quit = true;
                }
                KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => {
                    if !self.agents.is_empty() && self.selected_index < self.agents.len() - 1 {
                        self.selected_index += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char('\r') | KeyCode::Char('\n') => {
                    if !self.agents.is_empty() && self.selected_index < self.agents.len() {
                        let target_name = self.agents[self.selected_index].name.clone();
                        self.state = AppState::Detail(target_name);
                    }
                }
                KeyCode::Char('d')
                | KeyCode::Char('D')
                | KeyCode::Char('x')
                | KeyCode::Char('X') => {
                    if let Some(agent) = self.agents.get(self.selected_index) {
                        self.state = AppState::ConfirmDelete {
                            id: agent.id,
                            name: agent.name.clone(),
                            input: String::new(),
                        };
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    self.state = AppState::Adding {
                        name: String::new(),
                        model: "default".to_string(),
                        budget: "10.0".to_string(),
                        active_field: 0,
                    };
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.state = AppState::Settings;
                }
                _ => {}
            },
            AppState::Adding {
                name,
                model,
                budget,
                active_field,
            } => match code {
                KeyCode::Esc => {
                    self.state = AppState::Normal;
                }
                KeyCode::Tab | KeyCode::Down => {
                    *active_field = (*active_field + 1) % 3;
                }
                KeyCode::Up => {
                    *active_field = if *active_field == 0 {
                        2
                    } else {
                        *active_field - 1
                    };
                }
                KeyCode::Enter | KeyCode::Char('\r') | KeyCode::Char('\n') => {
                    if *active_field < 2 {
                        *active_field += 1;
                    } else {
                        let b = budget.parse::<f64>().unwrap_or(10.0);
                        if !name.trim().is_empty() {
                            let _ = add_agent_full_db(name, model, b);
                            let _ = self.refresh_data();
                        }
                        self.state = AppState::Normal;
                    }
                }
                KeyCode::Backspace => match *active_field {
                    0 => {
                        name.pop();
                    }
                    1 => {
                        model.pop();
                    }
                    2 => {
                        budget.pop();
                    }
                    _ => {}
                },
                KeyCode::Char(c) => match *active_field {
                    0 => name.push(c),
                    1 => model.push(c),
                    2 => budget.push(c),
                    _ => {}
                },
                _ => {}
            },
            AppState::Detail(_) => match code {
                KeyCode::Char('q')
                | KeyCode::Char('Q')
                | KeyCode::Esc
                | KeyCode::Backspace
                | KeyCode::Enter
                | KeyCode::Char('\r')
                | KeyCode::Char('\n') => {
                    self.state = AppState::Normal;
                }
                _ => {}
            },
            AppState::ConfirmDelete { id, name, input } => match code {
                KeyCode::Esc => {
                    self.state = AppState::Normal;
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter | KeyCode::Char('\r') | KeyCode::Char('\n') => {
                    if input.as_str() == name.as_str() {
                        confirmed_delete = Some(*id);
                        self.state = AppState::Normal;
                    }
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                _ => {}
            },
            AppState::Settings => match code {
                KeyCode::Char('q')
                | KeyCode::Char('Q')
                | KeyCode::Esc
                | KeyCode::Char('s')
                | KeyCode::Char('S')
                | KeyCode::Enter
                | KeyCode::Char('\r')
                | KeyCode::Char('\n') => {
                    self.state = AppState::Normal;
                }
                _ => {}
            },
        }

        // Perform destructive work only after the confirmation-state borrow ends.
        if let Some(target_id) = confirmed_delete {
            let _ = delete_agent_db(target_id);
            let _ = self.refresh_data();
            if !self.agents.is_empty() && self.selected_index >= self.agents.len() {
                self.selected_index = self.agents.len() - 1;
            }
        }
    }
}

fn get_db_path() -> PathBuf {
    if let Some(path) = std::env::var_os("TOKENGRID_DB_PATH") {
        return PathBuf::from(path);
    }

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    // Runtime state lives outside the plugin directory: the shell watches
    // ~/.config/omarchy/plugins with inotify and hot-reloads the whole bar
    // on any write inside it, so a database in the plugin dir would reload
    // the bar every time the TUI opens.
    let path = home.join(".local/share/omarchy/tokengrid/tokengrid.db");
    let legacy = home.join(
        ".config/omarchy/plugins/io.github.cybercoretech.tokengrid/data/tokengrid.db",
    );
    // One-time migration: adopt a pre-0.2 database still in the plugin dir.
    if !path.exists() && legacy.exists() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::copy(&legacy, &path);
    }
    path
}

fn delete_agent_db(id: i64) -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("DELETE FROM agents WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

fn add_agent_full_db(name: &str, model: &str, budget: f64) -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute(
        "INSERT OR REPLACE INTO agents (agent_name, model_default, budget_limit) VALUES (?1, ?2, ?3)",
        rusqlite::params![name.trim(), model.trim(), budget],
    )?;
    Ok(())
}

fn fetch_agent_details(agent_name: &str) -> Result<AgentDetailData, Box<dyn Error>> {
    let conn = Connection::open(get_db_path())?;

    let mut stmt =
        conn.prepare("SELECT model_default, budget_limit FROM agents WHERE agent_name = ?1")?;
    let (model, budget) = stmt
        .query_row(rusqlite::params![agent_name], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .unwrap_or_else(|_| ("default".to_string(), 10.0));

    let mut stmt_today = conn.prepare(
        "SELECT COALESCE(SUM(prompt_tokens + completion_tokens), 0), COALESCE(SUM(cost_est), 0.0)
         FROM token_usage WHERE agent_name = ?1
         AND measurement_type = 'measured' AND date(timestamp) = date('now')",
    )?;
    let (today_tokens, today_cost) = stmt_today
        .query_row(rusqlite::params![agent_name], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
        })
        .unwrap_or((0, 0.0));

    let mut stmt_week = conn.prepare(
        "SELECT COALESCE(SUM(prompt_tokens + completion_tokens), 0), COALESCE(SUM(cost_est), 0.0), COUNT(*)
         FROM token_usage WHERE agent_name = ?1
         AND measurement_type = 'measured'
         AND timestamp >= datetime('now', '-7 days')"
    )?;
    let (week_tokens, week_cost, total_requests) = stmt_week
        .query_row(rusqlite::params![agent_name], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .unwrap_or((0, 0.0, 0));

    let mut stmt_reqs = conn.prepare(
        "SELECT timestamp, COALESCE(model_name, 'default'), prompt_tokens, completion_tokens, cost_est,
                COALESCE(measurement_type, 'unknown')
         FROM token_usage WHERE agent_name = ?1 ORDER BY timestamp DESC LIMIT 8"
    )?;
    let req_iter = stmt_reqs.query_map(rusqlite::params![agent_name], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, f64>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;

    let mut recent_requests = Vec::new();
    for r in req_iter {
        if let Ok(item) = r {
            recent_requests.push(item);
        }
    }

    Ok(AgentDetailData {
        name: agent_name.to_string(),
        model,
        budget,
        today_tokens,
        today_cost,
        week_tokens,
        week_cost,
        total_requests,
        recent_requests,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key.code);
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(6),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(f.size());

    let total_tokens: i64 = app.agents.iter().map(|a| a.tokens_today).sum();
    let total_cost: f64 = app.agents.iter().map(|a| a.cost_today).sum();
    let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let header_text = format!("TOKENGRID TUI | System Time: {}", now_str);
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" TokenGrid Telemetry Hub "),
        );
    f.render_widget(header, chunks[0]);

    let summary_text = format!(
        "Today's Total Tokens: {:>10}  |  Total Cost: ${:<8.4}  |  Monitored Agents: {:>2}",
        total_tokens,
        total_cost,
        app.agents.len()
    );
    let summary_panel = Paragraph::new(summary_text)
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Daily Overview "),
        );
    f.render_widget(summary_panel, chunks[1]);

    match &app.state {
        AppState::Normal | AppState::ConfirmDelete { .. } => {
            let header_cells = [
                "Agent Name",
                "Default Model",
                "Budget ($)",
                "Tokens Today",
                "Cost ($)",
            ]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
            let header_row = Row::new(header_cells).height(1).bottom_margin(1);

            let rows = app.agents.iter().enumerate().map(|(i, item)| {
                let is_selected = i == app.selected_index;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let cells = vec![
                    Cell::from(item.name.clone()),
                    Cell::from(item.model.clone()),
                    Cell::from(format!("{:.2}", item.budget)),
                    Cell::from(format!("{}", item.tokens_today)),
                    Cell::from(format!("{:.4}", item.cost_today)),
                ];
                Row::new(cells).height(1).style(style)
            });

            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(15),
                    Constraint::Percentage(20),
                    Constraint::Percentage(15),
                ],
            )
            .header(header_row)
            .block(Block::default().borders(Borders::ALL).title(" Agent Management [j/k: navigate, Enter: details, a: add, d: delete, s: settings, q: quit] "));

            f.render_widget(table, chunks[2]);

            if let AppState::ConfirmDelete { name, input, .. } = &app.state {
                let vertical = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Min(0),
                            Constraint::Length(7),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[2]);

                let horizontal = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(15),
                            Constraint::Percentage(70),
                            Constraint::Percentage(15),
                        ]
                        .as_ref(),
                    )
                    .split(vertical[1]);

                let popup_area = horizontal[1];
                f.render_widget(Clear, popup_area);

                let prompt = format!(
                    "Agent: {}\nType the exact name to delete:\n> {}\n\nEnter: confirm  Esc: cancel",
                    name, input
                );
                let popup = Paragraph::new(prompt)
                    .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Confirm Agent Deletion "),
                    );
                f.render_widget(popup, popup_area);
            }
        }
        AppState::Adding {
            name,
            model,
            budget,
            active_field,
        } => {
            let form_text = format!(
                "=== REGISTER NEW AI AGENT ===\n\n[1] Agent Name:    {}{}\n[2] Default Model: {}{}\n[3] Budget Limit:  ${}{}\n\nControls:\n• Tab / Up / Down: Switch Fields\n• Enter: Next Field (or Submit on Budget)\n• Esc: Cancel",
                name, if *active_field == 0 { " █" } else { "" },
                model, if *active_field == 1 { " █" } else { "" },
                budget, if *active_field == 2 { " █" } else { "" }
            );
            let form_panel = Paragraph::new(form_text)
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Add New Agent Form "),
                );
            f.render_widget(form_panel, chunks[2]);
        }
        AppState::Detail(agent_name) => {
            let details = fetch_agent_details(agent_name).unwrap_or(AgentDetailData {
                name: agent_name.clone(),
                model: "unknown".into(),
                budget: 0.0,
                today_tokens: 0,
                today_cost: 0.0,
                week_tokens: 0,
                week_cost: 0.0,
                total_requests: 0,
                recent_requests: Vec::new(),
            });

            let detail_text = format!(
                "Agent: {}  |  Model: {}  |  Budget Cap: ${:.2}\n\n► TODAY (measured): {} Tokens | ${:.4}\n► 7-DAY (measured): {} Tokens | ${:.4} ({} usage records)\n\n--- Recent Usage Log ---",
                details.name, details.model, details.budget, details.today_tokens, details.today_cost, details.week_tokens, details.week_cost, details.total_requests
            );

            let req_rows = details.recent_requests.iter().map(|req| {
                let cells = vec![
                    Cell::from(req.0.clone()),
                    Cell::from(req.1.clone()),
                    Cell::from(format!("{}", req.2 + req.3)),
                    Cell::from(format!("${:.4}", req.4)),
                    Cell::from(req.5.clone()),
                ];
                Row::new(cells).height(1)
            });

            let req_table = Table::new(
                req_rows,
                [
                    Constraint::Percentage(28),
                    Constraint::Percentage(24),
                    Constraint::Percentage(16),
                    Constraint::Percentage(14),
                    Constraint::Percentage(18),
                ],
            )
            .header(
                Row::new(vec!["Timestamp", "Model", "Tokens", "Cost", "Type"])
                    .style(Style::default().fg(Color::Yellow)),
            )
            .block(Block::default().borders(Borders::ALL).title(format!(
                " Detailed Metrics: {} [Esc/Enter: Back] ",
                agent_name
            )));

            let detail_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(4)].as_ref())
                .split(chunks[2]);

            let summary_para = Paragraph::new(detail_text).style(Style::default().fg(Color::White));
            f.render_widget(summary_para, detail_chunks[0]);
            f.render_widget(req_table, detail_chunks[1]);
        }
        AppState::Settings => {
            let settings_text = format!(
                "=== TOKENGRID CONFIGURATION & SETTINGS ===\n\nActive Database Path: {}\nGlobal Daily Budget Cap: $50.00 (Configured in SQLite)\nWebhook Alerts URL: https://discord.com/api/webhooks/example\n\n[Press 's', 'Esc', or 'Enter' to return to Agent List]",
                get_db_path().display()
            );
            let settings_panel = Paragraph::new(settings_text)
                .style(Style::default().fg(Color::Magenta))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Settings & Configuration "),
                );
            f.render_widget(settings_panel, chunks[2]);
        }
    }

    let footer_text = match &app.state {
        AppState::Normal => "Vim Keys: [j/k] Navigate  [Enter] Agent Details  [a] Add  [d] Delete  [s] Settings  [q] Quit",
        AppState::Adding { .. } => "Form Input: [Tab/Up/Down] Switch Field  [Enter] Next/Submit  [Esc] Cancel",
        AppState::Detail(_) => "Press [Esc], [Enter], or [q] to return to Agent List",
        AppState::Settings => "Press [Esc], [Enter], or [s] to return to Agent List",
        AppState::ConfirmDelete { .. } => "Type the exact agent name, [Enter] to delete, [Esc] to cancel",
    };
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[3]);
}
