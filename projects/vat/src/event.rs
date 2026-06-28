// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Append-only structured event log.
//!
//! Every state transition writes one JSON line to `events.jsonl`. This is the
//! "what happened" half of agent legibility: instead of scraping a console,
//! an agent reads typed events with timestamps and structured payloads. The
//! most recent few are surfaced in [`crate::state::VatState::events_tail`].

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One logged event.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts: DateTime<Utc>,
    pub kind: EventKind,
    /// Human/agent-readable summary.
    pub message: String,
    /// Optional structured payload (exit codes, paths, counts, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Closed set of event kinds. Keep it small and meaningful.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Created,
    Setup,
    RunStarted,
    RunFinished,
    Snapshot,
    Fork,
    Removed,
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
impl Event {
    pub fn new(kind: EventKind, message: impl Into<String>) -> Self {
        Event {
            ts: Utc::now(),
            kind,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Append one event to a vat's `events.jsonl`, creating it if needed.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
pub fn append(events_path: &Path, event: &Event) -> Result<()> {
    let line = serde_json::to_string(event).context("serialize event")?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(events_path)
        .with_context(|| format!("open {} for append", events_path.display()))?;
    writeln!(f, "{line}").context("write event line")?;
    Ok(())
}

/// Read up to the last `n` events (chronological order). Malformed lines are
/// skipped rather than failing the whole read — the log must stay legible
/// even if a write was once torn.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-event-rs.md#source
pub fn tail(events_path: &Path, n: usize) -> Result<Vec<Event>> {
    if !events_path.exists() {
        return Ok(Vec::new());
    }
    let f = std::fs::File::open(events_path)
        .with_context(|| format!("open {}", events_path.display()))?;
    let mut all: Vec<Event> = Vec::new();
    for line in BufReader::new(f).lines() {
        let line = line.context("read event line")?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(ev) = serde_json::from_str::<Event>(&line) {
            all.push(ev);
        }
    }
    let start = all.len().saturating_sub(n);
    Ok(all.split_off(start))
}
// CODEGEN-END
