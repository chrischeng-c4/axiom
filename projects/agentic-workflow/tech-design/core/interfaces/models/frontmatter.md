---
id: sdd-models-frontmatter
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Frontmatter Model Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/frontmatter.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DesignElements` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 275 |  |
| `DiagramElementInfo` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 264 |  |
| `HistoryEntry` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 107 |  |
| `IssueBlock` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 442 |  |
| `IssueLocation` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 428 |  |
| `IssueSeverity` | projects/agentic-workflow/src/models/frontmatter.rs | enum | pub | 95 |  |
| `LayerBreakdown` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 178 |  |
| `LayerInfo` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 167 |  |
| `MainSpecFrontmatter` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 125 |  |
| `MergeStrategy` | projects/agentic-workflow/src/models/frontmatter.rs | enum | pub | 22 |  |
| `PriorityBreakdown` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 249 |  |
| `RequirementBlock` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 409 |  |
| `RequirementPriority` | projects/agentic-workflow/src/models/frontmatter.rs | enum | pub | 68 |  |
| `RequirementStatus` | projects/agentic-workflow/src/models/frontmatter.rs | enum | pub | 81 |  |
| `RequirementsSummary` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 235 |  |
| `SpecFileChange` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 302 |  |
| `SpecFrontmatter` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 319 |  |
| `SpecReference` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 225 |  |
| `TaskAction` | projects/agentic-workflow/src/models/frontmatter.rs | enum | pub | 37 |  |
| `TaskBlock` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 385 |  |
| `TaskStatus` | projects/agentic-workflow/src/models/frontmatter.rs | enum | pub | 52 |  |
| `TasksFrontmatter` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 196 |  |
| `TasksSummary` | projects/agentic-workflow/src/models/frontmatter.rs | struct | pub | 147 |  |
| `new` | projects/agentic-workflow/src/models/frontmatter.rs | function | pub | 462 | new(agent: &str, tool: &str, action: &str, duration_secs: Option<f64>) -> Self |
| `to_yaml` | projects/agentic-workflow/src/models/frontmatter.rs | function | pub | 472 | to_yaml(&self) -> String |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  MergeStrategy:
    type: string
    enum: [new, extend, replace, patch]
    description: Merge strategy for change spec application.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq]
      serde_rename_all: lowercase
      variants:
        - { name: New,     doc: "Create a new file." }
        - { name: Extend,  doc: "Extend an existing file." }
        - { name: Replace, doc: "Replace an existing file wholesale." }
        - { name: Patch,   doc: "Patch an existing file." }

  TaskAction:
    type: string
    enum: [CREATE, MODIFY, DELETE, RENAME]
    description: Action to perform on a file in a task block.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq]
      serde_rename_all: SCREAMING_SNAKE_CASE
      variants:
        - { name: Create, doc: "Create a new file." }
        - { name: Modify, doc: "Modify an existing file." }
        - { name: Delete, doc: "Delete a file." }
        - { name: Rename, doc: "Rename a file." }

  TaskStatus:
    type: string
    enum: [pending, in_progress, completed, blocked]
    description: Status of a task block.
    x-rust-enum:
      derive: [Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq]
      serde_rename_all: snake_case
      variants:
        - { name: Pending,    is_default: true, doc: "Task is pending (default)." }
        - { name: InProgress, doc: "Task is in progress." }
        - { name: Completed,  doc: "Task is completed." }
        - { name: Blocked,    doc: "Task is blocked." }

  RequirementPriority:
    type: string
    enum: [high, medium, low]
    description: Priority of a requirement block.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq]
      serde_rename_all: lowercase
      variants:
        - { name: High,   doc: "High priority." }
        - { name: Medium, doc: "Medium priority." }
        - { name: Low,    doc: "Low priority." }

  RequirementStatus:
    type: string
    enum: [draft, reviewed, approved]
    description: Status of a requirement block.
    x-rust-enum:
      derive: [Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq]
      serde_rename_all: lowercase
      variants:
        - { name: Draft,    is_default: true, doc: "Requirement is in draft (default)." }
        - { name: Reviewed, doc: "Requirement has been reviewed." }
        - { name: Approved, doc: "Requirement is approved." }

  IssueSeverity:
    type: string
    enum: [high, medium, low]
    description: Severity of an issue block.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq]
      serde_rename_all: lowercase
      variants:
        - { name: High,   doc: "High severity." }
        - { name: Medium, doc: "Medium severity." }
        - { name: Low,    doc: "Low severity." }

  HistoryEntry:
    type: object
    required: [timestamp, agent, tool, action]
    description: History entry for tracking document modifications by agents.
    properties:
      timestamp:
        type: string
        x-rust-type: "DateTime<Utc>"
        description: "Timestamp of the modification."
      agent:
        type: string
        description: "Agent identifier."
      tool:
        type: string
        description: "Tool used."
      action:
        type: string
        description: "Action performed."
      duration_secs:
        type: number
        x-rust-type: "Option<f64>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Duration in seconds, if measured."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  MainSpecFrontmatter:
    type: object
    required: [id, doc_type, title, version, files]
    description: |
      Frontmatter for main specs in .aw/tech-design/{group}/{id}.md.
      Describes WHAT to build — no implementation progress tracking.
    properties:
      id:
        type: string
        description: "Spec identifier (e.g. 'string-ops')."
      doc_type:
        type: string
        x-serde-rename: "type"
        description: "Document type (always 'spec')."
      title:
        type: string
        description: "Spec title."
      version:
        type: integer
        x-rust-type: "u32"
        x-serde-default: "default_version"
        description: "Document version."
      spec_type:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Spec type classification (algorithm, data-model, utility, integration, etc.)."
      files:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Codebase file paths relative to crate src/."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  TasksSummary:
    type: object
    required: [total, completed, in_progress, blocked, pending]
    description: Summary of task counts by status.
    properties:
      total:
        type: integer
        x-rust-type: "u32"
        description: "Total task count."
      completed:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "Completed task count."
      in_progress:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "In-progress task count."
      blocked:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "Blocked task count."
      pending:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "Pending task count."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  LayerInfo:
    type: object
    required: [task_count]
    description: Task statistics for a single layer.
    properties:
      task_count:
        type: integer
        x-rust-type: "u32"
        description: "Number of tasks in this layer."
      estimated_files:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
        description: "Estimated number of files affected."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  LayerBreakdown:
    type: object
    description: Per-layer task breakdown for a tasks.md document.
    properties:
      data:
        type: object
        x-rust-type: "Option<LayerInfo>"
        x-serde-default: true
        description: "Data layer stats."
      logic:
        type: object
        x-rust-type: "Option<LayerInfo>"
        x-serde-default: true
        description: "Logic layer stats."
      integration:
        type: object
        x-rust-type: "Option<LayerInfo>"
        x-serde-default: true
        description: "Integration layer stats."
      testing:
        type: object
        x-rust-type: "Option<LayerInfo>"
        x-serde-default: true
        description: "Testing layer stats."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  TasksFrontmatter:
    type: object
    required: [id, doc_type, version]
    description: Frontmatter for tasks.md documents.
    properties:
      id:
        type: string
        x-serde-alias: ["change_id"]
        description: "Change identifier. Accepts legacy 'change_id' alias."
      doc_type:
        type: string
        x-serde-rename: "type"
        description: "Document type (always 'tasks')."
      version:
        type: integer
        x-rust-type: "u32"
        description: "Document version."
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
      proposal_ref:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Optional reference to the proposal."
      summary:
        type: object
        x-rust-type: "Option<TasksSummary>"
        x-serde-default: true
        description: "Aggregated task summary."
      layers:
        type: object
        x-rust-type: "Option<LayerBreakdown>"
        x-serde-default: true
        description: "Per-layer task breakdown."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SpecReference:
    type: object
    required: [id, path]
    description: Reference to a spec file.
    properties:
      id:
        type: string
        description: "Spec identifier."
      path:
        type: string
        description: "Spec file path."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RequirementsSummary:
    type: object
    required: [total, ids]
    description: Summary of requirements in a spec.
    properties:
      total:
        type: integer
        x-rust-type: "u32"
        description: "Total requirement count."
      ids:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Requirement identifiers."
      by_priority:
        type: object
        x-rust-type: "Option<PriorityBreakdown>"
        x-serde-default: true
        description: "Breakdown by priority."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  PriorityBreakdown:
    type: object
    required: [high, medium, low]
    description: Requirement count per priority level.
    properties:
      high:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "High-priority count."
      medium:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "Medium-priority count."
      low:
        type: integer
        x-rust-type: "u32"
        x-serde-default: true
        description: "Low-priority count."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  DiagramElementInfo:
    type: object
    required: [diagram_type, title]
    description: Information about a diagram element in a spec.
    properties:
      diagram_type:
        type: string
        x-serde-rename: "type"
        description: "Diagram type identifier."
      title:
        type: string
        description: "Diagram title."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  DesignElements:
    type: object
    required: [has_mermaid, has_json_schema, has_pseudo_code, has_api_spec, has_semantic_diagrams, diagrams]
    description: Flags and metadata describing design artifacts present in a spec.
    properties:
      has_mermaid:
        type: boolean
        x-serde-default: true
        description: "Whether the spec contains Mermaid diagrams."
      has_json_schema:
        type: boolean
        x-serde-default: true
        description: "Whether the spec contains JSON Schema sections."
      has_pseudo_code:
        type: boolean
        x-serde-default: true
        description: "Whether the spec contains pseudo code."
      has_api_spec:
        type: boolean
        x-serde-default: true
        description: "Whether the spec contains an API spec section."
      has_semantic_diagrams:
        type: boolean
        x-serde-default: true
        description: "Whether the spec contains semantic diagrams."
      api_spec_type:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Type of API spec if present."
      diagrams:
        type: array
        items: { type: object }
        x-rust-type: "Vec<DiagramElementInfo>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "List of diagram elements."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  SpecFileChange:
    type: object
    required: [file, action]
    description: A file change record in a spec's change list.
    properties:
      file:
        type: string
        description: "Path of the file to change."
      action:
        type: string
        description: "Action to apply (create, modify, delete)."
      context_ref:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Optional reference to contextual spec."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Optional description of the change."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SpecFrontmatter:
    type: object
    required: [id, doc_type, title, version, tags, child_specs, related_specs, depends, changes, codebase_paths, knowledge_refs, history]
    description: |
      Frontmatter for change specs in .aw/changes/{id}/specs/*.md.
      Tracks implementation progress (status, history, merge_strategy).
    properties:
      id:
        type: string
        description: "Spec identifier."
      doc_type:
        type: string
        x-serde-rename: "type"
        description: "Document type (always 'spec')."
      title:
        type: string
        description: "Spec title."
      version:
        type: integer
        x-rust-type: "u32"
        description: "Document version."
      spec_type:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Spec type classification."
      tags:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Tags for the spec."
      spec_group:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Spec group identifier."
      main_spec_ref:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Reference to the main spec."
      merge_strategy:
        type: object
        x-rust-type: "Option<MergeStrategy>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Merge strategy for this spec."
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
      parent_spec:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Parent spec identifier."
      child_specs:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Child spec identifiers."
      related_specs:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SpecReference>"
        x-serde-default: true
        description: "Related spec references."
      depends:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Spec identifiers this spec depends on."
      requirements:
        type: object
        x-rust-type: "Option<RequirementsSummary>"
        x-serde-default: true
        description: "Requirements summary."
      design_elements:
        type: object
        x-rust-type: "Option<DesignElements>"
        x-serde-default: true
        description: "Design elements present in the spec."
      changes:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SpecFileChange>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "File changes listed in the spec."
      codebase_paths:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Relevant codebase paths."
      knowledge_refs:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Knowledge reference paths."
      history:
        type: array
        items: { type: object }
        x-rust-type: "Vec<HistoryEntry>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "History of document modifications."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  TaskBlock:
    type: object
    required: [id, action, status, file, depends_on]
    description: A single task entry in an inline YAML task block.
    properties:
      id:
        type: string
        description: "Task identifier."
      action:
        type: object
        x-rust-type: "TaskAction"
        description: "Action to perform on the file."
      status:
        type: object
        x-rust-type: "TaskStatus"
        x-serde-default: true
        description: "Current task status."
      file:
        type: string
        description: "Target file path."
      spec_ref:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Optional spec reference for this task."
      depends_on:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Task identifiers this task depends on."
      estimated_lines:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
        description: "Estimated line count for the change."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RequirementBlock:
    type: object
    required: [id, priority, status]
    description: A single requirement entry in an inline YAML requirement block.
    properties:
      id:
        type: string
        description: "Requirement identifier (e.g. R1)."
      priority:
        type: object
        x-rust-type: "RequirementPriority"
        description: "Priority level."
      status:
        type: object
        x-rust-type: "RequirementStatus"
        x-serde-default: true
        description: "Current status."
      scenarios:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
        description: "Number of associated scenarios."
      acceptance_criteria:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
        description: "Number of acceptance criteria."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  IssueLocation:
    type: object
    required: [file]
    description: Location of an issue within a reviewed artifact.
    properties:
      file:
        type: string
        description: "File path."
      line:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
        description: "Line number."
      section:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Section identifier."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  IssueBlock:
    type: object
    required: [id, severity, category, affects_requirements]
    description: A single issue entry in an inline YAML issue block.
    properties:
      id:
        type: integer
        x-rust-type: "u32"
        description: "Issue identifier."
      severity:
        type: object
        x-rust-type: "IssueSeverity"
        description: "Issue severity."
      category:
        type: string
        description: "Issue category."
      location:
        type: object
        x-rust-type: "Option<IssueLocation>"
        x-serde-default: true
        description: "Location of the issue."
      affects_requirements:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Requirement identifiers affected by this issue."
      auto_fixable:
        type: boolean
        x-rust-type: "Option<bool>"
        x-serde-default: true
        description: "Whether the issue can be automatically fixed."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/frontmatter.rs -->
```rust
//! Frontmatter Types for SDD Documents
//!
//! Defines the YAML frontmatter structures for:
//! - tasks.md (ChangeSpec)
//! - specs/*.md (ChangeSpec)
//! - .aw/tech-design/ (MainSpec)
//! - Inline block types

