// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Read-only console / network panels for the selected e2e step (#2887).
//!
//! `jet e2e open` shows one selected step at a time. The reviewer
//! often needs to correlate that step with the console messages and
//! network requests that occurred around it. This module owns the
//! projection: given the recorded [`E2eStepContext`] for the step, it
//! returns paged, classified row data the UI can render directly —
//! along with explicit empty-state markers so a missing record never
//! masquerades as "no logs produced".
//!
//! Live request editing and live console eval are out of scope.

use crate::e2e::{E2eConsoleEntry, E2eNetworkEntry, E2eStepContext};
use serde::{Deserialize, Serialize};

/// Stable schema tag for [`StepPanels`].
pub const STEP_PANELS_SCHEMA_VERSION: &str = "jet.e2e.step-panels.v1";

/// Console row severity. We collapse the wider browser console set
/// down to four buckets the UI styles distinctly.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsoleSeverity {
    Error,
    Warn,
    Info,
    Debug,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ConsoleSeverity {
    pub fn classify(level: &str) -> Self {
        match level.to_ascii_lowercase().as_str() {
            "error" | "err" | "severe" | "fatal" => Self::Error,
            "warn" | "warning" => Self::Warn,
            "debug" | "verbose" | "trace" => Self::Debug,
            _ => Self::Info,
        }
    }
}

/// Read-only console row the UI renders.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsoleRow {
    pub severity: ConsoleSeverity,
    pub level: String,
    pub text: String,
    pub ts_ms: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl From<&E2eConsoleEntry> for ConsoleRow {
    fn from(entry: &E2eConsoleEntry) -> Self {
        Self {
            severity: ConsoleSeverity::classify(&entry.level),
            level: entry.level.clone(),
            text: entry.text.clone(),
            ts_ms: entry.ts_ms,
        }
    }
}

/// Network row outcome. `Pending` is what the panel shows when the
/// recorded request never matured to a response (no status, no end
/// timestamp).
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkOutcome {
    Success,
    ClientError,
    ServerError,
    Pending,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl NetworkOutcome {
    pub fn classify(status: Option<u16>, ts_end_ms: Option<u64>) -> Self {
        match status {
            Some(s) if (200..400).contains(&s) => Self::Success,
            Some(s) if (400..500).contains(&s) => Self::ClientError,
            Some(s) if s >= 500 => Self::ServerError,
            Some(_) => Self::Pending,
            None => {
                if ts_end_ms.is_some() {
                    Self::Pending
                } else {
                    Self::Pending
                }
            }
        }
    }
}

/// Read-only network row the UI renders.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkRow {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub ts_start_ms: u64,
    pub ts_end_ms: Option<u64>,
    pub outcome: NetworkOutcome,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl From<&E2eNetworkEntry> for NetworkRow {
    fn from(entry: &E2eNetworkEntry) -> Self {
        Self {
            request_id: entry.request_id.clone(),
            method: entry.method.clone(),
            url: entry.url.clone(),
            status: entry.status,
            ts_start_ms: entry.ts_start_ms,
            ts_end_ms: entry.ts_end_ms,
            outcome: NetworkOutcome::classify(entry.status, entry.ts_end_ms),
        }
    }
}

/// Empty-state marker so the UI can distinguish "no rows recorded"
/// from "step had no logs". The variant names match the reviewer-
/// facing copy.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PanelEmptyState {
    NoRecordsForStep,
}

/// One panel's projected rows or its empty state.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PanelProjection<R> {
    Rows { rows: Vec<R> },
    Empty { state: PanelEmptyState },
}

impl<R> PanelProjection<R> {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty { .. })
    }

    pub fn row_count(&self) -> usize {
        match self {
            Self::Rows { rows } => rows.len(),
            Self::Empty { .. } => 0,
        }
    }
}

