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
    model: String,
    tokens: i64,
    cost: f64,
}

fn fetch_agent_metrics() -> Result<Vec<AgentMetricRow>> {
    let home = env::var("HOME").unwrap_or_else(|_| "/home/raven".to_string());
    let db_path = PathBuf::from(home)
        .join(".config/omarchy/plugins/io.github.cybercoretech.tokengrid/data/tokengrid.db");
    
    // Fallback to local devspace db if config path doesn't exist yet
    let active_db = if db_path.exists() {
        db_path
    } else {
        PathBuf::from("data/tokengrid.db")
    };

    if !active_db.exists() {
        // Return dummy row if DB isn't initialized yet so the UI shows data
        return Ok(vec![AgentMetricRow {
            timestamp: "No Data Recorded".into(),
            model: "N/A".into(),
            tokens: 0,
            cost: 0.0,
        }]);
    }

    let conn = Connection::open(active_db)?;
    // Ensure table exists safely
    conn.execute(
        "CREATE TABLE IF NOT EXISTS requests (
            timestamp TEXT,
            model TEXT,
            tokens INTEGER,
            cost REAL
        )",
        [],
    )?;

    let mut stmt = conn.prepare("SELECT timestamp, model, tokens, cost FROM requests ORDER BY timestamp DESC LIMIT 50")?;
    let rows = stmt.query_map([], |row| {
        Ok(AgentMetricRow {
            timestamp: row.get(0)?,
            model: row.get(1)?,
            tokens: row.get(2)?,
            cost: row.get(3)?,
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
            timestamp: "Awaiting Agent Activity".into(),
            model: "default".into(),
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

    let metrics = fetch_agent_metrics().unwrap_or_default();

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)].as_ref())
            .split(f.size());

        let title_block = Block::default()
            .title(" TokenGrid Detailed Metrics - Day / Week / Month View ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(title_block, chunks[0]);

        let rows: Vec<Row> = metrics
            .iter()
            .map(|m| {
                Row::new(vec![
                    Cell::from(m.timestamp.clone()),
                    Cell::from(m.model.clone()),
                    Cell::from(m.tokens.to_string()),
                    Cell::from(format!("${:.4}", m.cost)),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(25),
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(15),
            ],
        )
        .header(
            Row::new(vec!["Timestamp", "Model", "Tokens", "Cost"])
                .style(Style::default().fg(Color::Yellow)),
        )
        .block(Block::default().borders(Borders::ALL).title(" Agent Usage History "));

        f.render_widget(table, chunks[1]);
    })?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    Ok(())
}