// =============================================================================
// History Entry (shared across all document types)
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Merge strategy for change spec application.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    /// Create a new file.
    New,
    /// Extend an existing file.
    Extend,
    /// Replace an existing file wholesale.
    Replace,
    /// Patch an existing file.
    Patch,
}

/// Action to perform on a file in a task block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskAction {
    /// Create a new file.
    Create,
    /// Modify an existing file.
    Modify,
    /// Delete a file.
    Delete,
    /// Rename a file.
    Rename,
}

/// Status of a task block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is pending (default).
    #[default]
    Pending,
    /// Task is in progress.
    InProgress,
    /// Task is completed.
    Completed,
    /// Task is blocked.
    Blocked,
}

/// Priority of a requirement block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequirementPriority {
    /// High priority.
    High,
    /// Medium priority.
    Medium,
    /// Low priority.
    Low,
}

/// Status of a requirement block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequirementStatus {
    /// Requirement is in draft (default).
    #[default]
    Draft,
    /// Requirement has been reviewed.
    Reviewed,
    /// Requirement is approved.
    Approved,
}

/// Severity of an issue block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// High severity.
    High,
    /// Medium severity.
    Medium,
    /// Low severity.
    Low,
}

/// History entry for tracking document modifications by agents.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Timestamp of the modification.
    pub timestamp: DateTime<Utc>,
    /// Agent identifier.
    pub agent: String,
    /// Tool used.
    pub tool: String,
    /// Action performed.
    pub action: String,
    /// Duration in seconds, if measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f64>,
}

