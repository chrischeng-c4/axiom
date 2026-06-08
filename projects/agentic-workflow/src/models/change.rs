// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
use super::{Challenge, RequirementDelta, ValidationRules, Verification};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// Phase of a change.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangePhase {
    /// Proposal generated.
    #[serde(rename = "Proposed")]
    Proposed,
    /// Analyzing codebase.
    #[serde(rename = "Exploring")]
    Exploring,
    /// Challenge generated, awaiting review.
    #[serde(rename = "Challenged")]
    Challenged,
    /// Challenge rejected, requires manual intervention.
    #[serde(rename = "Rejected")]
    Rejected,
    /// Implementation in progress.
    #[serde(rename = "Implementing")]
    Implementing,
    /// All tasks complete, ready to archive.
    #[serde(rename = "Complete")]
    Complete,
    /// Archived.
    #[serde(rename = "Archived")]
    Archived,
}

/// Represents a change proposal with all associated files.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Unique identifier (e.g., 'add-oauth').
    pub id: String,
    /// Brief description.
    pub description: String,
    /// Current phase.
    pub phase: ChangePhase,
    /// When this change was created.
    pub created_at: String,
    /// When this change was last modified.
    pub updated_at: String,
    /// Spec deltas (what requirements are being added/modified/removed).
    #[serde(default)]
    pub deltas: Vec<RequirementDelta>,
    /// Challenge report (if challenged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub challenge: Option<Challenge>,
    /// Verification report (if verified).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<Verification>,
}

/// Workflow configuration. Legacy fields kept for backward-compatible TOML deserialization; all are skip_serializing.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowConfig {
    /// Legacy: ignored.
    #[serde(skip_serializing, default)]
    pub format_iterations: u32,
    /// Legacy: ignored.
    #[serde(skip_serializing, default)]
    pub planning_iterations: u32,
    /// Legacy: ignored.
    #[serde(skip_serializing, default)]
    pub implementation_iterations: u32,
    /// Legacy: ignored.
    #[serde(skip_serializing, default)]
    pub archive_iterations: u32,
    /// Legacy: ignored.
    #[serde(skip_serializing, default)]
    pub script_retries: u32,
    /// Legacy: ignored.
    #[serde(skip_serializing, default)]
    pub retry_delay_secs: u64,
    /// Legacy: ignored.
    #[serde(skip_serializing, default)]
    pub sequential_implementation: bool,
}

/// Configuration for a single workflow stage.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageConfig {
    /// Ordered list of agents to try for this stage.
    pub agents: Vec<String>,
}

/// Interface mode for SDD workflow. Mainthread always uses CLI commands.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SddInterface {
    /// CLI mode: workflow uses score commands.
    #[default]
    Cli,
}

/// Workflow artifact types for agent selection. NOTE: no Serialize/Deserialize — name() impls are hand-written outside CODEGEN.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowArtifact {
    /// Restructure input (grouping + requirements + questions).
    RestructureInput,
    /// Pre-clarifications (change init).
    CreatePreClarifications,
    /// Post-clarifications.
    CreatePostClarifications,
    /// Reference context (unified exploration).
    CreateReferenceContext,
    /// Review reference context.
    ReviewReferenceContext,
    /// Revise reference context.
    ReviseReferenceContext,
    /// Create change spec.
    CreateChangeSpec,
    /// Review change spec.
    ReviewChangeSpec,
    /// Revise change spec.
    ReviseChangeSpec,
    /// Create change implementation.
    CreateChangeImplementation,
    /// Review change implementation.
    ReviewChangeImplementation,
    /// Revise change implementation.
    ReviseChangeImplementation,
    /// Create change docs.
    CreateChangeDocs,
    /// Review change docs.
    ReviewChangeDocs,
    /// Revise change docs.
    ReviseChangeDocs,
    /// Create change merge (programmatic).
    CreateChangeMerge,
}

/// Language of a project module.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigLanguage {
    /// Rust.
    Rust,
    /// Python.
    Python,
    /// TypeScript.
    TypeScript,
    /// JavaScript.
    JavaScript,
    /// Go.
    Go,
}

