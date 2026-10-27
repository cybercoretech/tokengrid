use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};
use rusqlite::{Connection, Result};
use std::{error::Error, io};

#[derive(Debug)]
struct AgentMetricRow {
    timestamp: String,
    agent: String,
    model: String,
    tokens: i64,
    cost: f64,
}

fn fetch_agent_metrics() -> Result<Vec<AgentMetricRow>> {
    // Use the absolute path to the active plugin database
    let db_path = "/home/raven/.config/omarchy/plugins/io.github.cybercoretech.tokengrid/data/tokengrid.db";
    
    let conn = Connection::open(db_path)?;
    
    // Ensure table exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT,
            agent TEXT,
            model TEXT,
            tokens INTEGER,
            cost REAL
        )",
        [],
    )?;

    let mut stmt = conn.prepare("SELECT timestamp, agent, model, tokens, cost FROM requests ORDER BY timestamp DESC LIMIT 50")?;
    let rows = stmt.query_map([], |row| {
        Ok(AgentMetricRow {
            timestamp: row.get(0)?,
            agent: row.get(1)?,
            model: row.get(2)?,
            tokens: row.get(3)?,
            cost: row.get(4)?,
        })
    })?;

    let mut metrics = Vec::new();
    for row in rows {
        if let Ok(m) = row {
            metrics.push(m);
        }
    }

    if metrics.is_empty() {
        metrics.push(AgentMetricRow {
            timestamp: "No records found".into(),
            agent: "N/A".into(),
            model: "N/A".into(),
            tokens: 0,
            cost: 0.0,
        });
    }

    Ok(metrics)
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let metrics = fetch_agent_metrics().unwrap_or_else(|e| {
        vec![AgentMetricRow {
            timestamp: format!("DB Error: {}", e),
            agent: "Error".into(),
            model: "Error".into(),
            tokens: 0,
            cost: 0.0,
        }]
    });

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Min(10),
            ])
            .split(f.size());

        let title_block = Block::default()
            .title(" TokenGrid Telemetry Hub - Detailed Metrics ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(title_block, chunks[0]);

        let total_tokens: i64 = metrics.iter().map(|m| m.tokens).sum();
        let total_cost: f64 = metrics.iter().map(|m| m.cost).sum();
        let summary_text = format!(
            " TOTALS -> Tokens: {} | Cost: ${:.4} | Active Agents: 4 ",
            total_tokens, total_cost
        );
        let summary_block = Block::default()
            .title(" Summary Overview ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));
        
        let summary_widget = ratatui::widgets::Paragraph::new(summary_text).block(summary_block);
        f.render_widget(summary_widget, chunks[1]);

        let rows: Vec<Row> = metrics
            .iter()
            .map(|m| {
                Row::new(vec![
                    Cell::from(m.timestamp.clone()),
                    Cell::from(m.agent.clone()),
                    Cell::from(m.model.clone()),
                    Cell::from(m.tokens.to_string()),
                    Cell::from(format!("${:.4}", m.cost)),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(22),
                Constraint::Length(15),
                Constraint::Length(20),
                Constraint::Length(12),
                Constraint::Length(12),
            ],
        )
        .header(
            Row::new(vec!["Timestamp", "Agent", "Model", "Tokens", "Cost"])
                .style(Style::default().fg(Color::Yellow)),
        )
        .block(Block::default().borders(Borders::ALL).title(" Agent Usage History (Last 24 Hours) "));

        f.render_widget(table, chunks[2]);
    })?;

    // Keep TUI open briefly or wait for key press
    std::thread::sleep(std::time::Duration::from_secs(8));
    Ok(())
}
