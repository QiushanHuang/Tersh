//! Numeric observations only; missing or failed measurements stay missing.
use crate::cluster::{ConnectionState, HostSnapshot};
use std::time::SystemTime;

pub const HISTORY_LIMIT: usize = 60;

#[derive(Debug, Clone)]
pub struct MetricSample {
    pub at: SystemTime,
    /// Load average, used memory %, used storage %, complete probe duration ms.
    pub values: [Option<f64>; 4],
}

impl MetricSample {
    pub fn from_snapshot(snapshot: &HostSnapshot) -> Self {
        let mut values = [None; 4];
        if snapshot.connection == ConnectionState::Online {
            values[0] = snapshot
                .report
                .cpu_load
                .as_deref()
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<f64>().ok())
                .filter(|v| v.is_finite() && *v >= 0.0);
            values[1] = snapshot
                .report
                .memory
                .as_deref()
                .and_then(memory_used)
                .map(f64::from);
            values[2] = snapshot
                .report
                .storage
                .as_deref()
                .and_then(percent)
                .map(f64::from);
            values[3] = snapshot.latency_ms.map(|v| v as f64);
        }
        Self {
            at: snapshot.refreshed_at.unwrap_or_else(SystemTime::now),
            values,
        }
    }
}

pub fn percent(input: &str) -> Option<u16> {
    for (index, ch) in input.char_indices() {
        if ch != '%' && ch != '％' {
            continue;
        }
        let before = input[..index].trim_end();
        let start = before
            .char_indices()
            .rev()
            .find(|(_, ch)| !ch.is_ascii_digit() && *ch != '.' && *ch != '-')
            .map(|(index, ch)| index + ch.len_utf8())
            .unwrap_or(0);
        if let Ok(value) = before[start..].parse::<f64>()
            && value.is_finite()
            && (0.0..=100.0).contains(&value)
        {
            return Some(value.round() as u16);
        }
    }
    None
}

pub fn memory_used(raw: &str) -> Option<u16> {
    percent(raw).map(|value| {
        if raw.to_ascii_lowercase().contains("free") {
            100 - value
        } else {
            value
        }
    })
}

/// One character per observation, no interpolation across a failed probe.
pub fn sparkline(values: &[Option<f64>], ceiling: f64, unicode: bool) -> String {
    let symbols = if unicode {
        ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█']
    } else {
        ['_', '.', ':', '-', '=', '+', '*', '#']
    };
    values
        .iter()
        .map(|value| match value {
            Some(value) if value.is_finite() && *value >= 0.0 => {
                let ratio = if ceiling > 0.0 { value / ceiling } else { 0.0 };
                symbols[(ratio.clamp(0.0, 1.0) * 7.0).round() as usize]
            }
            _ => ' ',
        })
        .collect()
}
