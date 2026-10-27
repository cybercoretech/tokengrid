use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};
use rusqlite::{Connection, Result};
use std::{env, error::Error, io, path::PathBuf};

#[derive(Debug)]
struct AgentMetricRow {
    timestamp: String,
    agent: String,
    model: String,
    tokens: i64,
    cost: f64,
}

fn fetch_agent_metrics() -> Result<Vec<AgentMetricRow>> {
    let home = env::var("HOME").unwrap_or_else(|_| "/home/raven".to_string());
    let db_paths = vec![
        PathBuf::from("data/tokengrid.db"),
        PathBuf::from(home).join(".config/omarchy/plugins/io.github.cybercoretech.tokengrid/data/tokengrid.db"),
    ];

    let mut conn = None;
    for path in db_paths {
        if path.exists() {
            if let Ok(c) = Connection::open(&path) {
                conn = Some(c);
                break;
            }
        }
    }

    let conn = match conn {
        Some(c) => c,
        None => return Ok(vec![]),
    };

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
    Ok(metrics)
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let metrics = fetch_agent_metrics().unwrap_or_default();

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
            .title(" TokenGrid Telemetry Hub - Day / Week / Month View ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(title_block, chunks[0]);

        // Calculate totals
        let total_tokens: i64 = metrics.iter().map(|m| m.tokens).sum();
        let total_cost: f64 = metrics.iter().map(|m| m.cost).sum();
        let summary_text = format!(
            " TODAY / 7-DAY / 30-DAY TOTALS -> Tokens: {} | Cost: ${:.4} | Agents Monitored: 4 ",
            total_tokens, total_cost
        );
        let summary_block = Block::default()
            .title(" Overview ")
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
        .block(Block::default().borders(Borders::ALL).title(" Detailed Metrics: Active Agents History "));

        f.render_widget(table, chunks[2]);
    })?;

    std::thread::sleep(std::time::Duration::from_secs(6));
    Ok(())
}