/// Frontmatter for main specs in .aw/tech-design/{group}/{id}.md.
/// Describes WHAT to build — no implementation progress tracking.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainSpecFrontmatter {
    /// Spec identifier (e.g. 'string-ops').
    pub id: String,
    /// Document type (always 'spec').
    #[serde(rename = "type")]
    pub doc_type: String,
    /// Spec title.
    pub title: String,
    /// Document version.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Spec type classification (algorithm, data-model, utility, integration, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_type: Option<String>,
    /// Codebase file paths relative to crate src/.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

/// Summary of task counts by status.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TasksSummary {
    /// Total task count.
    pub total: u32,
    /// Completed task count.
    #[serde(default)]
    pub completed: u32,
    /// In-progress task count.
    #[serde(default)]
    pub in_progress: u32,
    /// Blocked task count.
    #[serde(default)]
    pub blocked: u32,
    /// Pending task count.
    #[serde(default)]
    pub pending: u32,
}

/// Task statistics for a single layer.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    /// Number of tasks in this layer.
    pub task_count: u32,
    /// Estimated number of files affected.
    #[serde(default)]
    pub estimated_files: Option<u32>,
}

/// Per-layer task breakdown for a tasks.md document.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayerBreakdown {
    /// Data layer stats.
    #[serde(default)]
    pub data: Option<LayerInfo>,
    /// Logic layer stats.
    #[serde(default)]
    pub logic: Option<LayerInfo>,
    /// Integration layer stats.
    #[serde(default)]
    pub integration: Option<LayerInfo>,
    /// Testing layer stats.
    #[serde(default)]
    pub testing: Option<LayerInfo>,
}

