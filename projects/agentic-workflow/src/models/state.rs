// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/state.md#source
// CODEGEN-BEGIN
//! State types for SDD STATE.yaml
//!
//! Defines the state machine and telemetry structures:
//! - State (root STATE.yaml)
//! - StatePhase (workflow phases)
//! - DagState, DelegationGuard
//! - Telemetry, ChecksumEntry, ValidationEntry

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#source
use std::collections::HashMap;

// =============================================================================
// DelegationGuard
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Workflow phase values for the SDD state machine. NOTE custom Serialize/Deserialize impls live outside CODEGEN — do not derive serde here.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatePhase {
    ChangeInited,
    ChangeSpecCreated,
    ChangeSpecReviewed,
    ChangeSpecRevised,
    ChangeImplementationCreated,
    ChangeImplementationReviewed,
    ChangeImplementationRevised,
    TestCheck,
    DocsCheck,
    DocsCreated,
    DocsReviewed,
    DocsRevised,
    ChangeMergeCreated,
    ChangeMergeReviewed,
    ChangeMergeRevised,
    ChangeArchived,
    ChangeRejected,
}

/// Validation mode.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationMode {
    /// Normal validation mode (default).
    #[default]
    Normal,
    /// Strict mode: warnings treated as errors.
    Strict,
}

/// Phase guard active during delegated agent execution.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationGuard {
    /// Phases the delegated agent is allowed to set.
    pub allowed_phases: Vec<StatePhase>,
    /// Phase snapshot before delegation started.
    pub phase_before: StatePhase,
    /// Action being delegated.
    pub action: String,
    /// When delegation started.
    pub started_at: DateTime<Utc>,
}

/// State file for tracking change progress.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Change identifier.
    pub change_id: String,
    /// Schema version for forward compatibility.
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// Current workflow phase.
    pub phase: StatePhase,
    /// Current iteration.
    #[serde(default = "default_iteration")]
    pub iteration: u32,
    /// Last action performed.
    #[serde(default)]
    pub last_action: Option<String>,
    /// Session identifier.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Checksums for artifact staleness detection.
    #[serde(default)]
    pub checksums: HashMap<String, ChecksumEntry>,
    /// Validation history.
    #[serde(default)]
    pub validations: Vec<ValidationEntry>,
    /// Git workflow.
    #[serde(default)]
    pub git_workflow: Option<String>,
    /// Revision counts per phase.
    #[serde(default)]
    pub revision_counts: HashMap<String, u32>,
    /// Current task ID.
    #[serde(default)]
    pub current_task_id: Option<String>,
    /// Per-task revision counts.
    #[serde(default)]
    pub task_revisions: HashMap<String, u32>,
    /// Per-spec implementation phase tracking.
    #[serde(default)]
    pub impl_spec_phase: HashMap<String, String>,
    /// LLM call telemetry.
    #[serde(default)]
    pub telemetry: Option<Telemetry>,
    /// DAG state for multi-issue workflows.
    #[serde(default)]
    pub dag: Option<DagState>,
    /// Delegation guard for agent phase protection.
    #[serde(default)]
    pub delegation_guard: Option<DelegationGuard>,
    /// Git branch name for this change.
    #[serde(default)]
    pub branch: Option<String>,
}

/// DAG state for multi-issue topological ordering.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagState {
    /// Ordered list of issues.
    #[serde(default)]
    pub issues: Vec<DagIssue>,
    /// Index of current issue being processed.
    #[serde(default)]
    pub current_index: usize,
    /// Whether the DAG is complete.
    #[serde(default)]
    pub complete: bool,
}

/// A single issue in the DAG.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagIssue {
    /// Issue number.
    pub number: u64,
    /// Issue title.
    #[serde(default)]
    pub title: String,
    /// Issues this one depends on.
    #[serde(default)]
    pub depends: Vec<u64>,
    /// Issues that depend on this one.
    #[serde(default)]
    pub dependents: Vec<u64>,
    /// Whether this issue has been processed.
    #[serde(default)]
    pub processed: bool,
    /// Issues this one is blocked by.
    #[serde(default)]
    pub blocked_by: Vec<u64>,
}

/// Checksum entry with validation timestamp.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumEntry {
    /// Hash value.
    pub hash: String,
    /// Timestamp when validated.
    #[serde(default)]
    pub validated_at: Option<DateTime<Utc>>,
}

/// Validation history entry.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationEntry {
    /// Validation step name.
    pub step: String,
    /// Timestamp.
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    /// Rules version.
    #[serde(default)]
    pub rules_version: Option<String>,
    /// Rules hash.
    #[serde(default)]
    pub rules_hash: Option<String>,
    /// Validation mode.
    #[serde(default)]
    pub mode: Option<ValidationMode>,
    /// Validation result.
    #[serde(default)]
    pub result: Option<ValidationResult>,
    /// Errors.
    #[serde(default)]
    pub errors: Vec<String>,
    /// Warnings.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Validation result.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed.
    pub valid: bool,
    /// High severity count.
    #[serde(default)]
    pub high: u32,
    /// Medium severity count.
    #[serde(default)]
    pub medium: u32,
    /// Low severity count.
    #[serde(default)]
    pub low: u32,
    /// Optional verdict.
    #[serde(default)]
    pub verdict: Option<String>,
    /// Number of issues parsed.
    #[serde(default)]
    pub issues_parsed: Option<u32>,
}

