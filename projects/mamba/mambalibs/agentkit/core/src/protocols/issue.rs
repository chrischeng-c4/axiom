//! IssueProtocol — domain contract for an issue/ticket.
//!
//! Consolidates `Issue`, `IssueState`, and `IssueSummary` from
//! `integrations/mod.rs` into a single, agent-facing type.

use crate::integrations::{Issue, IssueState, IssueSummary};
use serde::{Deserialize, Serialize};

/// Lifecycle status of an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Open,
    Closed,
    InProgress,
    Resolved,
    Wontfix,
}

impl Default for IssueStatus {
    fn default() -> Self {
        Self::Open
    }
}

impl From<IssueState> for IssueStatus {
    fn from(state: IssueState) -> Self {
        match state {
            IssueState::Open => IssueStatus::Open,
            IssueState::Closed => IssueStatus::Closed,
            IssueState::InProgress => IssueStatus::InProgress,
            IssueState::Resolved => IssueStatus::Resolved,
            IssueState::Wontfix => IssueStatus::Wontfix,
        }
    }
}

/// Priority level of an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuePriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for IssuePriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Domain contract for an issue/ticket.
///
/// Used by `RestructureIssueAgent`.  Consumers (Conductor, etc.) map
/// their ORM issue models to/from this type.
///
/// Consolidates: `Issue` + `IssueState` + `IssueSummary` from
/// `integrations/mod.rs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueProtocol {
    /// Unique issue identifier (platform-assigned number or string).
    pub id: String,
    /// Issue title / one-line summary.
    pub title: String,
    /// Full issue description/body.
    pub description: String,
    /// Current lifecycle status.
    pub status: IssueStatus,
    /// Priority level.
    pub priority: IssuePriority,
    /// Labels/tags attached to the issue.
    pub labels: Vec<String>,
    /// Acceptance criteria (done-when conditions).
    pub acceptance_criteria: Vec<String>,
}

impl IssueProtocol {
    /// Create a minimal protocol from an id and title.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            status: IssueStatus::default(),
            priority: IssuePriority::default(),
            labels: Vec::new(),
            acceptance_criteria: Vec::new(),
        }
    }
}

impl From<&Issue> for IssueProtocol {
    fn from(issue: &Issue) -> Self {
        Self {
            id: issue.id.clone(),
            title: issue.title.clone(),
            description: issue.body.clone(),
            status: IssueStatus::from(issue.state),
            priority: IssuePriority::default(),
            labels: issue.labels.clone(),
            acceptance_criteria: Vec::new(),
        }
    }
}

impl From<&IssueSummary> for IssueProtocol {
    fn from(summary: &IssueSummary) -> Self {
        Self {
            id: summary.id.clone(),
            title: summary.title.clone(),
            description: String::new(),
            status: IssueStatus::from(summary.state),
            priority: IssuePriority::default(),
            labels: summary.labels.clone(),
            acceptance_criteria: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::{Issue, IssueState};
    use chrono::Utc;

    fn make_issue() -> Issue {
        Issue {
            id: "42".to_string(),
            title: "Fix login bug".to_string(),
            body: "Users cannot log in with SSO.".to_string(),
            state: IssueState::Open,
            author: "alice".to_string(),
            labels: vec!["bug".to_string(), "P0".to_string()],
            assignees: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            url: "https://github.com/org/repo/issues/42".to_string(),
            comments: vec![],
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn test_from_issue() {
        let issue = make_issue();
        let proto = IssueProtocol::from(&issue);

        assert_eq!(proto.id, "42");
        assert_eq!(proto.title, "Fix login bug");
        assert_eq!(proto.description, "Users cannot log in with SSO.");
        assert_eq!(proto.status, IssueStatus::Open);
        assert_eq!(proto.labels, vec!["bug", "P0"]);
        assert!(proto.acceptance_criteria.is_empty());
    }

    #[test]
    fn test_issue_protocol_roundtrip() {
        let proto = IssueProtocol {
            id: "123".to_string(),
            title: "Add dark mode".to_string(),
            description: "Support dark mode across the UI.".to_string(),
            status: IssueStatus::InProgress,
            priority: IssuePriority::High,
            labels: vec!["enhancement".to_string()],
            acceptance_criteria: vec!["Toggle in settings works".to_string()],
        };

        let json = serde_json::to_string(&proto).unwrap();
        let decoded: IssueProtocol = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, "123");
        assert_eq!(decoded.status, IssueStatus::InProgress);
        assert_eq!(decoded.priority, IssuePriority::High);
        assert_eq!(decoded.acceptance_criteria.len(), 1);
    }

    #[test]
    fn test_issue_status_from_issue_state() {
        assert_eq!(IssueStatus::from(IssueState::Open), IssueStatus::Open);
        assert_eq!(IssueStatus::from(IssueState::Closed), IssueStatus::Closed);
        assert_eq!(
            IssueStatus::from(IssueState::InProgress),
            IssueStatus::InProgress
        );
        assert_eq!(
            IssueStatus::from(IssueState::Resolved),
            IssueStatus::Resolved
        );
        assert_eq!(IssueStatus::from(IssueState::Wontfix), IssueStatus::Wontfix);
    }
}