/// Wrapper the inspector hands to the UI for a selected step.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepPanels {
    pub schema_version: String,
    pub console: PanelProjection<ConsoleRow>,
    pub network: PanelProjection<NetworkRow>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl StepPanels {
    /// Project the recorded step context into panel rows.
    pub fn from_context(context: &E2eStepContext) -> Self {
        let console = if context.console.is_empty() {
            PanelProjection::Empty {
                state: PanelEmptyState::NoRecordsForStep,
            }
        } else {
            PanelProjection::Rows {
                rows: context.console.iter().map(ConsoleRow::from).collect(),
            }
        };
        let network = if context.network.is_empty() {
            PanelProjection::Empty {
                state: PanelEmptyState::NoRecordsForStep,
            }
        } else {
            PanelProjection::Rows {
                rows: context.network.iter().map(NetworkRow::from).collect(),
            }
        };
        Self {
            schema_version: STEP_PANELS_SCHEMA_VERSION.to_string(),
            console,
            network,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::e2e::{E2eConsoleEntry, E2eNetworkEntry, E2eStepContext};

    fn context_with_rows() -> E2eStepContext {
        let mut c = E2eStepContext::default();
        c.console.push(E2eConsoleEntry {
            level: "error".into(),
            text: "Uncaught ReferenceError: foo".into(),
            ts_ms: 1_000,
        });
        c.console.push(E2eConsoleEntry {
            level: "warn".into(),
            text: "deprecated API".into(),
            ts_ms: 1_010,
        });
        c.network.push(E2eNetworkEntry {
            request_id: "r-1".into(),
            method: "GET".into(),
            url: "https://example.test/api/x".into(),
            status: Some(200),
            ts_start_ms: 900,
            ts_end_ms: Some(950),
        });
        c.network.push(E2eNetworkEntry {
            request_id: "r-2".into(),
            method: "POST".into(),
            url: "https://example.test/api/y".into(),
            status: Some(500),
            ts_start_ms: 960,
            ts_end_ms: Some(1_005),
        });
        c
    }

    #[test]
    fn fixture_evidence_renders_console_and_network_rows() {
        // Stop condition (#2887): a fixture evidence bundle renders
        // console and network rows.
        let panels = StepPanels::from_context(&context_with_rows());
        assert_eq!(panels.console.row_count(), 2);
        assert_eq!(panels.network.row_count(), 2);
        match &panels.console {
            PanelProjection::Rows { rows } => {
                assert_eq!(rows[0].severity, ConsoleSeverity::Error);
                assert_eq!(rows[1].severity, ConsoleSeverity::Warn);
            }
            other => panic!("expected rows, got {other:?}"),
        }
    }

    #[test]
    fn missing_records_show_explicit_empty_state() {
        // Stop condition (#2887): missing records show explicit
        // empty states, not silent collapse.
        let panels = StepPanels::from_context(&E2eStepContext::default());
        assert!(panels.console.is_empty());
        assert!(panels.network.is_empty());
        match &panels.console {
            PanelProjection::Empty { state } => {
                assert_eq!(*state, PanelEmptyState::NoRecordsForStep);
            }
            other => panic!("expected empty, got {other:?}"),
        }
    }

    #[test]
    fn console_severity_classification_buckets_unknown_to_info() {
        assert_eq!(ConsoleSeverity::classify("ERROR"), ConsoleSeverity::Error);
        assert_eq!(ConsoleSeverity::classify("Warning"), ConsoleSeverity::Warn);
        assert_eq!(ConsoleSeverity::classify("info"), ConsoleSeverity::Info);
        assert_eq!(ConsoleSeverity::classify("debug"), ConsoleSeverity::Debug);
        assert_eq!(ConsoleSeverity::classify("log"), ConsoleSeverity::Info);
        assert_eq!(ConsoleSeverity::classify("trace"), ConsoleSeverity::Debug);
    }

    #[test]
    fn network_outcome_classifies_status_ranges() {
        assert_eq!(
            NetworkOutcome::classify(Some(204), Some(1)),
            NetworkOutcome::Success
        );
        assert_eq!(
            NetworkOutcome::classify(Some(404), Some(1)),
            NetworkOutcome::ClientError
        );
        assert_eq!(
            NetworkOutcome::classify(Some(503), Some(1)),
            NetworkOutcome::ServerError
        );
        assert_eq!(
            NetworkOutcome::classify(None, None),
            NetworkOutcome::Pending
        );
    }

    #[test]
    fn panels_round_trip_through_json() {
        let panels = StepPanels::from_context(&context_with_rows());
        let json = serde_json::to_string(&panels).unwrap();
        let back: StepPanels = serde_json::from_str(&json).unwrap();
        assert_eq!(back, panels);
        assert!(json.contains("\"outcome\":\"server-error\""), "{json}");
        assert!(json.contains("\"severity\":\"error\""), "{json}");
    }
}
// CODEGEN-END
