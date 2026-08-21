//! ChangeProtocol — domain contract for a code change (issue → spec → branch → PR).
//!
//! Fields `id`, `branch`, and `status` are conceptually related to the
//! `SessionState` in `storage/mod.rs`, but `ChangeProtocol` is a pure
//! domain contract with no persistence logic.

use serde::{Deserialize, Serialize};

/// Lifecycle status of a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    /// Not yet started.
    Pending,
    /// Actively being implemented.
    InProgress,
    /// Submitted for code review.
    Review,
    /// Merged into the target branch.
    Merged,
    /// Abandoned or rejected.
    Cancelled,
}

impl Default for ChangeStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Domain contract for a code change.
///
/// Ties together issues, specs, and an implementation branch into a
/// single unit of work consumed by `CodeAgent`.  Consumers (Conductor,
/// etc.) map their ORM change records to/from this type.
///
/// Related: fields `id`, `branch`, and `status` partially correspond
/// to `SessionState` in `storage/mod.rs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeProtocol {
    /// Unique change identifier (e.g. `agent-protocols`).
    pub id: String,
    /// Identifier of the parent project.
    pub project_id: String,
    /// Issue IDs addressed by this change.
    pub issue_ids: Vec<String>,
    /// Spec IDs produced for this change.
    pub spec_ids: Vec<String>,
    /// Git branch name (set once implementation begins).
    pub branch: Option<String>,
    /// Current lifecycle status.
    pub status: ChangeStatus,
}

impl ChangeProtocol {
    /// Create a new pending change.
    pub fn new(
        id: impl Into<String>,
        project_id: impl Into<String>,
        issue_ids: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            project_id: project_id.into(),
            issue_ids,
            spec_ids: Vec::new(),
            branch: None,
            status: ChangeStatus::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_protocol_roundtrip() {
        let change = ChangeProtocol {
            id: "agent-protocols".to_string(),
            project_id: "cclab-agent".to_string(),
            issue_ids: vec!["958".to_string()],
            spec_ids: vec!["agent-protocols-spec".to_string()],
            branch: Some("feat/agent-protocols".to_string()),
            status: ChangeStatus::InProgress,
        };

        let json = serde_json::to_string(&change).unwrap();
        let decoded: ChangeProtocol = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, "agent-protocols");
        assert_eq!(decoded.status, ChangeStatus::InProgress);
        assert_eq!(decoded.issue_ids, vec!["958"]);
        assert_eq!(decoded.branch, Some("feat/agent-protocols".to_string()));
    }

    #[test]
    fn test_change_protocol_new() {
        let change = ChangeProtocol::new(
            "my-change",
            "my-project",
            vec!["101".to_string(), "102".to_string()],
        );

        assert_eq!(change.status, ChangeStatus::Pending);
        assert!(change.branch.is_none());
        assert!(change.spec_ids.is_empty());
        assert_eq!(change.issue_ids.len(), 2);
    }

    #[test]
    fn test_change_status_default() {
        assert_eq!(ChangeStatus::default(), ChangeStatus::Pending);
    }
}