/// A module within a project (maps a directory path to a language).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectModule {
    /// Relative path to the module root (e.g., 'api/', 'projects/agentic-workflow/').
    pub path: String,
    /// Programming language of this module.
    pub language: ConfigLanguage,
    /// Optional framework (e.g., 'axum', 'react', 'django').
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
}

/// Project-level configuration supporting monorepos.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Module-to-language mappings.
    #[serde(default)]
    pub modules: Vec<ProjectModule>,
}

/// Spec scope configuration. Maps spec group names to parent subdirectory under .aw/tech-design/.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecsConfig {
    /// Maps group name to parent subdirectory under .aw/tech-design/. Example: { 'sdd': 'crates', 'conductor': 'projects' }.
    #[serde(default)]
    pub scopes: HashMap<String, String>,
}

/// Repository platform configuration — [agentic_workflow.repo_platform] in .aw/config.toml.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoPlatformConfig {
    /// VCS platform type (e.g. 'github', 'gitlab').
    #[serde(rename = "type")]
    pub type_: String,
    /// Repository in owner/repo format.
    pub repo: String,
    /// Optional self-hosted base host (e.g. 'gitlab.example.com'). None = use platform default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// Target branch for auto-PR creation (default: 'main').
    #[serde(default = "default_main_branch")]
    pub default_branch: String,
    /// Auto git-commit cclab/ changes after merge archive (default: false).
    #[serde(default)]
    pub auto_commit: bool,
    /// Auto-create PR after auto-commit. Requires auto_commit=true. (default: false).
    #[serde(default)]
    pub auto_pr: bool,
}

/// Docs generation phase configuration — [agentic_workflow.docs] in .aw/config.toml.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsConfig {
    /// Output directory for generated docs, relative to project root.
    #[serde(default = "default_docs_dir")]
    pub output_dir: String,
    /// Per-crate doc generation targets.
    pub targets: Vec<DocsTarget>,
}

/// Single doc generation target — [[agentic_workflow.docs.targets]] in config.toml.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsTarget {
    /// Crate name to match against change-affected crates.
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// Output guide file path relative to project root.
    pub guide: String,
    /// Target audience: developer | end-user | admin.
    pub audience: String,
    /// Guide section names to generate/update.
    pub sections: Vec<String>,
}

/// Tech design storage platform configuration — [agentic_workflow.tech_design_platform] in .aw/config.toml.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechDesignPlatformConfig {
    /// Storage backend type. Currently only 'local' supported.
    #[serde(rename = "type")]
    pub type_: String,
    /// Relative path to tech design storage directory from project root (default: '.aw/tech-design').
    #[serde(default = "default_tech_design_path")]
    pub path: String,
}

/// TDD test gate configuration — [agentic_workflow.test] in .aw/config.toml.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Global default test command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_cmd: Option<String>,
    /// Global setup command run before tests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,
    /// Global teardown command run after tests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teardown: Option<String>,
    /// Per-module test scope definitions [[agentic_workflow.test.scope]].
    #[serde(default)]
    pub scope: Vec<TestScope>,
}

/// Single test scope — [[agentic_workflow.test.scope]] in config.toml.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScope {
    /// Human-readable scope name.
    pub name: String,
    /// GitLab CI-style gitignore glob patterns matching file paths.
    pub changes: Vec<String>,
    /// Override test command for this scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_cmd: Option<String>,
    /// Override setup command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,
    /// Override teardown command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teardown: Option<String>,
}

