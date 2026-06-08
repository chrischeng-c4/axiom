// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Controlled Jet Browser session lifecycle for E2E cases (#2876).
//!
//! `jet e2e run` needs a controlled browser session to drive the AUT
//! through one case at a time. That session is *separate* from the
//! desktop review shell: in run mode there is no shell at all, and in
//! open mode the browser target is a second visible window so the
//! reviewer can watch product behaviour without the shell stealing
//! focus.
//!
//! This module owns the lifecycle contract — the state machine, the
//! per-step events, and the cleanup record that lands in evidence —
//! without binding to any specific browser driver. The CDP/headless
//! glue lives in `browser` / `cdp_driver`; this layer is what the
//! runner records and asserts on.

use crate::e2e_lifecycle::AutPhaseOutcome;
use serde::{Deserialize, Serialize};

/// Stable schema tag for [`BrowserSessionRecord`].
pub const BROWSER_SESSION_SCHEMA_VERSION: &str = "jet.e2e.browser-session.v1";

/// Driver flavour the runner asked for. Stored verbatim in evidence
/// so a reviewer can tell which engine produced the trace.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserDriver {
    /// Controlled headless Chromium over CDP (default).
    Chromium,
    /// Reserved — controlled Firefox would land here.
    Firefox,
    /// Reserved — controlled WebKit would land here.
    Webkit,
}

/// Where the browser session is anchored. Run mode launches its own
/// process; open mode attaches to the visible browser target the
/// review shell already opened.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionAnchor {
    Launched,
    Attached,
}

/// Lifecycle state of the controlled session. The runner moves
/// linearly through these states; nothing skips.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionState {
    Idle,
    Launching,
    Ready,
    InCase,
    ClosingCase,
    Closed,
    Failed,
}

/// One lifecycle event written into the session record so evidence
/// can show exactly when the session opened, ran the case, and
/// closed.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SessionEvent {
    Launching,
    Ready {
        connection_label: String,
    },
    CaseStarted {
        case_id: String,
    },
    CaseFinished {
        case_id: String,
        outcome: AutPhaseOutcome,
    },
    ClosingCase,
    Closed {
        graceful: bool,
    },
    Failed {
        reason: String,
    },
}

/// Configuration the runner passes when asking for a session.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserSessionRequest {
    pub driver: BrowserDriver,
    pub anchor: SessionAnchor,
    /// Tag the runner prints next to the connection — driver name
    /// plus a short id, e.g. `chromium#42` so multiple runs can be
    /// distinguished in logs.
    pub connection_label: String,
    /// True when the controlled browser should be visible to a
    /// reviewer (open mode). Run mode keeps this `false`.
    pub headed: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl BrowserSessionRequest {
    pub fn run_mode_default() -> Self {
        Self {
            driver: BrowserDriver::Chromium,
            anchor: SessionAnchor::Launched,
            connection_label: "chromium#run".into(),
            headed: false,
        }
    }
}

