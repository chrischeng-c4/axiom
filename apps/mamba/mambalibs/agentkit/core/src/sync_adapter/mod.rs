//! SyncAdapter trait — generic async abstraction for syncing protocol types
//! to and from third-party platforms.
//!
//! # Adapters
//!
//! | Adapter | Domains | Priority |
//! |---------|---------|----------|
//! | [`GitLabSyncAdapter`] | Issue, Code, Change (MR) | P1 |
//! | [`GitHubSyncAdapter`] | Issue, Code, Change (PR) | P1 |
//! | [`JiraSyncAdapter`] | Issue only | P2 |
//! | [`ConfluenceSyncAdapter`] | Spec only | P2 |
//! | [`GDocsSyncAdapter`] | Spec only | P2 |

mod confluence;
mod gdocs;
mod github;
mod gitlab;
mod jira;

pub use confluence::ConfluenceSyncAdapter;
pub use gdocs::GDocsSyncAdapter;
pub use github::GitHubSyncAdapter;
pub use gitlab::GitLabSyncAdapter;
pub use jira::JiraSyncAdapter;

use crate::error::NovaResult;
use crate::protocols::{ChangeProtocol, CodeIndexProtocol, IssueProtocol, SpecProtocol};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// The action taken during a sync operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncAction {
    /// A new resource was created on the platform.
    Created,
    /// An existing resource was updated on the platform.
    Updated,
    /// The remote resource was already in sync — no write was performed.
    NoChange,
}

/// Result returned by successful sync operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Platform-assigned identifier of the synced resource.
    pub external_id: String,
    /// URL to view the resource on the platform (if available).
    pub url: Option<String>,
    /// What action was taken.
    pub action: SyncAction,
}

impl SyncResult {
    /// Create a result for a newly created resource.
    pub fn created(external_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            external_id: external_id.into(),
            url: Some(url.into()),
            action: SyncAction::Created,
        }
    }

    /// Create a result for an updated resource.
    pub fn updated(external_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            external_id: external_id.into(),
            url: Some(url.into()),
            action: SyncAction::Updated,
        }
    }

    /// Create a no-change result.
    pub fn no_change(external_id: impl Into<String>) -> Self {
        Self {
            external_id: external_id.into(),
            url: None,
            action: SyncAction::NoChange,
        }
    }
}

/// Generic async abstraction for syncing protocol types to and from third-party platforms.
///
/// Each platform adapter implements only the methods relevant to its domain;
/// unsupported methods return [`NovaError::NotSupported`].
///
/// Auth credentials are injected at construction time — adapters hold no
/// global or mutable state.
#[async_trait]
pub trait SyncAdapter: Send + Sync {
    /// Push an issue to the platform (create or update).
    async fn push_issue(&self, issue: &IssueProtocol) -> NovaResult<SyncResult>;

    /// Pull an issue from the platform by its external ID.
    async fn pull_issue(&self, external_id: &str) -> NovaResult<IssueProtocol>;

    /// Push a spec to the platform (create or update).
    async fn push_spec(&self, spec: &SpecProtocol) -> NovaResult<SyncResult>;

    /// Push a change (PR / MR) to the platform.
    async fn push_change(&self, change: &ChangeProtocol) -> NovaResult<SyncResult>;

    /// Pull a code index entry from the platform for the given module path.
    async fn pull_code_index(&self, path: &str) -> NovaResult<CodeIndexProtocol>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_result_created() {
        let r = SyncResult::created("42", "https://example.com/issues/42");
        assert_eq!(r.external_id, "42");
        assert_eq!(r.action, SyncAction::Created);
        assert!(r.url.is_some());
    }

    #[test]
    fn test_sync_result_updated() {
        let r = SyncResult::updated("99", "https://example.com/issues/99");
        assert_eq!(r.action, SyncAction::Updated);
        assert!(r.url.is_some());
    }

    #[test]
    fn test_sync_result_no_change() {
        let r = SyncResult::no_change("7");
        assert_eq!(r.external_id, "7");
        assert!(r.url.is_none());
        assert_eq!(r.action, SyncAction::NoChange);
    }

    #[test]
    fn test_sync_result_roundtrip() {
        let r = SyncResult::created("123", "https://example.com");
        let json = serde_json::to_string(&r).unwrap();
        let decoded: SyncResult = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.external_id, "123");
        assert_eq!(decoded.action, SyncAction::Created);
    }
}
