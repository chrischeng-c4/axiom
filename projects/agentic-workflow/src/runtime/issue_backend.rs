// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
// CODEGEN-BEGIN
//! Issue subsystem abstraction. cue's runtime talks to issues via this
//! trait; concrete impls back it with local SDD files, GitHub Issues
//! (gh CLI), GitLab Issues (glab CLI), or Jira REST API.
//!
//! @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md
//!
//! Slice-1 contract:
//! - `create / list / read` MUST work on every backend
//! - `update / close` are required for `local`; remote backends MAY return
//!   `BackendError::Unsupported` (per issue R8 — full SDD CRRR fill
//!   semantics stay scoped to local in slice 1).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#schema
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#changes
/// Selects which backend `Session` constructs at startup. Matches the
/// `[issue].backend` key in `.cue/config.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    #[default]
    Local,
    GitHub,
    GitLab,
    Jira,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl BackendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BackendKind::Local => "local",
            BackendKind::GitHub => "github",
            BackendKind::GitLab => "gitlab",
            BackendKind::Jira => "jira",
        }
    }
}

/// Opaque platform-specific issue identifier:
/// - local: slug (e.g. "add-metrics-dashboard")
/// - github / gitlab: numeric string ("123")
/// - jira: project-prefixed ("PROJ-456")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct IssueId(pub String);

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl IssueId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl std::fmt::Display for IssueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub enum IssueState {
    Open,
    Closed,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
impl Default for IssueState {
    fn default() -> Self {
        IssueState::Open
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct ListFilter {
    #[serde(default)]
    pub state: IssueState,
    #[serde(default)]
    pub labels: Vec<String>,
}

/// List-view record — minimum fields for the TUI issue picker row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct IssueRef {
    pub id: IssueId,
    pub title: String,
    pub state: IssueState,
    #[serde(default)]
    pub labels: Vec<String>,
}

/// Full issue record returned by `read`. `frontmatter` is populated only
/// for the local backend (remote platforms have no SDD YAML frontmatter
/// concept; their backends return an empty map).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub struct IssueBody {
    pub id: IssueId,
    pub title: String,
    pub body_md: String,
    #[serde(default)]
    pub frontmatter: BTreeMap<String, serde_json::Value>,
}

/// Discriminated error type. `Unsupported` is the canonical return for
/// `update` / `close` on slice-1 remote backends (R8).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub enum BackendError {
    #[error("operation not supported by this backend in slice 1")]
    Unsupported,

    #[error("issue not found: {0}")]
    NotFound(IssueId),

    #[error("auth failed: {0}")]
    Auth(String),

    #[error("network/transport error: {0}")]
    Network(String),

    #[error("internal backend error: {0}")]
    Internal(String),
}

/// Issue subsystem trait — the abstraction `Session` holds via
/// `Arc<dyn IssueBackend>`. Object-safe.
#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#source
pub trait IssueBackend: Send + Sync {
    /// Identifies which concrete impl this is (for the TUI status bar
    /// and SessionEvent payloads).
    fn backend_kind(&self) -> BackendKind;

    /// Create a new issue with the given title. Returns the platform's
    /// canonical issue id.
    async fn create(&self, title: &str) -> Result<IssueId, BackendError>;

    /// List issues matching `filter`. The `state` field defaults to
    /// `Open`; `labels` empty means no label filter.
    async fn list(&self, filter: &ListFilter) -> Result<Vec<IssueRef>, BackendError>;

    /// Read the full issue body for a given id.
    async fn read(&self, id: &IssueId) -> Result<IssueBody, BackendError>;

    /// Update a section of the issue body. Slice 1: required on local;
    /// remote backends MAY return `Unsupported`.
    async fn update(&self, id: &IssueId, section: &str, body: &str) -> Result<(), BackendError>;

    /// Close the issue with an optional message. Slice 1: required on
    /// local; remote backends MAY return `Unsupported`.
    async fn close(&self, id: &IssueId, message: Option<&str>) -> Result<(), BackendError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_kind_serde_roundtrip_lowercase() {
        let k = BackendKind::GitHub;
        let s = serde_json::to_string(&k).unwrap();
        assert_eq!(s, "\"github\"");
        let back: BackendKind = serde_json::from_str(&s).unwrap();
        assert_eq!(back, BackendKind::GitHub);
    }

    #[test]
    fn backend_kind_default_is_local() {
        assert_eq!(BackendKind::default(), BackendKind::Local);
    }

    #[test]
    fn list_filter_default_state_is_open() {
        let f = ListFilter::default();
        assert_eq!(f.state, IssueState::Open);
        assert!(f.labels.is_empty());
    }

    #[test]
    fn issue_id_display_round_trip() {
        let id = IssueId::new("add-metrics-dashboard");
        assert_eq!(format!("{id}"), "add-metrics-dashboard");
    }

    #[test]
    fn backend_error_display() {
        let e = BackendError::Unsupported;
        assert!(e.to_string().contains("not supported"));
        let nf = BackendError::NotFound(IssueId::new("abc"));
        assert!(nf.to_string().contains("abc"));
    }
}

// CODEGEN-END