/// Append-only record of a single session, from idle to closed. The
/// runner clones this into the case's evidence step context so
/// cleanup is auditable.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserSessionRecord {
    pub schema_version: String,
    pub request: BrowserSessionRequest,
    pub state: SessionState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<SessionEvent>,
    pub cleanup_reported: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl BrowserSessionRecord {
    pub fn new(request: BrowserSessionRequest) -> Self {
        Self {
            schema_version: BROWSER_SESSION_SCHEMA_VERSION.to_string(),
            request,
            state: SessionState::Idle,
            events: Vec::new(),
            cleanup_reported: false,
        }
    }

    pub fn launch(&mut self) {
        self.state = SessionState::Launching;
        self.events.push(SessionEvent::Launching);
    }

    pub fn ready(&mut self) {
        self.state = SessionState::Ready;
        self.events.push(SessionEvent::Ready {
            connection_label: self.request.connection_label.clone(),
        });
    }

    pub fn enter_case(&mut self, case_id: impl Into<String>) {
        self.state = SessionState::InCase;
        self.events.push(SessionEvent::CaseStarted {
            case_id: case_id.into(),
        });
    }

    pub fn finish_case(&mut self, case_id: impl Into<String>, outcome: AutPhaseOutcome) {
        self.state = SessionState::ClosingCase;
        self.events.push(SessionEvent::CaseFinished {
            case_id: case_id.into(),
            outcome,
        });
        self.events.push(SessionEvent::ClosingCase);
    }

    pub fn close(&mut self, graceful: bool) {
        self.state = SessionState::Closed;
        self.events.push(SessionEvent::Closed { graceful });
        self.cleanup_reported = true;
    }

    pub fn fail(&mut self, reason: impl Into<String>) {
        self.state = SessionState::Failed;
        self.events.push(SessionEvent::Failed {
            reason: reason.into(),
        });
        self.cleanup_reported = true;
    }

    /// True when the record is in a terminal state (Closed or Failed).
    /// Evidence assertions use this to check the runner did not leak
    /// a session.
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, SessionState::Closed | SessionState::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_mode_default_launches_headless_chromium() {
        let req = BrowserSessionRequest::run_mode_default();
        assert_eq!(req.driver, BrowserDriver::Chromium);
        assert_eq!(req.anchor, SessionAnchor::Launched);
        assert!(!req.headed);
    }

    #[test]
    fn smoke_one_case_open_then_close_records_full_lifecycle() {
        // Stop condition (#2876): a smoke run opens and closes one
        // controlled browser session, and cleanup is reported in
        // evidence.
        let mut record = BrowserSessionRecord::new(BrowserSessionRequest::run_mode_default());
        record.launch();
        record.ready();
        record.enter_case("flows/buy::buyer");
        record.finish_case("flows/buy::buyer", AutPhaseOutcome::Succeeded);
        record.close(true);

        assert!(record.is_terminal());
        assert!(record.cleanup_reported);
        assert_eq!(record.state, SessionState::Closed);

        let kinds: Vec<&str> = record
            .events
            .iter()
            .map(|e| match e {
                SessionEvent::Launching => "launching",
                SessionEvent::Ready { .. } => "ready",
                SessionEvent::CaseStarted { .. } => "case-started",
                SessionEvent::CaseFinished { .. } => "case-finished",
                SessionEvent::ClosingCase => "closing-case",
                SessionEvent::Closed { .. } => "closed",
                SessionEvent::Failed { .. } => "failed",
            })
            .collect();
        assert_eq!(
            kinds,
            vec![
                "launching",
                "ready",
                "case-started",
                "case-finished",
                "closing-case",
                "closed",
            ],
        );
    }

    #[test]
    fn open_mode_attach_uses_separate_anchor() {
        // Stop condition (#2876): browser session stays separate from
        // the desktop review shell — open mode attaches rather than
        // launches.
        let req = BrowserSessionRequest {
            driver: BrowserDriver::Chromium,
            anchor: SessionAnchor::Attached,
            connection_label: "chromium#open".into(),
            headed: true,
        };
        let mut record = BrowserSessionRecord::new(req);
        record.launch();
        record.ready();
        assert_eq!(record.request.anchor, SessionAnchor::Attached);
        assert!(record.request.headed);
    }

    #[test]
    fn failed_session_still_reports_cleanup() {
        let mut record = BrowserSessionRecord::new(BrowserSessionRequest::run_mode_default());
        record.launch();
        record.fail("driver crashed");
        assert!(record.is_terminal());
        assert!(record.cleanup_reported);
        assert_eq!(record.state, SessionState::Failed);
    }

    #[test]
    fn finish_case_with_failed_outcome_records_outcome() {
        let mut record = BrowserSessionRecord::new(BrowserSessionRequest::run_mode_default());
        record.launch();
        record.ready();
        record.enter_case("case-1");
        record.finish_case("case-1", AutPhaseOutcome::Failed);
        record.close(false);
        let outcome = record.events.iter().find_map(|e| match e {
            SessionEvent::CaseFinished { outcome, .. } => Some(*outcome),
            _ => None,
        });
        assert_eq!(outcome, Some(AutPhaseOutcome::Failed));
    }

    #[test]
    fn record_round_trips_through_json() {
        let mut record = BrowserSessionRecord::new(BrowserSessionRequest::run_mode_default());
        record.launch();
        record.ready();
        record.close(true);
        let json = serde_json::to_string(&record).unwrap();
        let back: BrowserSessionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
        assert!(json.contains("\"kind\":\"launching\""), "{json}");
        assert!(json.contains("\"anchor\":\"launched\""), "{json}");
    }
}
// CODEGEN-END