/// Frontmatter for tasks.md documents.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksFrontmatter {
    /// Change identifier. Accepts legacy 'change_id' alias.
    #[serde(alias = "change_id")]
    pub id: String,
    /// Document type (always 'tasks').
    #[serde(rename = "type")]
    pub doc_type: String,
    /// Document version.
    pub version: u32,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// Optional reference to the proposal.
    #[serde(default)]
    pub proposal_ref: Option<String>,
    /// Aggregated task summary.
    #[serde(default)]
    pub summary: Option<TasksSummary>,
    /// Per-layer task breakdown.
    #[serde(default)]
    pub layers: Option<LayerBreakdown>,
}

/// Reference to a spec file.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecReference {
    /// Spec identifier.
    pub id: String,
    /// Spec file path.
    pub path: String,
}

/// Summary of requirements in a spec.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequirementsSummary {
    /// Total requirement count.
    pub total: u32,
    /// Requirement identifiers.
    #[serde(default)]
    pub ids: Vec<String>,
    /// Breakdown by priority.
    #[serde(default)]
    pub by_priority: Option<PriorityBreakdown>,
}

/// Requirement count per priority level.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PriorityBreakdown {
    /// High-priority count.
    #[serde(default)]
    pub high: u32,
    /// Medium-priority count.
    #[serde(default)]
    pub medium: u32,
    /// Low-priority count.
    #[serde(default)]
    pub low: u32,
}

