// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/frontmatter.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
