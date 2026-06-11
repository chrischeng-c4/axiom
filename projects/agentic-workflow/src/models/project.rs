//! Data model for `.aw/projects.toml` — auto-generated project/workspace registry.
//!
//! These types are the canonical representation shared between `project_discovery`
//! (writes) and `project_registry` (reads/merges).

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R9

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::models::tech_stack::Language;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/project.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Codegen configuration for a workspace.
/// Used in both per-workspace overrides and `[defaults.workspace]`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/project.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodegenProfile {
    /// Optional language/runtime target override for code generation. Defaults to the workspace target when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<Language>,
    /// Named generator/template profile for this workspace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Optional web/app framework (e.g. `axum-service`, `react-component`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    /// Optional runtime identifier (e.g. `tokio`, `uvicorn`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// Optional bundler (e.g. `vite`, `webpack`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundler: Option<String>,
    /// Default `#[derive(...)]` attributes for generated structs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_derives: Vec<String>,
}

/// Binds one EC category to an external measurement tool (wi-13).
/// The deterministic verify command is built by `EcBinding::command()`
/// (project-health source): arena -> `arena run --spec <spec>`,
/// rig -> `rig run --dir <dir>`, meter -> `meter run --target <meter>`,
/// vat -> `vat run [runner]`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/project.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EcBinding {
    /// Which external tool verifies this category: `arena`, `rig`, `meter`, or `vat`. Validated by the command builder, not serde — an unknown tool is a Failed EC command, not a parse error.
    pub tool: String,
    /// arena: comparison spec path -> `arena run --spec <spec>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,
    /// rig: scenario directory -> `rig run --dir <dir>`; vat: optional runner id -> `vat run <dir>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    /// meter: target path whose meter.toml [gate] ceilings the run honors -> `meter run --target <meter>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meter: Option<String>,
}

/// A discovered or manually declared project entry in `.aw/projects.toml`.
/// Each project maps to a top-level directory under `crates/`, `projects/`, or `packages/`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/project.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    /// Project identifier derived from directory name.
    pub name: String,
    /// Path relative to repo root (e.g. `projects/agentic-workflow`, `projects/conductor`).
    pub path: PathBuf,
    /// Override for `.aw/tech-design` sub-path. Defaults to the discovered path when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tech_design_dir: Option<String>,
    /// EC tool bindings by category (free strings, e.g. `benchmark`, `stability`). A category absent from this map falls back to the EC manifest command. Declared before `workspaces`: contract before implementation.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ec: BTreeMap<String, EcBinding>,
    /// Non-empty list of workspaces contained in this project.
    pub workspaces: Vec<Workspace>,
}

/// Container for the `[defaults]` table in `projects.toml`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/project.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProjectsDefaults {
    /// Default values applied to every workspace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceDefaults>,
}

/// Top-level document structure for `.aw/projects.toml`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/project.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProjectsToml {
    /// Workspace-level fallback defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defaults: Option<ProjectsDefaults>,
    /// Ordered list of discovered/declared project entries.
    #[serde(default)]
    pub projects: Vec<Project>,
}

/// A single language workspace within a project.
/// Single-language projects have one workspace; `be`/`fe` projects have two.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/project.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    /// Short identifier (e.g. `be`, `fe`, `cli`, or same as project name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Glob path patterns relative to repo root (e.g. `["projects/agentic-workflow/**"]`).
    pub paths: Vec<String>,
    /// Language/runtime target inferred from manifest files.
    pub target: Language,
    /// Shell command to run the workspace test suite. Omitted when the required tool/lock file is not present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_cmd: Option<String>,
    /// Optional codegen profile override for this workspace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codegen: Option<CodegenProfile>,
}

/// Fallback values applied when a workspace field is absent in both
/// auto-discovery and `config.toml` overrides.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/project.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WorkspaceDefaults {
    /// Default codegen profile applied to every workspace missing one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codegen: Option<CodegenProfile>,
}
// CODEGEN-END