/// Information about a diagram element in a spec.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramElementInfo {
    /// Diagram type identifier.
    #[serde(rename = "type")]
    pub diagram_type: String,
    /// Diagram title.
    pub title: String,
}

/// Flags and metadata describing design artifacts present in a spec.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignElements {
    /// Whether the spec contains Mermaid diagrams.
    #[serde(default)]
    pub has_mermaid: bool,
    /// Whether the spec contains JSON Schema sections.
    #[serde(default)]
    pub has_json_schema: bool,
    /// Whether the spec contains pseudo code.
    #[serde(default)]
    pub has_pseudo_code: bool,
    /// Whether the spec contains an API spec section.
    #[serde(default)]
    pub has_api_spec: bool,
    /// Whether the spec contains semantic diagrams.
    #[serde(default)]
    pub has_semantic_diagrams: bool,
    /// Type of API spec if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_spec_type: Option<String>,
    /// List of diagram elements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagrams: Vec<DiagramElementInfo>,
}

/// A file change record in a spec's change list.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFileChange {
    /// Path of the file to change.
    pub file: String,
    /// Action to apply (create, modify, delete).
    pub action: String,
    /// Optional reference to contextual spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_ref: Option<String>,
    /// Optional description of the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Frontmatter for change specs in .aw/changes/{id}/specs/*.md.
