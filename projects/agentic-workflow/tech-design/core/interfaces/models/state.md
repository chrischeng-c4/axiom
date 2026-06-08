---
id: sdd-models-state
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# State Model Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/state.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ChecksumEntry` | projects/agentic-workflow/src/models/state.rs | struct | pub | 172 |  |
| `DagIssue` | projects/agentic-workflow/src/models/state.rs | struct | pub | 149 |  |
| `DagState` | projects/agentic-workflow/src/models/state.rs | struct | pub | 134 |  |
| `DelegationGuard` | projects/agentic-workflow/src/models/state.rs | struct | pub | 59 |  |
| `LlmCall` | projects/agentic-workflow/src/models/state.rs | struct | pub | 253 |  |
| `State` | projects/agentic-workflow/src/models/state.rs | struct | pub | 73 |  |
| `StatePhase` | projects/agentic-workflow/src/models/state.rs | enum | pub | 24 |  |
| `Telemetry` | projects/agentic-workflow/src/models/state.rs | struct | pub | 235 |  |
| `ValidationEntry` | projects/agentic-workflow/src/models/state.rs | struct | pub | 183 |  |
| `ValidationMode` | projects/agentic-workflow/src/models/state.rs | enum | pub | 48 |  |
| `ValidationResult` | projects/agentic-workflow/src/models/state.rs | struct | pub | 212 |  |
| `is_terminal` | projects/agentic-workflow/src/models/state.rs | function | pub | 331 | is_terminal(&self) -> bool |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  StatePhase:
    type: string
    enum: [ChangeInited, ChangeSpecCreated, ChangeSpecReviewed, ChangeSpecRevised, ChangeImplementationCreated, ChangeImplementationReviewed, ChangeImplementationRevised, TestCheck, DocsCheck, DocsCreated, DocsReviewed, DocsRevised, ChangeMergeCreated, ChangeMergeReviewed, ChangeMergeRevised, ChangeArchived, ChangeRejected]
    description: Workflow phase values for the SDD state machine. NOTE custom Serialize/Deserialize impls live outside CODEGEN — do not derive serde here.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Eq]

  ValidationMode:
    type: string
    enum: [Normal, Strict]
    description: Validation mode.
    x-rust-enum:
      derive: [Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq]
      serde_rename_all: lowercase
      variants:
        - { name: Normal, is_default: true, doc: "Normal validation mode (default)." }
        - { name: Strict, doc: "Strict mode: warnings treated as errors." }

  DelegationGuard:
    type: object
    required: [allowed_phases, phase_before, action, started_at]
    description: Phase guard active during delegated agent execution.
    properties:
      allowed_phases:
        type: array
        items: { type: string }
        x-rust-type: "Vec<StatePhase>"
        description: "Phases the delegated agent is allowed to set."
      phase_before:
        type: object
        x-rust-type: "StatePhase"
        description: "Phase snapshot before delegation started."
      action:
        type: string
        description: "Action being delegated."
      started_at:
        type: string
        x-rust-type: "DateTime<Utc>"
        description: "When delegation started."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  State:
    type: object
    required: [change_id, schema_version, created_at, updated_at, phase, iteration, last_action, session_id, checksums, validations, git_workflow, revision_counts, current_task_id, task_revisions, impl_spec_phase, telemetry, dag, delegation_guard, branch]
    description: State file for tracking change progress.
    properties:
      change_id:
        type: string
        description: "Change identifier."
      schema_version:
        type: string
        x-serde-default: "default_schema_version"
        description: "Schema version for forward compatibility."
      created_at:
        type: string
        x-rust-type: "Option<DateTime<Utc>>"
        x-serde-default: true
        description: "Creation timestamp."
      updated_at:
        type: string
        x-rust-type: "Option<DateTime<Utc>>"
        x-serde-default: true
        description: "Last update timestamp."
      phase:
        type: string
        x-rust-type: "StatePhase"
        description: "Current workflow phase."
      iteration:
        type: integer
        x-rust-type: "u32"
        x-serde-default: "default_iteration"
        description: "Current iteration."
      last_action:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Last action performed."
      session_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Session identifier."
      checksums:
        type: object
        x-rust-type: "HashMap<String, ChecksumEntry>"
        x-serde-default: true
        description: "Checksums for artifact staleness detection."
      validations:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ValidationEntry>"
        x-serde-default: true
        description: "Validation history."
      git_workflow:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Git workflow."
      revision_counts:
        type: object
        x-rust-type: "HashMap<String, u32>"
        x-serde-default: true
        description: "Revision counts per phase."
      current_task_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Current task ID."
      task_revisions:
        type: object
        x-rust-type: "HashMap<String, u32>"
        x-serde-default: true
        description: "Per-task revision counts."
      impl_spec_phase:
        type: object
        x-rust-type: "HashMap<String, String>"
        x-serde-default: true
        description: "Per-spec implementation phase tracking."
      telemetry:
        type: object
        x-rust-type: "Option<Telemetry>"
        x-serde-default: true
        description: "LLM call telemetry."
      dag:
        type: object
        x-rust-type: "Option<DagState>"
        x-serde-default: true
        description: "DAG state for multi-issue workflows."
      delegation_guard:
        type: object
        x-rust-type: "Option<DelegationGuard>"
        x-serde-default: true
        description: "Delegation guard for agent phase protection."
      branch:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Git branch name for this change."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  DagState:
    type: object
    required: [issues, current_index, complete]
    description: DAG state for multi-issue topological ordering.
    properties:
      issues:
        type: array
        items: { type: object }
        x-rust-type: "Vec<DagIssue>"
        x-serde-default: true
        description: "Ordered list of issues."
      current_index:
        type: integer
        x-rust-type: "usize"
        x-serde-default: true
        description: "Index of current issue being processed."
      complete:
        type: boolean
        x-serde-default: true
        description: "Whether the DAG is complete."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  DagIssue:
    type: object
    required: [number, title, depends, dependents, processed, blocked_by]
    description: A single issue in the DAG.
    properties:
      number:
        type: integer
        x-rust-type: "u64"
        description: "Issue number."
      title:
        type: string
        x-serde-default: true
        description: "Issue title."
      depends:
        type: array
        items: { type: integer }
        x-rust-type: "Vec<u64>"
        x-serde-default: true
        description: "Issues this one depends on."
      dependents:
        type: array
        items: { type: integer }
        x-rust-type: "Vec<u64>"
        x-serde-default: true
        description: "Issues that depend on this one."
      processed:
        type: boolean
        x-serde-default: true
        description: "Whether this issue has been processed."
      blocked_by:
        type: array
        items: { type: integer }
        x-rust-type: "Vec<u64>"
        x-serde-default: true
        description: "Issues this one is blocked by."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ChecksumEntry:
    type: object
    required: [hash, validated_at]
    description: Checksum entry with validation timestamp.
    properties:
      hash:
        type: string
        description: "Hash value."
      validated_at:
        type: string
        x-rust-type: "Option<DateTime<Utc>>"
        x-serde-default: true
        description: "Timestamp when validated."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ValidationEntry:
    type: object
    required: [step, timestamp, rules_version, rules_hash, mode, result, errors, warnings]
    description: Validation history entry.
    properties:
      step:
        type: string
        description: "Validation step name."
      timestamp:
        type: string
        x-rust-type: "Option<DateTime<Utc>>"
        x-serde-default: true
        description: "Timestamp."
      rules_version:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Rules version."
      rules_hash:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Rules hash."
      mode:
        type: object
        x-rust-type: "Option<ValidationMode>"
        x-serde-default: true
        description: "Validation mode."
      result:
        type: object
        x-rust-type: "Option<ValidationResult>"
        x-serde-default: true
        description: "Validation result."
      errors:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Errors."
      warnings:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Warnings."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ValidationResult:
    type: object
    required: [valid, high, medium, low, verdict, issues_parsed]
    description: Validation result.
    properties:
      valid:
        type: boolean
        description: "Whether validation passed."
      high:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "High severity count."
      medium:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "Medium severity count."
      low:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "Low severity count."
      verdict:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Optional verdict."
      issues_parsed:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
        description: "Number of issues parsed."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Telemetry:
    type: object
    required: [calls, total_cost_usd, total_tokens_in, total_tokens_out]
    description: LLM telemetry.
    properties:
      calls:
        type: array
        items: { type: object }
        x-rust-type: "Vec<LlmCall>"
        x-serde-default: true
        description: "LLM calls."
      total_cost_usd:
        type: number
        x-rust-type: "f64"
        x-serde-default: true
        description: "Total cost in USD."
      total_tokens_in:
        type: integer
        x-rust-type: "u64"
        x-serde-default: true
        description: "Total input tokens."
      total_tokens_out:
        type: integer
        x-rust-type: "u64"
        x-serde-default: true
        description: "Total output tokens."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  LlmCall:
    type: object
    required: [step, sdd_version, model, tokens_in, tokens_out, cost_usd, duration_ms, timestamp]
    description: Single LLM call telemetry.
    properties:
      step:
        type: string
        description: "Step identifier."
      sdd_version:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "SDD version."
      model:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Model identifier."
      tokens_in:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-default: true
        description: "Input tokens."
      tokens_out:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-default: true
        description: "Output tokens."
      cost_usd:
        type: number
        x-rust-type: "Option<f64>"
        x-serde-default: true
        description: "Cost in USD."
      duration_ms:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-default: true
        description: "Duration in milliseconds."
      timestamp:
        type: string
        x-rust-type: "Option<DateTime<Utc>>"
        x-serde-default: true
        description: "Timestamp."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/state.rs -->
```rust
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/state.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete state model module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 11 types: 2 enums + 9 structs.
- [schema] StatePhase deliberately omits Serialize/Deserialize from derive list because custom impls handle it (with 60+ legacy alias deserialization paths).
- [changes] All eleven in `replaces`; all hand-written impls preserved.
