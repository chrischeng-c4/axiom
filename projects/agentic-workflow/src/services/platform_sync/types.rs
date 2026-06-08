//! Common types for platform sync

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/types.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Payload for a spec issue.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/types.md#schema
#[derive(Debug, Clone)]
pub struct SpecPayload {
    pub spec_id: String,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub existing_issue: Option<u64>,
}

/// Result of syncing a spec issue.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecSyncResult {
    pub spec_id: String,
    pub status: SyncStatus,
    #[serde(default)]
    pub issue_url: Option<String>,
    #[serde(default)]
    pub issue_number: Option<u64>,
}

/// Payload to sync to platform.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/types.md#schema
#[derive(Debug, Clone)]
pub struct SyncPayload {
    pub change_id: String,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    /// Existing issue number (from frontmatter), None if new.
    pub existing_issue: Option<u64>,
    pub specs: Vec<SpecPayload>,
}

/// Result of a sync operation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub status: SyncStatus,
    /// Optional URL of the synced issue on the platform.
    #[serde(default)]
    pub issue_url: Option<String>,
    /// Optional platform-side issue number.
    #[serde(default)]
    pub issue_number: Option<u64>,
    /// Human-readable message describing the outcome.
    pub message: String,
    /// Results for spec issues.
    #[serde(default)]
    pub spec_results: Vec<SpecSyncResult>,
}

/// Status of the sync operation.
/// Created: new issue created on platform.
/// Updated: existing issue updated.
/// Partial: parent succeeded but some specs failed.
/// Error: sync failed.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    /// New issue created on platform.
    Created,
    /// Existing issue updated.
    Updated,
    /// Parent succeeded but some specs failed.
    Partial,
    /// Sync failed.
    Error,
}
// CODEGEN-END