/// Tracks implementation progress (status, history, merge_strategy).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFrontmatter {
    /// Spec identifier.
    pub id: String,
    /// Document type (always 'spec').
    #[serde(rename = "type")]
    pub doc_type: String,
    /// Spec title.
    pub title: String,
    /// Document version.
    pub version: u32,
    /// Spec type classification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_type: Option<String>,
    /// Tags for the spec.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Spec group identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_group: Option<String>,
    /// Reference to the main spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_spec_ref: Option<String>,
    /// Merge strategy for this spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_strategy: Option<MergeStrategy>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// Parent spec identifier.
    #[serde(default)]
    pub parent_spec: Option<String>,
    /// Child spec identifiers.
    #[serde(default)]
    pub child_specs: Vec<String>,
    /// Related spec references.
    #[serde(default)]
    pub related_specs: Vec<SpecReference>,
    /// Spec identifiers this spec depends on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends: Vec<String>,
    /// Requirements summary.
    #[serde(default)]
    pub requirements: Option<RequirementsSummary>,
    /// Design elements present in the spec.
    #[serde(default)]
    pub design_elements: Option<DesignElements>,
    /// File changes listed in the spec.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<SpecFileChange>,
    /// Relevant codebase paths.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub codebase_paths: Vec<String>,
    /// Knowledge reference paths.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub knowledge_refs: Vec<String>,
    /// History of document modifications.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<HistoryEntry>,
}

/// A single task entry in an inline YAML task block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBlock {
    /// Task identifier.
    pub id: String,
    /// Action to perform on the file.
    pub action: TaskAction,
    /// Current task status.
    #[serde(default)]
    pub status: TaskStatus,
    /// Target file path.
    pub file: String,
    /// Optional spec reference for this task.
    #[serde(default)]
    pub spec_ref: Option<String>,
    /// Task identifiers this task depends on.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Estimated line count for the change.
    #[serde(default)]
    pub estimated_lines: Option<u32>,
}

/// A single requirement entry in an inline YAML requirement block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementBlock {
    /// Requirement identifier (e.g. R1).
    pub id: String,
    /// Priority level.
    pub priority: RequirementPriority,
    /// Current status.
    #[serde(default)]
    pub status: RequirementStatus,
    /// Number of associated scenarios.
    #[serde(default)]
    pub scenarios: Option<u32>,
    /// Number of acceptance criteria.
    #[serde(default)]
    pub acceptance_criteria: Option<u32>,
}

/// Location of an issue within a reviewed artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLocation {
    /// File path.
    pub file: String,
    /// Line number.
    #[serde(default)]
    pub line: Option<u32>,
    /// Section identifier.
    #[serde(default)]
    pub section: Option<String>,
}

/// A single issue entry in an inline YAML issue block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueBlock {
    /// Issue identifier.
    pub id: u32,
    /// Issue severity.
    pub severity: IssueSeverity,
    /// Issue category.
    pub category: String,
    /// Location of the issue.
    #[serde(default)]
    pub location: Option<IssueLocation>,
    /// Requirement identifiers affected by this issue.
    #[serde(default)]
    pub affects_requirements: Vec<String>,
    /// Whether the issue can be automatically fixed.
    #[serde(default)]
    pub auto_fixable: Option<bool>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#source
