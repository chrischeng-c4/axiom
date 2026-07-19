// HANDWRITE-BEGIN gap="sift-projection-model" tracker="1660" reason="Define descriptors, checkpoints, state envelopes, replay jobs, and projection-lag errors."
use std::{collections::BTreeMap, fmt};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const PROJECTION_STATE_FORMAT_VERSION: u16 = 1;
pub const SIFT_COMMAND_FORMAT_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ProjectionDescriptor {
    pub name: String,
    pub schema_version: u32,
    pub retention: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ProjectionCheckpoint {
    pub projection: String,
    pub schema_version: u32,
    pub cursor: u64,
    pub event_id: Option<String>,
    pub state_sha256: String,
    pub updated_at: String,
}

impl ProjectionCheckpoint {
    pub fn empty(descriptor: &ProjectionDescriptor) -> Self {
        Self {
            projection: descriptor.name.clone(),
            schema_version: descriptor.schema_version,
            cursor: 0,
            event_id: None,
            state_sha256: String::new(),
            updated_at: now(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ProjectionStateEnvelope {
    pub format_version: u16,
    pub checkpoint: ProjectionCheckpoint,
    pub state_encoding: String,
    pub state_base64: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReplayState {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ReplayJob {
    pub id: String,
    pub projection: String,
    pub state: ReplayState,
    pub requested_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub source_cursor: u64,
    pub rebuilt_cursor: Option<u64>,
    pub live_digest: Option<String>,
    pub rebuilt_digest: Option<String>,
    pub equal: Option<bool>,
    pub error: Option<String>,
    /// Commit index of the most recent durable lifecycle transition.
    #[serde(default)]
    pub commit_index: u64,
}

impl ReplayJob {
    pub fn pending(id: String, projection: String, source_cursor: u64) -> Self {
        Self {
            id,
            projection,
            state: ReplayState::Pending,
            requested_at: now(),
            started_at: None,
            completed_at: None,
            source_cursor,
            rebuilt_cursor: None,
            live_digest: None,
            rebuilt_digest: None,
            equal: None,
            error: None,
            commit_index: 0,
        }
    }

    pub fn mark_running(&mut self) {
        self.state = ReplayState::Running;
        self.started_at = Some(now());
        self.completed_at = None;
        self.error = None;
    }

    pub fn mark_completed(&mut self, comparison: RebuildComparison) {
        self.state = ReplayState::Completed;
        self.completed_at = Some(now());
        self.source_cursor = comparison.source_cursor;
        self.rebuilt_cursor = Some(comparison.rebuilt_cursor);
        self.live_digest = Some(comparison.live_digest);
        self.rebuilt_digest = Some(comparison.rebuilt_digest);
        self.equal = Some(comparison.equal);
        self.error = None;
    }

    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.state = ReplayState::Failed;
        self.completed_at = Some(now());
        self.equal = Some(false);
        self.error = Some(error.into());
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SiftControlState {
    pub format_version: u16,
    pub applied_index: u64,
    pub replay_jobs: BTreeMap<String, ReplayJob>,
    #[serde(default)]
    pub error_lifecycles: BTreeMap<String, ErrorLifecycleV1>,
    #[serde(default)]
    pub audit_legal_holds: BTreeMap<String, AuditLegalHoldV1>,
    #[serde(default)]
    pub audit_exports: BTreeMap<String, AuditExportManifestV1>,
}

impl Default for SiftControlState {
    fn default() -> Self {
        Self {
            format_version: SIFT_COMMAND_FORMAT_VERSION,
            applied_index: 0,
            replay_jobs: BTreeMap::new(),
            error_lifecycles: BTreeMap::new(),
            audit_legal_holds: BTreeMap::new(),
            audit_exports: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorLifecycleState {
    Open,
    Acknowledged,
    Resolved,
    Muted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ErrorLifecycleV1 {
    pub project: String,
    pub fingerprint: String,
    pub state: ErrorLifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted_until: Option<String>,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub occurrence_cursor: u64,
    pub updated_at: String,
    #[serde(default)]
    pub commit_index: u64,
}

impl ErrorLifecycleV1 {
    pub fn key(&self) -> String {
        error_lifecycle_key(&self.project, &self.fingerprint)
    }
}

pub fn error_lifecycle_key(project: &str, fingerprint: &str) -> String {
    format!("{project}\u{1f}{fingerprint}")
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct AuditLegalHoldV1 {
    pub id: String,
    pub project: String,
    pub start_time: String,
    pub end_time: String,
    pub reason: String,
    pub actor: String,
    pub active: bool,
    pub updated_at: String,
    #[serde(default)]
    pub commit_index: u64,
}

impl AuditLegalHoldV1 {
    pub fn key(&self) -> String {
        audit_control_key(&self.project, &self.id)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct AuditExportManifestV1 {
    pub id: String,
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    pub record_count: u64,
    pub content_sha256: String,
    pub actor: String,
    pub exported_at: String,
    #[serde(default)]
    pub commit_index: u64,
}

impl AuditExportManifestV1 {
    pub fn key(&self) -> String {
        audit_control_key(&self.project, &self.id)
    }
}

pub fn audit_control_key(project: &str, id: &str) -> String {
    format!("{project}\u{1f}{id}")
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ProjectionLag {
    pub error: String,
    pub projection: String,
    pub required_cursor: u64,
    pub current_cursor: u64,
    pub retryable: bool,
    pub retry_after_seconds: u64,
}

impl ProjectionLag {
    pub fn new(
        projection: impl Into<String>,
        required_cursor: u64,
        current_cursor: u64,
        retry_after_seconds: u64,
    ) -> Self {
        Self {
            error: "projection_lag".into(),
            projection: projection.into(),
            required_cursor,
            current_cursor,
            retryable: true,
            retry_after_seconds,
        }
    }
}

impl fmt::Display for ProjectionLag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "projection {} is at cursor {}, requires {}",
            self.projection, self.current_cursor, self.required_cursor
        )
    }
}

impl std::error::Error for ProjectionLag {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RebuildComparison {
    pub source_cursor: u64,
    pub rebuilt_cursor: u64,
    pub live_digest: String,
    pub rebuilt_digest: String,
    pub equal: bool,
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

// HANDWRITE-END