/// SDD configuration.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SddConfig {
    /// SDD version (replaces .version file).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Interface mode: cli (default: cli).
    #[serde(default)]
    pub interface: SddInterface,
    /// Project configuration (monorepo-aware modules, not serialized to TOML).
    #[serde(skip, default)]
    pub project: ProjectConfig,
    /// Workflow iteration settings.
    #[serde(default)]
    pub workflow: WorkflowConfig,
    /// Spec scope configuration.
    #[serde(default, skip_serializing_if = "SpecsConfig::is_empty")]
    pub specs: SpecsConfig,
    /// Repository platform configuration — [agentic_workflow.repo_platform].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_platform: Option<RepoPlatformConfig>,
    /// Tech design storage platform configuration — [agentic_workflow.tech_design_platform].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tech_design_platform: Option<TechDesignPlatformConfig>,
    /// Docs generation phase configuration — [agentic_workflow.docs].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<DocsConfig>,
    /// TDD test gate configuration — [agentic_workflow.test].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<TestConfig>,
    /// Validation rules for spec files (fixed, not configurable).
    #[serde(skip, default)]
    pub validation: ValidationRules,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl ChangePhase {
    pub fn name(&self) -> &'static str {
        match self {
            ChangePhase::Proposed => "Proposed",
            ChangePhase::Exploring => "Exploring",
            ChangePhase::Challenged => "Challenged",
            ChangePhase::Rejected => "Rejected",
            ChangePhase::Implementing => "Implementing",
            ChangePhase::Complete => "Complete",
            ChangePhase::Archived => "Archived",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            ChangePhase::Proposed => "📝",
            ChangePhase::Exploring => "🔍",
            ChangePhase::Challenged => "🔍",
            ChangePhase::Rejected => "⛔",
            ChangePhase::Implementing => "🔨",
            ChangePhase::Complete => "✅",
            ChangePhase::Archived => "📦",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl Change {
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            id: id.into(),
            description: description.into(),
            phase: ChangePhase::Proposed,
            created_at: now.clone(),
            updated_at: now,
            deltas: Vec::new(),
            challenge: None,
            verification: None,
        }
    }

    /// Get the path to this change's directory
    pub fn path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".aw/changes").join(&self.id)
    }

    /// Get path to proposal.md
    pub fn proposal_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("proposal.md")
    }

    /// Get path to tasks.md
    pub fn tasks_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("tasks.md")
    }

    /// Get path to specs directory
    pub fn specs_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("specs")
    }

    /// Get path to IMPLEMENTATION.md
    pub fn implementation_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("IMPLEMENTATION.md")
    }

    /// Get path to REVIEW.md
    pub fn review_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("REVIEW.md")
    }

    /// Get path to VERIFICATION.md
    pub fn verification_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("VERIFICATION.md")
    }

    /// Update phase and timestamp
    pub fn update_phase(&mut self, phase: ChangePhase) {
        self.phase = phase;
        self.updated_at = chrono::Local::now().to_rfc3339();
    }

    /// Check if all required files exist
    pub fn validate_structure(&self, project_root: &Path) -> anyhow::Result<()> {
        let proposal = self.proposal_path(project_root);
        if !proposal.exists() {
            anyhow::bail!("Missing proposal.md at {:?}", proposal);
        }

        let tasks = self.tasks_path(project_root);
        if !tasks.exists() {
            anyhow::bail!("Missing tasks.md at {:?}", tasks);
        }

        let specs = self.specs_path(project_root);
        if !specs.exists() {
            anyhow::bail!("Missing specs/ directory at {:?}", specs);
        }

        Ok(())
    }
}

// =============================================================================
// SDD Configuration
// =============================================================================