impl HistoryEntry {
    pub fn new(agent: &str, tool: &str, action: &str, duration_secs: Option<f64>) -> Self {
        Self {
            timestamp: Utc::now(),
            agent: agent.to_string(),
            tool: tool.to_string(),
            action: action.to_string(),
            duration_secs,
        }
    }

    pub fn to_yaml(&self) -> String {
        let mut yaml = format!(
            "  - timestamp: {}\n    agent: \"{}\"\n    tool: \"{}\"\n    action: \"{}\"",
            self.timestamp.to_rfc3339(),
            self.agent,
            self.tool,
            self.action
        );
        if let Some(secs) = self.duration_secs {
            yaml.push_str(&format!("\n    duration_secs: {:.2}", secs));
        }
        yaml
    }
}

// =============================================================================
// MainSpec Frontmatter (#489)
// =============================================================================

fn default_version() -> u32 {
    1
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#source
impl Default for MainSpecFrontmatter {
    fn default() -> Self {
        Self {
            id: String::new(),
            doc_type: "spec".to_string(),
            title: String::new(),
            version: 1,
            spec_type: None,
            files: Vec::new(),
        }
    }
}

// =============================================================================
// Tasks Frontmatter
// =============================================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#source
impl Default for TasksFrontmatter {
    fn default() -> Self {
        Self {
            id: String::new(),
            doc_type: "tasks".to_string(),
            version: 1,
            created_at: None,
            updated_at: None,
            proposal_ref: None,
            summary: None,
            layers: None,
        }
    }
}

// =============================================================================
// Spec Frontmatter (Change Specs)
// =============================================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#source
impl Default for SpecFrontmatter {
    fn default() -> Self {
        Self {
            id: String::new(),
            doc_type: "spec".to_string(),
            title: String::new(),
            version: 1,
            spec_type: None,
            tags: Vec::new(),
            spec_group: None,
            main_spec_ref: None,
            merge_strategy: None,
            created_at: None,
            updated_at: None,
            parent_spec: None,
            child_specs: Vec::new(),
            related_specs: Vec::new(),
            depends: Vec::new(),
            requirements: None,
            design_elements: None,
            changes: Vec::new(),
            codebase_paths: Vec::new(),
            knowledge_refs: Vec::new(),
            history: Vec::new(),
        }
    }
}

// =============================================================================
// Inline YAML Block Types
// =============================================================================
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/frontmatter.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete frontmatter model module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] `TasksFrontmatter.id` correctly declares `x-serde-alias: ["change_id"]`, matching the source `#[serde(alias = "change_id")]` — primary verification target confirmed.
- [schema] All six enum `serde_rename_all` strategies match source exactly: `MergeStrategy` (lowercase), `TaskAction` (SCREAMING_SNAKE_CASE), `TaskStatus` (snake_case), `RequirementPriority` (lowercase), `RequirementStatus` (lowercase), `IssueSeverity` (lowercase).
- [schema] All `x-serde-rename: "type"` field annotations match source `#[serde(rename = "type")]` on `doc_type` fields in `MainSpecFrontmatter`, `TasksFrontmatter`, `SpecFrontmatter`, and `DiagramElementInfo`.
- [schema] `is_default: true` on `TaskStatus::Pending` and `RequirementStatus::Draft` correctly maps to `#[default]` derive in source.
- [schema] `MainSpecFrontmatter.version` `x-serde-default: "default_version"` matches source `#[serde(default = "default_version")]`. `TasksFrontmatter.version` and `SpecFrontmatter.version` correctly omit `x-serde-default` (bare field in source).
- [schema] Optional fields with `x-serde-skip-if` vs. without are correctly differentiated — fields that only have `#[serde(default)]` in source (no `skip_serializing_if`) omit `x-serde-skip-if` in spec; fields with both have both.
- [changes] All 23 types listed in `replaces`; hand-written regions (`impl Default` ×3, `impl HistoryEntry`, `default_version`) explicitly enumerated and excluded from CODEGEN scope.
