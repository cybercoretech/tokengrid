use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetricPoint {
    pub timestamp: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
}

pub fn load_metric_history(view_type: &str) -> Vec<MetricPoint> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/raven".to_string());
    let path = PathBuf::from(home)
        .join(".local/share/omarchy/tokengrid/metrics")
        .join(format!("{}.json", view_type));

    if let Ok(contents) = fs::read_to_string(path) {
        if let Ok(points) = serde_json::from_str::<Vec<MetricPoint>>(&contents) {
            return points;
        }
    }
    Vec::new()
}