// =============================================================================
// Workflow Stage Configuration
// =============================================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl StageConfig {
    /// Create a new stage config with the given agents
    pub fn new(agents: Vec<&str>) -> Self {
        Self {
            agents: agents.into_iter().map(String::from).collect(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl SddInterface {
    pub fn name(&self) -> &'static str {
        "cli"
    }

    pub fn description(&self) -> &'static str {
        "Uses `score` CLI commands"
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cli" => Some(Self::Cli),
            _ => None,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl WorkflowArtifact {
    /// Get display name for the artifact (matches config.toml key)
    pub fn name(&self) -> &'static str {
        match self {
            Self::RestructureInput => "restructure_input",
            Self::CreatePreClarifications => "create_pre_clarifications",
            Self::CreatePostClarifications => "create_post_clarifications",
            Self::CreateReferenceContext => "create_reference_context",
            Self::ReviewReferenceContext => "review_reference_context",
            Self::ReviseReferenceContext => "revise_reference_context",
            Self::CreateChangeSpec => "create_change_spec",
            Self::ReviewChangeSpec => "review_change_spec",
            Self::ReviseChangeSpec => "revise_change_spec",
            Self::CreateChangeImplementation => "create_change_implementation",
            Self::ReviewChangeImplementation => "review_change_implementation",
            Self::ReviseChangeImplementation => "revise_change_implementation",
            Self::CreateChangeDocs => "create_change_docs",
            Self::ReviewChangeDocs => "review_change_docs",
            Self::ReviseChangeDocs => "revise_change_docs",
            Self::CreateChangeMerge => "create_change_merge",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl ConfigLanguage {
    /// File extension for this language
    pub fn extension(&self) -> &str {
        match self {
            ConfigLanguage::Rust => "rs",
            ConfigLanguage::Python => "py",
            ConfigLanguage::TypeScript => "ts",
            ConfigLanguage::JavaScript => "js",
            ConfigLanguage::Go => "go",
        }
    }

    /// Test directory convention for this language
    pub fn test_dir(&self) -> &str {
        match self {
            ConfigLanguage::Rust => "tests",
            ConfigLanguage::Python => "tests",
            ConfigLanguage::TypeScript | ConfigLanguage::JavaScript => "__tests__",
            ConfigLanguage::Go => "",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl ProjectConfig {
    /// Find the language for a given file path by matching against module paths
    pub fn language_for_path(&self, file_path: &str) -> Option<ConfigLanguage> {
        // Find the longest matching module path (most specific match)
        self.modules
            .iter()
            .filter(|m| file_path.starts_with(&m.path))
            .max_by_key(|m| m.path.len())
            .map(|m| m.language)
    }

    /// Get the primary language (first module, or None)
    pub fn primary_language(&self) -> Option<ConfigLanguage> {
        self.modules.first().map(|m| m.language)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl SpecsConfig {
    /// Returns `true` when no scopes are configured (used for skip_serializing_if).
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
fn default_main_branch() -> String {
    "main".to_string()
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
fn default_docs_dir() -> String {
    "docs".to_string()
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
fn default_tech_design_path() -> String {
    ".aw/tech-design".to_string()
}

/// TDD test gate configuration — `[agentic_workflow.test]` in .aw/config.toml.
///
/// Presence of this section = test gate enabled. When absent, TestCheck phase
/// skips (same pattern as DocsConfig/DocsCheck).
// @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R1

/// Single test scope — `[[agentic_workflow.test.scope]]` in config.toml.
// @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R1

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl Default for SddConfig {
    fn default() -> Self {
        Self {
            version: None,
            interface: SddInterface::default(),
            project: ProjectConfig::default(),
            workflow: WorkflowConfig::default(),
            specs: SpecsConfig::default(),
            repo_platform: None,
            tech_design_platform: None,
            docs: None,
            test: None,
            validation: ValidationRules::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/change.md#source
impl SddConfig {
    /// Create config with specified interface.
    ///
    /// Score uses a fixed executor mapping (Claude Code subagent + mainthread
    /// hybrid) — there is no agent mode to configure.
    pub fn with_interface(interface: SddInterface) -> Self {
        Self {
            version: None,
            interface,
            workflow: WorkflowConfig::default(),
            specs: SpecsConfig::default(),
            ..Default::default()
        }
    }

    /// Set the version field
    pub fn set_version(&mut self, version: &str) {
        self.version = Some(version.to_string());
    }

    /// Load config from .aw/config.toml
    ///
    /// Platform configs (`repo_platform`, `tech_design_platform`) live under the
    /// `[agentic_workflow.*]` TOML namespace. After the primary deserialization
    /// we overlay them from the `[agentic_workflow]` table.
    pub fn load(project_root: &Path) -> anyhow::Result<Self> {
        let config_path = crate::shared::workspace::config_path(project_root);
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let mut config: SddConfig = toml::from_str(&content)?;

        // Extract platform configs from [agentic_workflow.*] sections.
        // These are nested in TOML but stored as flat fields on SddConfig.
        let parsed: toml::Value = toml::from_str(&content)?;
        if let Some(agentic_workflow) = parsed.get("agentic_workflow") {
            if config.repo_platform.is_none() {
                if let Some(rp) = agentic_workflow.get("repo_platform") {
                    config.repo_platform = rp.clone().try_into().ok();
                }
            }
            if config.tech_design_platform.is_none() {
                if let Some(sp) = agentic_workflow.get("tech_design_platform") {
                    config.tech_design_platform = sp.clone().try_into().ok();
                }
            }
            // @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R2
            if config.test.is_none() {
                if let Some(test) = agentic_workflow.get("test") {
                    config.test = test.clone().try_into().ok();
                }
            }
        }

        Ok(config)
    }

    /// Save config to .aw/config.toml
    pub fn save(&self, project_root: &Path) -> anyhow::Result<()> {
        let config_path = crate::shared::workspace::config_path(project_root);
        std::fs::create_dir_all(config_path.parent().unwrap())?;

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Validate the configuration
    ///
    /// Checks that required platform sections are present. Agent validation
    /// was removed when Score moved to hardcoded executor mapping — the
    /// `workflow.agents` field no longer exists.
    pub fn validate(&self) -> Result<(), String> {
        // Required platform sections
        if self.repo_platform.is_none() {
            return Err(
                "Missing [agentic_workflow.repo_platform] in .aw/config.toml.\n\
                 Add:\n\n\
                 [agentic_workflow.repo_platform]\n\
                 type = \"github\"\n\
                 repo = \"owner/repo\"\n\
                 default_branch = \"main\"\n\
                 auto_commit = true\n\
                 auto_pr = false\n\n\
                 Or rerun aw init to refresh platform settings"
                    .to_string(),
            );
        }
        if self.tech_design_platform.is_none() {
            return Err(
                "Missing [agentic_workflow.tech_design_platform] in .aw/config.toml.\n\
                 Add:\n\n\
                 [agentic_workflow.tech_design_platform]\n\
                 type = \"local\"\n\
                 path = \".aw/tech-design\"\n\n\
                 Or rerun aw init to refresh platform settings"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Load and validate config from .aw/config.toml
    ///
    /// Uses an in-process cache with mtime-based invalidation to avoid
    /// repeated TOML deserialization, which can cause stack overflow on
    /// tokio worker threads (2MB default stack) due to the large AgentsConfig struct.
    pub fn load_validated(project_root: &Path) -> anyhow::Result<Self> {
        struct CachedConfig {
            config: SddConfig,
            mtime: SystemTime,
            path: PathBuf,
        }

        static CONFIG_CACHE: OnceLock<Mutex<Option<CachedConfig>>> = OnceLock::new();

        let config_path = crate::shared::workspace::config_path(project_root);
        let cache_mutex = CONFIG_CACHE.get_or_init(|| Mutex::new(None));
        let mut cache = cache_mutex.lock().unwrap();

        // Check mtime for cache hit
        if let Some(cached) = cache.as_ref() {
            if cached.path == config_path {
                if let Ok(meta) = std::fs::metadata(&config_path) {
                    if let Ok(current_mtime) = meta.modified() {
                        if current_mtime == cached.mtime {
                            return Ok(cached.config.clone());
                        }
                    }
                }
            }
        }

        // Cache miss — load and validate
        let config = Self::load(project_root)?;
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("Config validation failed: {}", e))?;

        let mtime = std::fs::metadata(&config_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        *cache = Some(CachedConfig {
            config: config.clone(),
            mtime,
            path: config_path,
        });

        Ok(config)
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_workflow_artifact_name() {
        assert_eq!(
            WorkflowArtifact::CreateReferenceContext.name(),
            "create_reference_context"
        );
        assert_eq!(
            WorkflowArtifact::ReviewReferenceContext.name(),
            "review_reference_context"
        );
        assert_eq!(
            WorkflowArtifact::CreateChangeSpec.name(),
            "create_change_spec"
        );
        assert_eq!(
            WorkflowArtifact::ReviewChangeImplementation.name(),
            "review_change_implementation"
        );
    }

    #[test]
    fn test_sdd_config_validate() {
        // Default config fails validation — repo_platform and tech_design_platform are required.
        let config = SddConfig::default();
        assert!(config.validate().is_err());

        // Config with both platforms validates successfully.
        let mut config = SddConfig::default();
        config.repo_platform = Some(RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "owner/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: false,
            auto_pr: false,
        });
        config.tech_design_platform = Some(TechDesignPlatformConfig {
            type_: "local".to_string(),
            path: ".aw/tech-design".to_string(),
        });
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_project_config_monorepo() {
        let config = ProjectConfig {
            modules: vec![
                ProjectModule {
                    path: "api/".to_string(),
                    language: ConfigLanguage::Python,
                    framework: Some("django".to_string()),
                },
                ProjectModule {
                    path: "frontend/".to_string(),
                    language: ConfigLanguage::TypeScript,
                    framework: Some("react".to_string()),
                },
            ],
        };

        assert_eq!(config.primary_language(), Some(ConfigLanguage::Python));
        assert_eq!(
            config.language_for_path("api/views.py"),
            Some(ConfigLanguage::Python)
        );
        assert_eq!(
            config.language_for_path("frontend/src/App.tsx"),
            Some(ConfigLanguage::TypeScript)
        );
        assert_eq!(config.language_for_path("docs/readme.md"), None);
    }

    #[test]
    fn test_project_config_optional_framework() {
        let config = ProjectConfig {
            modules: vec![
                ProjectModule {
                    path: ".".to_string(),
                    language: ConfigLanguage::Rust,
                    framework: Some("axum".to_string()),
                },
                ProjectModule {
                    path: "scripts/".to_string(),
                    language: ConfigLanguage::Python,
                    framework: None,
                },
            ],
        };

        assert_eq!(config.modules[0].framework, Some("axum".to_string()));
        assert_eq!(config.modules[1].framework, None);
    }

    #[test]
    fn test_project_config_skipped_in_serialization() {
        // project is #[serde(skip)] — serialize should not include it,
        // deserialize always uses default (empty modules)
        let config = SddConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(!toml_str.contains("[project]"));
        let parsed: SddConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed.project.modules.is_empty());
    }

    #[test]
    fn test_project_config_empty_default() {
        let config = ProjectConfig::default();
        assert!(config.modules.is_empty());
        assert_eq!(config.primary_language(), None);
    }

    #[test]
    fn test_config_language_extension() {
        assert_eq!(ConfigLanguage::Rust.extension(), "rs");
        assert_eq!(ConfigLanguage::Python.extension(), "py");
        assert_eq!(ConfigLanguage::TypeScript.extension(), "ts");
        assert_eq!(ConfigLanguage::JavaScript.extension(), "js");
        assert_eq!(ConfigLanguage::Go.extension(), "go");
    }

    #[test]
    fn test_config_language_serde() {
        #[derive(Deserialize)]
        struct Wrapper {
            lang: ConfigLanguage,
        }

        let w: Wrapper = toml::from_str("lang = \"rust\"").unwrap();
        assert_eq!(w.lang, ConfigLanguage::Rust);

        let w: Wrapper = toml::from_str("lang = \"python\"").unwrap();
        assert_eq!(w.lang, ConfigLanguage::Python);

        // Invalid language should fail
        let result: Result<Wrapper, _> = toml::from_str("lang = \"cpp\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_sdd_config_backward_compat_no_project() {
        // Config without [project] section should deserialize with empty modules
        let toml_str = r#"
[workflow.agents]
implement = ["mainthread"]
"#;
        let config: SddConfig = toml::from_str(toml_str).unwrap();
        assert!(config.project.modules.is_empty());
    }

    // TC_config_deser — REQ-1: SddConfig.specs.scopes deserialized from [specs.scopes] TOML
    #[test]
    fn test_specs_config_deserialization() {
        let toml_str = r#"
[specs.scopes]
agentic-workflow = "projects"
conductor = "projects"
"#;
        let config: SddConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.specs.scopes.get("agentic-workflow"),
            Some(&"projects".to_string())
        );
        assert_eq!(
            config.specs.scopes.get("conductor"),
            Some(&"projects".to_string())
        );
        assert_eq!(config.specs.scopes.len(), 2);
    }

    #[test]
    fn test_specs_config_empty_by_default() {
        let config = SddConfig::default();
        assert!(config.specs.scopes.is_empty());
        assert!(config.specs.is_empty());
    }

    #[test]
    fn test_specs_config_not_serialized_when_empty() {
        // skip_serializing_if = "SpecsConfig::is_empty" — no [specs] block when empty
        let config = SddConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(
            !toml_str.contains("[specs]"),
            "empty specs should be omitted from TOML"
        );
    }

    #[test]
    fn test_specs_config_serialized_when_non_empty() {
        let mut config = SddConfig::default();
        config
            .specs
            .scopes
            .insert("agentic-workflow".to_string(), "projects".to_string());
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("agentic-workflow"));
        assert!(toml_str.contains("projects"));
    }

    #[test]
    fn test_config_roundtrip_with_scopes() {
        let mut config = SddConfig::default();
        config
            .specs
            .scopes
            .insert("agentic-workflow".to_string(), "projects".to_string());
        config
            .specs
            .scopes
            .insert("conductor".to_string(), "projects".to_string());

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: SddConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(
            parsed.specs.scopes.get("agentic-workflow"),
            Some(&"projects".to_string())
        );
        assert_eq!(
            parsed.specs.scopes.get("conductor"),
            Some(&"projects".to_string())
        );
        assert_eq!(parsed.specs.scopes.len(), 2);
    }

    #[test]
    fn test_specs_config_missing_section_gives_empty_scopes() {
        // Projects without [specs.scopes] deserialize with empty map (backward compat)
        let toml_str = r#"
[workflow.agents]
create_change_spec = ["mainthread"]
"#;
        let config: SddConfig = toml::from_str(toml_str).unwrap();
        assert!(config.specs.scopes.is_empty());
    }

    // S1: Config parsing — round-trip with TestConfig
    // REQ: REQ-001
    #[test]
    fn test_test_config_deserialization() {
        let toml_str = r#"
[agentic_workflow.test]
test_cmd = "cargo test"
setup = "docker compose up -d"
teardown = "docker compose down"

[[agentic_workflow.test.scope]]
name = "conductor"
changes = ["projects/conductor/**"]
test_cmd = "bash test-env.sh"

[[agentic_workflow.test.scope]]
name = "queue"
changes = ["crates/cclab-queue/**"]
"#;
        // Parse the full table and extract [agentic_workflow.test]
        let parsed: toml::Value = toml::from_str(toml_str).unwrap();
        let test_val = parsed.get("agentic_workflow").unwrap().get("test").unwrap();
        let test_config: TestConfig = test_val.clone().try_into().unwrap();

        assert_eq!(test_config.test_cmd, Some("cargo test".to_string()));
        assert_eq!(test_config.setup, Some("docker compose up -d".to_string()));
        assert_eq!(
            test_config.teardown,
            Some("docker compose down".to_string())
        );
        assert_eq!(test_config.scope.len(), 2);
        assert_eq!(test_config.scope[0].name, "conductor");
        assert_eq!(test_config.scope[0].changes, vec!["projects/conductor/**"]);
        assert_eq!(
            test_config.scope[0].test_cmd,
            Some("bash test-env.sh".to_string())
        );
        assert_eq!(test_config.scope[1].name, "queue");
        assert_eq!(test_config.scope[1].test_cmd, None); // inherits from global
    }

    // S2: Config absent — test field is None
    // REQ: REQ-002
    #[test]
    fn test_test_config_absent_is_none() {
        let toml_str = r#"
[workflow]
"#;
        let config: SddConfig = toml::from_str(toml_str).unwrap();
        assert!(config.test.is_none());
    }

    // Test that TestConfig is not serialized when None
    #[test]
    fn test_test_config_not_serialized_when_none() {
        let config = SddConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(
            !toml_str.contains("[test]"),
            "None test config should not appear in TOML"
        );
    }

    // Test roundtrip: TestConfig serializes and deserializes correctly
    #[test]
    fn test_test_config_roundtrip() {
        let config = TestConfig {
            test_cmd: Some("cargo test".to_string()),
            setup: None,
            teardown: None,
            scope: vec![TestScope {
                name: "agentic-workflow".to_string(),
                changes: vec!["projects/agentic-workflow/**".to_string()],
                test_cmd: None,
                setup: None,
                teardown: None,
            }],
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: TestConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.test_cmd, Some("cargo test".to_string()));
        assert_eq!(parsed.scope.len(), 1);
        assert_eq!(parsed.scope[0].name, "agentic-workflow");
        assert_eq!(
            parsed.scope[0].changes,
            vec!["projects/agentic-workflow/**".to_string()]
        );
    }
}

// CODEGEN-END