/// LLM telemetry.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Telemetry {
    /// LLM calls.
    #[serde(default)]
    pub calls: Vec<LlmCall>,
    /// Total cost in USD.
    #[serde(default)]
    pub total_cost_usd: f64,
    /// Total input tokens.
    #[serde(default)]
    pub total_tokens_in: u64,
    /// Total output tokens.
    #[serde(default)]
    pub total_tokens_out: u64,
}

/// Single LLM call telemetry.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCall {
    /// Step identifier.
    pub step: String,
    /// SDD version.
    #[serde(default)]
    pub sdd_version: Option<String>,
    /// Model identifier.
    #[serde(default)]
    pub model: Option<String>,
    /// Input tokens.
    #[serde(default)]
    pub tokens_in: Option<u64>,
    /// Output tokens.
    #[serde(default)]
    pub tokens_out: Option<u64>,
    /// Cost in USD.
    #[serde(default)]
    pub cost_usd: Option<f64>,
    /// Duration in milliseconds.
    #[serde(default)]
    pub duration_ms: Option<u64>,
    /// Timestamp.
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
}

// =============================================================================
// State (STATE.yaml)
// =============================================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#source
fn default_schema_version() -> String {
    "2.0".to_string()
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#source
fn default_iteration() -> u32 {
    1
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#source
impl Default for State {
    fn default() -> Self {
        Self {
            change_id: String::new(),
            schema_version: default_schema_version(),
            created_at: None,
            updated_at: None,
            phase: StatePhase::ChangeInited,
            iteration: 1,
            last_action: None,
            session_id: None,
            git_workflow: None,
            checksums: HashMap::new(),
            validations: Vec::new(),
            revision_counts: HashMap::new(),
            current_task_id: None,
            task_revisions: HashMap::new(),
            impl_spec_phase: HashMap::new(),
            telemetry: None,
            dag: None,
            delegation_guard: None,
            branch: None,
        }
    }
}

// =============================================================================
// DAG State
// =============================================================================

// =============================================================================
// StatePhase
// =============================================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#source
impl StatePhase {
    /// Check if this is a terminal/complete state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            StatePhase::ChangeArchived | StatePhase::ChangeRejected
        )
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#source
impl Default for StatePhase {
    fn default() -> Self {
        StatePhase::ChangeInited
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#source
impl Serialize for StatePhase {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            StatePhase::ChangeInited => "change_inited",
            StatePhase::ChangeSpecCreated => "change_spec_created",
            StatePhase::ChangeSpecReviewed => "change_spec_reviewed",
            StatePhase::ChangeSpecRevised => "change_spec_revised",
            StatePhase::ChangeImplementationCreated => "change_implementation_created",
            StatePhase::ChangeImplementationReviewed => "change_implementation_reviewed",
            StatePhase::ChangeImplementationRevised => "change_implementation_revised",
            StatePhase::TestCheck => "test_check",
            StatePhase::DocsCheck => "docs_check",
            StatePhase::DocsCreated => "docs_created",
            StatePhase::DocsReviewed => "docs_reviewed",
            StatePhase::DocsRevised => "docs_revised",
            StatePhase::ChangeMergeCreated => "change_merge_created",
            StatePhase::ChangeMergeReviewed => "change_merge_reviewed",
            StatePhase::ChangeMergeRevised => "change_merge_revised",
            StatePhase::ChangeArchived => "change_archived",
            StatePhase::ChangeRejected => "change_rejected",
        };
        serializer.serialize_str(s)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#source
impl<'de> Deserialize<'de> for StatePhase {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            // Primary strings
            "change_inited" => Ok(StatePhase::ChangeInited),
            // Backward compat: all removed clarification/reference phases → ChangeInited
            "post_clarifications_created"
            | "input_restructured"
            | "pre_clarifications_created"
            | "reference_context_created"
            | "reference_context_reviewed"
            | "reference_context_revised" => Ok(StatePhase::ChangeInited),
            "change_spec_created" => Ok(StatePhase::ChangeSpecCreated),
            "change_spec_reviewed" => Ok(StatePhase::ChangeSpecReviewed),
            "change_spec_revised" => Ok(StatePhase::ChangeSpecRevised),
            "change_implementation_created" => Ok(StatePhase::ChangeImplementationCreated),
            "change_implementation_reviewed" => Ok(StatePhase::ChangeImplementationReviewed),
            "change_implementation_revised" => Ok(StatePhase::ChangeImplementationRevised),
            "test_check" => Ok(StatePhase::TestCheck),
            "docs_check" => Ok(StatePhase::DocsCheck),
            "docs_created" => Ok(StatePhase::DocsCreated),
            "docs_reviewed" => Ok(StatePhase::DocsReviewed),
            "docs_revised" => Ok(StatePhase::DocsRevised),
            "change_merge_created" => Ok(StatePhase::ChangeMergeCreated),
            "change_merge_reviewed" => Ok(StatePhase::ChangeMergeReviewed),
            "change_merge_revised" => Ok(StatePhase::ChangeMergeRevised),
            "change_archived" => Ok(StatePhase::ChangeArchived),
            "change_rejected" => Ok(StatePhase::ChangeRejected),
            // Backward compat: removed v3 variants
            "pre_clarifications_reviewed"
            | "pre_clarifications_revised"
            | "pre_clarifications_approved"
            | "reference_context_approved" => Ok(StatePhase::ChangeInited),
            "post_clarifications_reviewed"
            | "post_clarifications_revised"
            | "post_clarifications_approved" => Ok(StatePhase::ChangeInited),
            "change_spec_approved" => Ok(StatePhase::ChangeImplementationCreated),
            "change_implementation_approved" => Ok(StatePhase::ChangeMergeCreated),
            "change_merge_approved" => Ok(StatePhase::ChangeArchived),
            "archived" => Ok(StatePhase::ChangeArchived),
            "rejected" => Ok(StatePhase::ChangeRejected),
            // Legacy impl/merge phase names
            "implementing" | "testing" | "implemented" | "complete" => {
                Ok(StatePhase::ChangeImplementationCreated)
            }
            "impl_reviewed" | "code_reviewing" => Ok(StatePhase::ChangeImplementationReviewed),
            "impl_revised" => Ok(StatePhase::ChangeImplementationRevised),
            "impl_approved" => Ok(StatePhase::ChangeMergeCreated),
            "merging" | "merged" => Ok(StatePhase::ChangeMergeCreated),
            "merge_reviewed" => Ok(StatePhase::ChangeMergeReviewed),
            "merge_revised" => Ok(StatePhase::ChangeMergeRevised),
            "merge_approved" => Ok(StatePhase::ChangeArchived),
            // v2 legacy aliases (old STATE.yaml files)
            "clarified" | "clarifying" => Ok(StatePhase::ChangeInited),
            "clarifications_reviewed" | "clarifications_revised" | "clarifications_approved" => {
                Ok(StatePhase::ChangeInited)
            }
            "clarifications_rejected"
            | "reference_context_rejected"
            | "post_clarifications_rejected"
            | "spec_rejected" => Ok(StatePhase::ChangeRejected),
            "decided" => Ok(StatePhase::ChangeInited),
            "spec_created" => Ok(StatePhase::ChangeSpecCreated),
            "spec_reviewed" => Ok(StatePhase::ChangeSpecReviewed),
            "spec_revised" => Ok(StatePhase::ChangeSpecRevised),
            "spec_approved" | "all_specs_approved" | "specs_generated" | "tasks_generated"
            | "planned" | "challenged" => Ok(StatePhase::ChangeImplementationCreated),
            // Older legacy phases — all reference-context variants → PostClarificationsCreated
            "exploring"
            | "explored"
            | "needs_second_clarification"
            | "spec_context_created"
            | "spec_context_reviewed"
            | "spec_context_revised"
            | "spec_context_approved"
            | "spec_context_rejected"
            | "knowledge_context_created"
            | "knowledge_context_reviewed"
            | "knowledge_context_revised"
            | "knowledge_context_approved"
            | "knowledge_context_rejected"
            | "codebase_context_created"
            | "codebase_context_reviewed"
            | "codebase_context_revised"
            | "codebase_context_approved"
            | "codebase_context_rejected"
            | "gap_codebase_spec_created"
            | "gap_codebase_spec_reviewed"
            | "gap_codebase_spec_revised"
            | "gap_codebase_spec_approved"
            | "gap_codebase_spec_rejected"
            | "gap_codebase_knowledge_created"
            | "gap_codebase_knowledge_reviewed"
            | "gap_codebase_knowledge_revised"
            | "gap_codebase_knowledge_approved"
            | "gap_codebase_knowledge_rejected"
            | "gap_spec_knowledge_created"
            | "gap_spec_knowledge_reviewed"
            | "gap_spec_knowledge_revised"
            | "gap_spec_knowledge_approved"
            | "gap_spec_knowledge_rejected" => Ok(StatePhase::ChangeInited),
            "drafting" | "proposed" | "proposal_created" | "proposal_reviewed"
            | "proposal_revised" | "proposal_approved" | "proposal_rejected" => {
                Ok(StatePhase::ChangeInited)
            }
            _ => Err(serde::de::Error::custom(format!(
                "unknown StatePhase: {}",
                s
            ))),
        }
    }
}

// =============================================================================
// Checksum / Validation
// =============================================================================

// =============================================================================
// Telemetry
// =============================================================================

// CODEGEN-END
