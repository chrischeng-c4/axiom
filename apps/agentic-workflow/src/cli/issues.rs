// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
// CODEGEN-BEGIN
//! `aw wi` CLI -- list/show/sync/create/update/close/find across
//! local + GitHub + GitLab backends.
//!
//! Backend selection is resolved from `aw.toml`
//! (`[agentic_workflow.issue_platform]` / `[agentic_workflow.repo_platform]`); there is no
//! `--backend` flag. Workflow-facing detail/validation commands default to
//! machine-parseable JSON; `--human` keeps legacy prose where available.

use crate::issues::{
    make_backend, remote_read_cache_backend, resolve_default_backend, Issue, IssueBackend,
    IssueErrorCode, IssueFilter, IssuePatch, IssueState, IssueType, LocalBackend, ShipStatus,
};
use crate::parser::frontmatter::parse_document;
use crate::services::issue_parser::{validate_structured_issue, ValidationError};
use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

// Top-level args for `aw wi`.
// @spec apps/agentic-workflow/tech-design/surface/issues_top.md#schema
#[derive(Debug, Args)]
pub struct IssuesArgs {
    /// The selected subcommand.
    #[command(subcommand)]
    pub command: IssuesCommand,
}

// Available subcommands for `aw wi`.
// @spec apps/agentic-workflow/tech-design/surface/issues_top.md#schema
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md#cli
#[derive(Debug, Subcommand)]
pub enum IssuesCommand {
    /// Work with local draft work-items before creating a tracker issue.
    Draft(DraftArgs),
    /// List work-items from a backend.
    List(ListArgs),
    /// Show a single work-item by slug or numeric id.
    Show(ShowArgs),
    /// Retired (#1899): emits a structured redirect to `aw goal wi <id>`,
    /// the unified re-home of the root-driven work-item runner.
    Run(WiRunArgs),
    /// Create a new work-item.
    Create(CreateArgs),
    /// Update an existing work-item's metadata or body.
    Update(UpdateArgs),
    /// Close a work-item, optionally with a reason.
    Close(CloseArgs),
    /// Search work-items by text query.
    Find(FindArgs),
    /// Plan work-item candidates from a confirmed capability map / README.
    Plan(PlanArgs),
    /// Submit independent review evidence for a capability WI plan.
    PlanReview(PlanReviewArgs),
    /// Plan a project phase from the current work-item inventory.
    Epicize(EpicizeArgs),
    /// Split epic/roadmap-sized work into atomic work-item candidates.
    Atomize(AtomizeArgs),
    /// Re-rank issue backlog by priority, dependency, and readiness.
    Prioritize(PrioritizeArgs),
    /// Fill the Reference Context section via agent exploration.
    Enrich(EnrichArgs),
    /// Validate work-item quality and boundedness.
    Validate(ValidateArgs),
    /// Fill work-item sections via structured round-trip.
    FillSection(FillSectionArgs),
}
#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct DraftArgs {
    #[command(subcommand)]
    pub command: DraftCommand,
}

#[derive(Debug, Subcommand)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub enum DraftCommand {
    /// Initialize a local draft work-item under /tmp/aw/workspaces/<workspace>/workitems/{project}/.
    Init(DraftInitArgs),
    /// Fill sections in a local draft work-item.
    Fill(DraftFillArgs),
    /// Validate a local draft work-item.
    Validate(DraftValidateArgs),
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct DraftInitArgs {
    /// Work-item title.
    #[arg(long)]
    pub title: String,

    /// Work-item type.
    #[arg(long = "type")]
    pub issue_type: TypeFilter,

    /// Project name. Required on main; inferred from project branches otherwise.
    #[arg(long)]
    pub project: Option<String>,

    /// Inline body text. Free text is wrapped in the structured draft template;
    /// structured bodies are normalized before the draft is written.
    #[arg(long, conflicts_with = "body_file")]
    pub body: Option<String>,

    /// Read body from a file path, or `-` for stdin. Free text is wrapped in
    /// the structured draft template.
    #[arg(long)]
    pub body_file: Option<String>,

    /// Priority level.
    #[arg(long = "priority")]
    pub priority: Option<PriorityFilter>,

    /// Agent name.
    #[arg(long = "agent")]
    pub agent: Option<String>,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct DraftFillArgs {
    /// Draft markdown file created by `aw wi draft init`.
    pub draft_path: PathBuf,

    /// Which section to fill. `all` replaces/validates every structured section.
    #[arg(long, default_value = "all")]
    pub section: String,

    /// Inline replacement markdown body. Requirements and Scope are normalized
    /// before validation.
    #[arg(long, conflicts_with = "body_file")]
    pub body: Option<String>,

    /// Read replacement markdown body from a file path, or `-` for stdin.
    /// Requirements and Scope are normalized before validation.
    #[arg(long)]
    pub body_file: Option<String>,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct DraftValidateArgs {
    /// Draft markdown file created by `aw wi draft init`.
    pub draft_path: PathBuf,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct ListArgs {
    /// Filter by state.
    #[arg(long)]
    pub state: Option<StateFilter>,

    /// Filter by type (matches the `type:*` label).
    #[arg(long = "type")]
    pub issue_type: Option<TypeFilter>,

    /// Filter by label (exact match against any of the work-item's labels).
    #[arg(long)]
    pub label: Option<String>,

    /// Filter by configured project name.
    #[arg(long)]
    pub project: Option<String>,

    /// Filter by author username.
    #[arg(long)]
    pub author: Option<String>,

    /// Output machine-readable JSON instead of a pretty table.
    #[arg(long)]
    pub json: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct ShowArgs {
    /// Work-item identifier (slug for local, numeric for github).
    pub id: String,

    /// Deprecated compatibility no-op: agent JSON is the default.
    #[arg(long, hide = true)]
    pub json: bool,

    /// Emit human-readable detail instead of the default agent JSON envelope.
    #[arg(long)]
    pub human: bool,

    /// Pretty-print the default JSON envelope for debugging.
    #[arg(long)]
    pub pretty: bool,

    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct WiRunArgs {
    /// Work-item identifier (slug for local, numeric for github).
    pub id: String,

    /// Emit human-readable text instead of the default agent JSON envelope.
    #[arg(long)]
    pub human: bool,

    /// Pretty-print the default JSON envelope for debugging.
    #[arg(long)]
    pub pretty: bool,

    /// Generate a /goal-ready prompt for this work item instead of the normal run envelope.
    #[arg(long)]
    pub goal: bool,
}

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R1
// @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Draft markdown file created by `aw wi draft init`.
    pub draft_path: Option<PathBuf>,

    /// Work-item title.
    #[arg(long, required_unless_present = "draft_path")]
    pub title: Option<String>,

    /// Work-item type. Closed enum: bug | enhancement | refactor | test | epic.
    /// Emits a `type::<value>` scoped label.
    #[arg(long = "type", required_unless_present = "draft_path")]
    pub issue_type: Option<TypeFilter>,

    /// Inline body text. Mutually exclusive with --body-file.
    #[arg(long, conflicts_with = "body_file")]
    pub body: Option<String>,

    /// Read body from a file path, or `-` for stdin.
    #[arg(long)]
    pub body_file: Option<String>,

    /// Project name (repeatable). Resolved against `[[projects]]` in
    /// `aw.toml`; emits each entry's `label` field. Cardinality
    /// rules (per `--type`):
    ///   * `epic`  → 0 or 1 value (lead/owner; multi-project spans live in body)
    ///   * other types → exactly 1 value.
    /// Unknown name → error envelope.
    #[arg(long = "project")]
    pub projects: Vec<String>,

    /// Priority level. Closed enum: p0 | p1 | p2 | p3.
    /// Emits a `priority::<value>` scoped label.
    #[arg(long = "priority")]
    pub priority: Option<PriorityFilter>,

    /// Agent name. Resolved against `[[agents]]` in `aw.toml`;
    /// emits the entry's `label` field (e.g. `agent::claude-code`).
    /// Unknown name → error envelope.
    #[arg(long = "agent")]
    pub agent: Option<String>,

    /// Deprecated compatibility no-op. Backend selection is configured in
    /// `aw.toml`; local-only authoring lives under `aw wi draft`.
    #[arg(long, hide = true)]
    pub remote: bool,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

// Priority levels accepted by `aw wi create --priority`.
// @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum PriorityFilter {
    P0,
    P1,
    P2,
    P3,
}

// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
impl PriorityFilter {
    /// Returns the label suffix for `priority::<suffix>`.
    pub fn as_label_suffix(&self) -> &'static str {
        match self {
            PriorityFilter::P0 => "p0",
            PriorityFilter::P1 => "p1",
            PriorityFilter::P2 => "p2",
            PriorityFilter::P3 => "p3",
        }
    }
}

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Work-item identifier (slug for local, numeric for remote).
    pub id: String,

    /// New title.
    #[arg(long)]
    pub title: Option<String>,

    /// New state.
    #[arg(long)]
    pub state: Option<StateFilter>,

    /// Add a label (repeatable).
    #[arg(long = "add-label")]
    pub add_labels: Vec<String>,

    /// Remove a label (repeatable).
    #[arg(long = "remove-label")]
    pub remove_labels: Vec<String>,

    /// Read replacement body from a file path, or `-` for stdin.
    #[arg(long)]
    pub body_file: Option<String>,

    /// Also push to remote backend via `gh issue edit`.
    #[arg(long)]
    pub push: bool,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R3
#[derive(Debug, Args)]
pub struct CloseArgs {
    /// Work-item identifier (slug for local, numeric for remote).
    pub id: String,

    /// Close reason (optional comment).
    #[arg(long)]
    pub reason: Option<String>,

    /// Also close on remote backend.
    #[arg(long)]
    pub push: bool,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R4
#[derive(Debug, Args)]
pub struct FindArgs {
    /// Text query to search for.
    pub query: String,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct PlanArgs {
    /// Project name. Defaults to the current project branch when omitted.
    #[arg(long)]
    pub project: Option<String>,

    /// Optional planning title.
    #[arg(long)]
    pub title: Option<String>,

    /// Capability map path. Defaults to [[projects]].cap_path or [[projects]].path/README.md.
    #[arg(long = "cap-path")]
    pub cap_path: Option<PathBuf>,

    /// Write plan to this path instead of /tmp/aw/workspaces/<workspace>/workitems/{project}/capability-plan/.
    /// Direct /tmp/*.md outputs are rejected; keep tmp artifacts under /tmp/aw/workspaces/<workspace>/workitems/{project}/capability-plan/.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md#cli
pub struct PlanReviewArgs {
    /// Agent- or human-backed digest-bound capability-plan review payload.
    #[arg(long = "evidence-file")]
    pub evidence_file: PathBuf,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct EpicizeArgs {
    /// Project name. Defaults to the current project branch when omitted.
    #[arg(long)]
    pub project: Option<String>,

    /// Optional phase title.
    #[arg(long)]
    pub title: Option<String>,

    /// Write plan to this path instead of /tmp/aw/workspaces/<workspace>/workitems/{project}/epics/.
    /// Direct /tmp/*.md outputs are rejected; keep tmp artifacts under /tmp/aw/workspaces/<workspace>/workitems/{project}/epics/.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct AtomizeArgs {
    /// Project name. Defaults to the current project branch when omitted.
    #[arg(long)]
    pub project: Option<String>,

    /// Optional atomization title.
    #[arg(long)]
    pub title: Option<String>,

    /// Write plan to this path instead of /tmp/aw/workspaces/<workspace>/workitems/{project}/atomize/.
    /// Direct /tmp/*.md outputs are rejected; keep tmp artifacts under /tmp/aw/workspaces/<workspace>/workitems/{project}/atomize/.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct PrioritizeArgs {
    /// Project name. Defaults to the current project branch when omitted.
    #[arg(long)]
    pub project: Option<String>,

    /// Optional planning title.
    #[arg(long)]
    pub title: Option<String>,

    /// Write plan to this path instead of /tmp/aw/workspaces/<workspace>/workitems/{project}/priorities/.
    /// Direct /tmp/*.md outputs are rejected; keep tmp artifacts under /tmp/aw/workspaces/<workspace>/workitems/{project}/priorities/.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct EnrichArgs {
    /// Work-item slug.
    pub slug: String,
}

// Deterministic admission gate for bounded work-items.
#[derive(Debug, Args)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md#cli
pub struct ValidateArgs {
    /// Work-item slug.
    pub slug: String,

    /// Deprecated compatibility no-op: agent JSON is the default.
    #[arg(long, hide = true)]
    pub json: bool,

    /// Emit human-readable validation text instead of the default agent JSON envelope.
    #[arg(long)]
    pub human: bool,

    /// Pretty-print the default JSON envelope for debugging.
    #[arg(long)]
    pub pretty: bool,

    /// GitHub/GitLab repo override.
    #[arg(long)]
    pub repo: Option<String>,
}

// @spec apps/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R4 #R5
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md#cli
#[derive(Debug, Args)]
pub struct FillSectionArgs {
    /// Work-item slug.
    #[arg(long)]
    pub slug: String,

    /// Which section to fill. `all` (default) means the subagent writes the
    /// complete body in one pass.
    #[arg(long, default_value = "all")]
    pub section: String,

    /// Apply mode: merge
    /// `/tmp/aw/workspaces/<workspace>/payloads/wi/<slug>/body.md` into the
    /// checkout issue and emit the next validate envelope. Without this
    /// flag the CLI prints a plain-text brief.
    #[arg(long)]
    pub apply: bool,

    /// Deprecated transcript metrics accepted for older hook payloads.
    #[arg(long)]
    pub duration_ms: Option<u64>,
    #[arg(long)]
    pub tokens_in: Option<u64>,
    #[arg(long)]
    pub tokens_out: Option<u64>,
    #[arg(long)]
    pub cache_read_tokens: Option<u64>,
    #[arg(long)]
    pub tool_calls: Option<u64>,
    #[arg(long)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub enum BackendKind {
    Local,
    Github,
    Gitlab,
    Jira,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub enum StateFilter {
    Open,
    Closed,
    Draft,
}

// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
impl From<StateFilter> for IssueState {
    fn from(s: StateFilter) -> Self {
        match s {
            StateFilter::Open => IssueState::Open,
            StateFilter::Closed => IssueState::Closed,
            StateFilter::Draft => IssueState::Draft,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub enum TypeFilter {
    Epic,
    Change,
    Bug,
    Enhancement,
    Refactor,
    Test,
}

// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
impl From<TypeFilter> for IssueType {
    fn from(t: TypeFilter) -> Self {
        match t {
            TypeFilter::Epic => IssueType::Epic,
            TypeFilter::Change => IssueType::Enhancement,
            TypeFilter::Bug => IssueType::Bug,
            TypeFilter::Enhancement => IssueType::Enhancement,
            TypeFilter::Refactor => IssueType::Refactor,
            TypeFilter::Test => IssueType::Test,
        }
    }
}

// ---------------------------------------------------------------------------
// Structured error helpers (R7)
// ---------------------------------------------------------------------------

fn emit_create_envelope_error(slug_or_title: &str, message: &str) -> ! {
    let env = serde_json::json!({
        "action": "error",
        "slug": slug_or_title,
        "message": message,
    });
    println!("{}", env);
    std::process::exit(2);
}

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R7
fn emit_json_error(message: &str, code: IssueErrorCode) -> ! {
    let err = serde_json::json!({
        "error": message,
        "code": code.as_str(),
    });
    eprintln!("{}", err);
    std::process::exit(code.exit_code());
}

// Emit a structured validation error to stderr and exit with code 2.
///
// Wire format per `apps/agentic-workflow/logic/structured-issue.md` R6:
// `{"error": "...", "code": "VALIDATION_ERROR", "missing": [...]}`.
// @spec structured-issue#R6
fn emit_validation_error(err: &ValidationError) -> ! {
    // Always serialize the full struct so callers see the `missing` array.
    match serde_json::to_string(err) {
        Ok(s) => eprintln!("{}", s),
        Err(_) => eprintln!(
            "{{\"error\":\"{}\",\"code\":\"VALIDATION_ERROR\",\"missing\":[]}}",
            err.error.replace('"', "\\\"")
        ),
    }
    std::process::exit(2);
}

// Read body content from `--body-file` (path or `-` for stdin).
fn read_body_file(path: &str) -> Result<String> {
    if path == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read body from stdin")?;
        Ok(buf)
    } else {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read body from '{}'", path))
    }
}

fn default_structured_issue_body(title: &str) -> String {
    let title = title.trim();
    let title = if title.is_empty() {
        "the requested work"
    } else {
        title
    };
    let table_title = markdown_table_cell(title);
    format!(
        "## Problem\n\n{title}\n\n## Capability Alignment\n\nCapability: {title}\nCapability Gap: {title} is not yet delivered.\nProgress Evidence: Completion evidence is recorded on this work item.\n\n## Requirements\n\n- R1: Deliver {title}.\n\n## Scope\n\n### In Scope\n- Deliver the bounded change described by {title}.\n\n### Out of Scope\n- Unrelated work outside this work item.\n\n## Acceptance Criteria\n\n- AC1: {title} is implemented and verified.\n\n## Reference Context\n\n### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| {table_title} | source request |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| wi-draft | update | {table_title} |\n"
    )
}

fn body_from_inputs(
    title: &str,
    body: &Option<String>,
    body_file: &Option<String>,
) -> Result<String> {
    if let Some(bf) = body_file {
        read_body_file(bf)
    } else if let Some(b) = body {
        Ok(b.clone())
    } else {
        Ok(default_structured_issue_body(title))
    }
}

// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-draft-valid-by-construction.md#draft_authoring_contract
fn draft_body_from_inputs(
    title: &str,
    body: &Option<String>,
    body_file: &Option<String>,
) -> Result<String> {
    let raw = body_from_inputs(title, body, body_file)?;
    Ok(normalize_initial_draft_body(title, &raw))
}

fn normalize_initial_draft_body(title: &str, raw_body: &str) -> String {
    let body = raw_body.trim_start();
    if body.trim().is_empty() {
        return default_structured_issue_body(title);
    }
    let base = default_structured_issue_body(title);
    let merged = if looks_like_structured_attempt(body) {
        merge_all_sections(&base, body)
    } else {
        replace_h2_content(&base, "## Problem", &format!("\n{}\n\n", body))
    };
    normalize_known_draft_sections(&merged)
}

fn replace_h2_content(body: &str, heading: &str, replacement: &str) -> String {
    let mut sections = split_body_by_h2(body);
    for (section_heading, content) in &mut sections {
        if section_heading == heading {
            *content = replacement.to_string();
            return join_body_from_sections(&sections);
        }
    }
    let mut out = body.to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(heading);
    out.push('\n');
    out.push_str(replacement);
    out
}

// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-draft-valid-by-construction.md#draft_authoring_contract
fn normalize_known_draft_sections(body: &str) -> String {
    let mut sections = split_body_by_h2(body);
    for (heading, content) in &mut sections {
        match heading.as_str() {
            "## Requirements" => *content = normalize_requirements_section_content(content),
            "## Scope" => *content = normalize_scope_section_content(content),
            _ => {}
        }
    }
    join_body_from_sections(&sections)
}

fn normalize_requirements_section_content(content: &str) -> String {
    let mut out = Vec::new();
    let mut saw_list_item = false;
    let mut next_id = 1usize;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            saw_list_item = true;
            let item = trimmed.trim_start_matches("- ").trim_start_matches("* ");
            out.push(format!("- {}", normalize_requirement_item(item, next_id)));
            next_id += 1;
        } else {
            out.push(line.to_string());
        }
    }

    if !saw_list_item {
        let text = content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !text.is_empty() {
            out.clear();
            out.push(String::new());
            out.push(format!("- R1: {}", text));
        }
    }

    let mut normalized = out.join("\n");
    if !normalized.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

fn normalize_requirement_item(item: &str, fallback_id: usize) -> String {
    let trimmed = item.trim();
    if plain_rid_requirement_item(trimmed) {
        return trimmed.to_string();
    }
    if let Some((id, text)) = bold_rid_requirement_item(trimmed) {
        let text = text.trim_start_matches(':').trim();
        if text.is_empty() {
            return format!("{}: requirement", id);
        }
        return format!("{}: {}", id, text);
    }
    format!(
        "R{}: {}",
        fallback_id,
        trimmed.trim_start_matches(':').trim()
    )
}

fn plain_rid_requirement_item(item: &str) -> bool {
    let Some(rest) = item.strip_prefix('R') else {
        return false;
    };
    let Some(colon) = rest.find(':') else {
        return false;
    };
    let num = &rest[..colon];
    !num.is_empty() && num.chars().all(|c| c.is_ascii_digit())
}

fn bold_rid_requirement_item(item: &str) -> Option<(&str, &str)> {
    let rest = item.strip_prefix("**")?;
    let end = rest.find("**")?;
    let id = &rest[..end];
    if !plain_rid_requirement_item(&format!("{}:", id)) {
        return None;
    }
    let mut tail = rest[end + 2..].trim_start();
    if let Some(after_priority) = tail.strip_prefix('(') {
        if let Some(end_priority) = after_priority.find(')') {
            tail = after_priority[end_priority + 1..].trim_start();
        }
    }
    Some((id, tail))
}

fn normalize_scope_section_content(content: &str) -> String {
    let lower = content.to_ascii_lowercase();
    let has_in = lower.contains("### in scope") || lower.contains("### in-scope");
    let has_out = lower.contains("### out of scope") || lower.contains("### out-of-scope");
    if has_in && has_out {
        let mut out = content.to_string();
        if !out.ends_with('\n') {
            out.push('\n');
        }
        return out;
    }

    if let Some(out) = normalize_loose_scope_labeled_content(content) {
        return out;
    }

    let in_scope = scope_list_items(content);
    format!(
        "\n### In Scope\n{}\n\n### Out of Scope\n- Unrelated work outside this work item.\n",
        in_scope
    )
}

#[derive(Clone, Copy)]
enum LooseScopeBucket {
    In,
    Out,
}

fn normalize_loose_scope_labeled_content(content: &str) -> Option<String> {
    let mut current = None;
    let mut saw_in = false;
    let mut saw_out = false;
    let mut in_scope = Vec::new();
    let mut out_scope = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(bucket) = loose_scope_label(trimmed) {
            current = Some(bucket);
            match bucket {
                LooseScopeBucket::In => saw_in = true,
                LooseScopeBucket::Out => saw_out = true,
            }
            continue;
        }
        let Some(bucket) = current else {
            continue;
        };
        let item = trimmed
            .trim_start_matches("- ")
            .trim_start_matches("* ")
            .trim();
        if item.is_empty() {
            continue;
        }
        match bucket {
            LooseScopeBucket::In => in_scope.push(format!("- {item}")),
            LooseScopeBucket::Out => out_scope.push(format!("- {item}")),
        }
    }

    if !(saw_in && saw_out) {
        return None;
    }

    let in_scope = if in_scope.is_empty() {
        "- Scope explicitly labeled but empty.".to_string()
    } else {
        in_scope.join("\n")
    };
    let out_scope = if out_scope.is_empty() {
        "- No explicit exclusions.".to_string()
    } else {
        out_scope.join("\n")
    };
    Some(format!(
        "\n### In Scope\n{}\n\n### Out of Scope\n{}\n",
        in_scope, out_scope
    ))
}

fn loose_scope_label(line: &str) -> Option<LooseScopeBucket> {
    let normalized = line
        .trim_matches('*')
        .trim()
        .trim_end_matches(':')
        .trim()
        .to_ascii_lowercase();
    match normalized.as_str() {
        "in scope" | "in-scope" => Some(LooseScopeBucket::In),
        "out of scope" | "out-of-scope" => Some(LooseScopeBucket::Out),
        _ => None,
    }
}

fn scope_list_items(content: &str) -> String {
    let list_items = content
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("- ") || line.starts_with("* "))
        .map(|line| {
            let item = line
                .trim_start_matches("- ")
                .trim_start_matches("* ")
                .trim();
            format!("- {}", item)
        })
        .collect::<Vec<_>>();
    if !list_items.is_empty() {
        return list_items.join("\n");
    }
    let text = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("### "))
        .collect::<Vec<_>>()
        .join(" ");
    if text.is_empty() {
        "- Deliver the bounded work item.".to_string()
    } else {
        format!("- {}", text)
    }
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub async fn run(args: IssuesArgs) -> Result<()> {
    match args.command {
        IssuesCommand::Draft(a) => run_draft(a).await,
        IssuesCommand::List(a) => run_list(a).await,
        IssuesCommand::Show(a) => run_show(a).await,
        IssuesCommand::Run(a) => run_wi_run(a).await,
        IssuesCommand::Create(a) => run_create(a).await,
        IssuesCommand::Update(a) => run_update(a).await,
        IssuesCommand::Close(a) => run_close(a).await,
        IssuesCommand::Find(a) => run_find(a).await,
        IssuesCommand::Plan(a) => run_plan(a).await,
        IssuesCommand::PlanReview(a) => run_plan_review(a).await,
        IssuesCommand::Epicize(a) => run_epicize(a).await,
        IssuesCommand::Atomize(a) => run_atomize(a).await,
        IssuesCommand::Prioritize(a) => run_prioritize(a).await,
        IssuesCommand::Enrich(a) => run_enrich(a).await,
        IssuesCommand::Validate(a) => run_validate(a).await,
        IssuesCommand::FillSection(a) => run_fill_section(a).await,
    }
}

async fn run_draft(args: DraftArgs) -> Result<()> {
    match args.command {
        DraftCommand::Init(a) => run_draft_init(a).await,
        DraftCommand::Fill(a) => run_draft_fill(a).await,
        DraftCommand::Validate(a) => run_draft_validate(a).await,
    }
}

// ---------------------------------------------------------------------------
// Backend resolution helper (Phase A)
// ---------------------------------------------------------------------------

// Resolve the backend triple `(kind, repo, host)` from `aw.toml`.
// `--repo` overrides the resolved repo.
fn resolve_backend(
    repo_override: Option<String>,
    project_root: &std::path::Path,
) -> Result<(String, Option<String>, Option<String>)> {
    let (kind, resolved_repo, host) = resolve_default_backend(project_root)?;
    let repo = repo_override.or(resolved_repo);
    Ok((kind, repo, host))
}

// ---------------------------------------------------------------------------
// List (with R5 broken-reference warnings)
// ---------------------------------------------------------------------------

async fn run_list(args: ListArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let (kind, repo, host) = resolve_backend(args.repo.clone(), &project_root)?;
    let backend =
        make_backend(&kind, &project_root, repo, host).context("Failed to create backend")?;
    let label = resolve_list_label_filter(
        &project_root,
        args.label.as_deref(),
        args.project.as_deref(),
    )?;

    let filter = IssueFilter {
        state: args.state.map(Into::into),
        issue_type: args.issue_type.map(Into::into),
        label,
        author: args.author.clone(),
    };

    let issues = backend.list(&filter).await?;

    // @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R5 — warn on broken cross-references
    if backend.name() == "local" {
        check_broken_references(&issues, &project_root);
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&issues)?);
    } else {
        print_table(&issues, backend.name());
    }
    Ok(())
}

fn resolve_list_label_filter(
    project_root: &Path,
    label: Option<&str>,
    project: Option<&str>,
) -> Result<Option<String>> {
    if label.is_some() && project.is_some() {
        return Err(anyhow::anyhow!("use either --label or --project, not both"));
    }
    if let Some(project) = project {
        return Ok(Some(
            resolve_project_label(project_root, project)
                .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?,
        ));
    }
    Ok(label.map(ToString::to_string))
}

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R5
fn check_broken_references(issues: &[Issue], project_root: &std::path::Path) {
    let slugs: std::collections::HashSet<&str> = issues.iter().map(|i| i.slug.as_str()).collect();
    let issues_dir = crate::shared::workspace::issues_path(project_root);

    for issue in issues {
        for ref_slug in issue.related.iter().chain(issue.implements.iter()) {
            // A reference is valid if it matches an existing slug OR is an existing file path
            let slug_exists = slugs.contains(ref_slug.as_str());
            let path_exists = if ref_slug.starts_with('/') || ref_slug.starts_with('.') {
                project_root.join(ref_slug).exists()
            } else {
                let filename = format!("{}.md", ref_slug);
                issues_dir.join("open").join(&filename).exists()
                    || issues_dir.join("closed").join(&filename).exists()
            };
            if !slug_exists && !path_exists {
                eprintln!("warning: broken reference '{}' in {}", ref_slug, issue.slug);
            }
        }
    }
}

// Read declared `[[projects]].label` values from `aw.toml`.
///
// Returns an empty vec when the config is missing, unparseable, has no
// `[[projects]]` table, or no entry declares a `label`. Callers treat
// "empty" as "value-check disabled" — degrade gracefully rather than
// fail loud, since the config may legitimately lack managed projects
// (fresh repo, pre-Phase-C tree, etc.).
fn read_known_project_labels(project_root: &Path) -> Vec<String> {
    if let Ok(rows) = crate::services::project_registry::load_project_config_rows(project_root) {
        let labels = rows
            .into_iter()
            .map(|row| row.label_or_default())
            .collect::<Vec<_>>();
        if !labels.is_empty() {
            return labels;
        }
    }
    let path = project_root.join("aw.toml");
    let Ok(body) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(value) = body.parse::<toml::Value>() else {
        return Vec::new();
    };
    let Some(projects) = value.get("projects").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    projects
        .iter()
        .filter_map(|p| p.get("label").and_then(|l| l.as_str()))
        .map(String::from)
        .collect()
}

// Read `[[projects]]` entries as a `(name, label)` list from `aw.toml`.
///
// Order is preserved from the config file so error messages and the
// emitted label vector are deterministic. Empty when the config is
// missing / unparseable / has no `[[projects]]` table.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
pub(crate) fn read_known_project_name_label_pairs(project_root: &Path) -> Vec<(String, String)> {
    read_name_label_pairs(project_root, "projects")
}

// Read `[[agents]]` entries as a `(name, label)` list from `aw.toml`.
///
// Same shape and contract as `read_known_project_name_label_pairs`.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
#[cfg(test)]
pub(crate) fn read_known_agent_name_label_pairs(project_root: &Path) -> Vec<(String, String)> {
    read_name_label_pairs(project_root, "agents")
}

// Shared loader for `[[projects]]` / `[[agents]]` tables. Reads the
// `aw.toml`, returns the entries as `(name, label)` pairs.
fn read_name_label_pairs(project_root: &Path, table: &str) -> Vec<(String, String)> {
    read_name_aliases_label(project_root, table)
        .into_iter()
        .map(|(name, _, label)| (name, label))
        .collect()
}

// Shared loader for `[[projects]]` / `[[agents]]` tables. Reads the
// `aw.toml`, returns the entries as `(name, aliases, label)`
// triples. Each entry's optional `aliases` array is a list of shorthand
// names that resolve to the same `label` as the canonical `name`.
fn read_name_aliases_label(project_root: &Path, table: &str) -> Vec<(String, Vec<String>, String)> {
    if table == "projects" {
        if let Ok(rows) = crate::services::project_registry::load_project_config_rows(project_root)
        {
            let entries = rows
                .into_iter()
                .map(|row| {
                    let label = row.label_or_default();
                    (row.name, row.aliases, label)
                })
                .collect::<Vec<_>>();
            if !entries.is_empty() {
                return entries;
            }
        }
    }
    let path = project_root.join("aw.toml");
    let Ok(body) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(value) = body.parse::<toml::Value>() else {
        return Vec::new();
    };
    let Some(entries) = value.get(table).and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|e| {
            let name = e.get("name").and_then(|v| v.as_str())?;
            let label = e.get("label").and_then(|v| v.as_str())?;
            let aliases = e
                .get("aliases")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|a| a.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Some((name.to_string(), aliases, label.to_string()))
        })
        .collect()
}

// Resolve a project name against the `[[projects]]` registry. Returns
// the matching `label` field, or `Err(CreateValidationError::UnknownProject)`
// listing all valid names from `aw.toml`. Accepts either the
// canonical `name` or any value listed under that entry's `aliases`.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
pub(crate) fn resolve_project_label(
    project_root: &Path,
    name: &str,
) -> std::result::Result<String, CreateValidationError> {
    let entries = read_name_aliases_label(project_root, "projects");
    if let Some((_, _, label)) = entries
        .iter()
        .find(|(n, aliases, _)| n == name || aliases.iter().any(|a| a == name))
    {
        return Ok(label.clone());
    }
    let known: Vec<String> = entries.into_iter().map(|(n, _, _)| n).collect();
    Err(CreateValidationError::UnknownProject {
        name: name.to_string(),
        known,
    })
}

fn infer_project_name_from_branch(
    project_root: &Path,
    branch: &str,
) -> std::result::Result<String, CreateValidationError> {
    let entries = read_name_aliases_label(project_root, "projects");
    let known: Vec<String> = entries.iter().map(|(n, _, _)| n.clone()).collect();
    if branch == "main" {
        return Err(CreateValidationError::ProjectRequiredOnMain { known });
    }

    for (name, aliases, _) in &entries {
        let candidates = std::iter::once(name.as_str()).chain(aliases.iter().map(String::as_str));
        for candidate in candidates {
            if branch == candidate
                || branch == format!("project-{}", candidate)
                || branch.starts_with(&format!("{}-wi-", candidate))
                || branch.starts_with(&format!("project-{}-wi-", candidate))
            {
                return Ok(name.clone());
            }
        }
    }

    Err(CreateValidationError::ProjectCannotInfer {
        branch: branch.to_string(),
        known,
    })
}

fn resolve_single_project_name(
    project_root: &Path,
    provided: Option<&str>,
) -> std::result::Result<String, CreateValidationError> {
    if let Some(name) = provided {
        resolve_project_label(project_root, name)?;
        return Ok(name.to_string());
    }
    let branch = crate::branch_switch::current_branch(project_root).map_err(|e| {
        CreateValidationError::ProjectCannotInfer {
            branch: format!("unknown ({})", e),
            known: read_known_project_name_label_pairs(project_root)
                .into_iter()
                .map(|(n, _)| n)
                .collect(),
        }
    })?;
    infer_project_name_from_branch(project_root, &branch)
}

// Resolve an agent name against the `[[agents]]` registry. Returns the
// matching `label` field, or `Err(CreateValidationError::UnknownAgent)`
// listing all valid names. Accepts either the canonical `name` or any
// value listed under that entry's `aliases`.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
pub(crate) fn resolve_agent_label(
    project_root: &Path,
    name: &str,
) -> std::result::Result<String, CreateValidationError> {
    let entries = read_name_aliases_label(project_root, "agents");
    if let Some((_, _, label)) = entries
        .iter()
        .find(|(n, aliases, _)| n == name || aliases.iter().any(|a| a == name))
    {
        return Ok(label.clone());
    }
    let known: Vec<String> = entries.into_iter().map(|(n, _, _)| n).collect();
    Err(CreateValidationError::UnknownAgent {
        name: name.to_string(),
        known,
    })
}

// Parse-time validator errors for `aw wi create` typed flags.
///
// Each variant maps 1:1 to an `error` envelope via `to_envelope_message`
// so mainthread can surface a concise reason without scraping prose.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#state-machine
#[derive(Debug)]
pub(crate) enum CreateValidationError {
    UnknownProject {
        name: String,
        known: Vec<String>,
    },
    UnknownAgent {
        name: String,
        known: Vec<String>,
    },
    /// Non-epic types require exactly 1 `--project`.
    ProjectCardinalityNonEpic {
        issue_type: IssueType,
        observed: usize,
    },
    /// Epic accepts 0 or 1 `--project`.
    ProjectCardinalityEpic {
        observed: usize,
    },
    ProjectRequiredOnMain {
        known: Vec<String>,
    },
    ProjectCannotInfer {
        branch: String,
        known: Vec<String>,
    },
}

// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
impl CreateValidationError {
    pub(crate) fn to_envelope_message(&self) -> String {
        match self {
            Self::UnknownProject { name, known } => format!(
                "unknown --project '{}' (valid: {:?}); update aw.toml [[projects]] or pick from the list",
                name, known
            ),
            Self::UnknownAgent { name, known } => format!(
                "unknown --agent '{}' (valid: {:?}); update aw.toml [[agents]] or pick from the list",
                name, known
            ),
            Self::ProjectCardinalityNonEpic {
                issue_type,
                observed,
            } => format!(
                "--type {:?} requires exactly 1 --project, observed {}",
                issue_type, observed
            ),
            Self::ProjectCardinalityEpic { observed } => format!(
                "--type epic accepts 0 or 1 --project (lead/owner; multi-project spans live in body), observed {}",
                observed
            ),
            Self::ProjectRequiredOnMain { known } => format!(
                "--project is required on branch 'main' (valid: {:?})",
                known
            ),
            Self::ProjectCannotInfer { branch, known } => format!(
                "cannot infer --project from branch '{}' (valid: {:?})",
                branch, known
            ),
        }
    }
}

// Apply the cardinality rule for `--project` based on `--type`.
///
// * `epic` → 0 or 1 value
// * other types → exactly 1 value
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#state-machine
pub(crate) fn check_project_cardinality(
    issue_type: IssueType,
    observed: usize,
) -> std::result::Result<(), CreateValidationError> {
    match (issue_type, observed) {
        (IssueType::Epic, 0) | (IssueType::Epic, 1) => Ok(()),
        (IssueType::Epic, n) => Err(CreateValidationError::ProjectCardinalityEpic { observed: n }),
        (other, 1) => {
            let _ = other;
            Ok(())
        }
        (other, n) => Err(CreateValidationError::ProjectCardinalityNonEpic {
            issue_type: other,
            observed: n,
        }),
    }
}

// Build the canonical label vector for `aw wi create`, in the
// stable order: type, project(s), priority?, agent?.
///
// Pure — does no I/O. Caller resolves names to labels first and passes
// the resolved label strings here.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#logic
pub(crate) fn build_create_label_vec(
    type_label: &str,
    project_labels: &[String],
    priority_label: Option<&str>,
    agent_label: Option<&str>,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    out.push(type_label.to_string());
    for p in project_labels {
        out.push(p.clone());
    }
    if let Some(pr) = priority_label {
        out.push(pr.to_string());
    }
    if let Some(ag) = agent_label {
        out.push(ag.to_string());
    }
    // De-duplicate while preserving first-seen order.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    out.retain(|l| seen.insert(l.clone()));
    out
}

fn is_tracker_routing_label(label: &str) -> bool {
    label.starts_with("app:") || label.starts_with("lib:")
}

// Pure: returns warning messages for `labels` that violate either
///
// 1. the one-issue-one-project count rule (epics excepted), or
// 2. the tracker-label-must-match-`[[projects]]` value rule.
///
// Rule 2 is skipped when `known_labels` is empty — that means config
// declares no managed projects, so there is nothing authoritative to
// validate against.
///
// GitHub has no scoped labels, so we enforce mutual exclusion +
// vocabulary client-side. Best-effort — caller decides how to surface
// the warnings. Edge cases (intentional multi-project non-epic) get
// cleaned up manually.
fn project_label_warnings(
    labels: &[String],
    issue_type: IssueType,
    slug: &str,
    known_labels: &[String],
) -> Vec<String> {
    let routing_labels: Vec<&String> = labels
        .iter()
        .filter(|l| is_tracker_routing_label(l))
        .collect();

    let mut warnings = Vec::new();

    // Rule 1: count.
    match (issue_type, routing_labels.len()) {
        (IssueType::Epic, _) => {} // epics may have any count, including 0
        (_, 1) => {}                // canonical case
        (_, 0) => warnings.push(format!(
            "issue '{}' has no app/lib label (non-epic issues should have exactly 1)",
            slug
        )),
        (_, n) => warnings.push(format!(
            "issue '{}' has {} app/lib labels {:?} (non-epic issues should have exactly 1; only epics may span multiple)",
            slug, n, routing_labels
        )),
    }

    // Rule 2: vocabulary. Each app/lib label must appear in
    // `[[projects]].label`. Applies to epics too — a typo'd project name
    // is still a typo regardless of issue type.
    if !known_labels.is_empty() {
        for label in &routing_labels {
            if !known_labels.iter().any(|k| k == *label) {
                warnings.push(format!(
                    "issue '{}' has tracker label '{}' not declared in [[projects]] in aw.toml (known: {:?})",
                    slug, label, known_labels
                ));
            }
        }
    }

    warnings
}

// Side-effecting wrapper: loads the known-projects vocabulary from
// `aw.toml` and prints any warnings to stderr.
fn check_project_labels(project_root: &Path, labels: &[String], issue_type: IssueType, slug: &str) {
    let known = read_known_project_labels(project_root);
    for msg in project_label_warnings(labels, issue_type, slug, &known) {
        eprintln!("warning: {}", msg);
    }
}

// ---------------------------------------------------------------------------
// Show
// ---------------------------------------------------------------------------

fn issue_show_json(issue: &Issue) -> Result<serde_json::Value> {
    let mut value = serde_json::to_value(issue)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("slug".to_string(), serde_json::json!(issue.slug));
        object.insert("body".to_string(), serde_json::json!(issue.body));
        // Surface the loop state when the WI body carries an `<!-- aw:loop-state -->`
        // block; absent block -> null (not an error). @spec workitem-loop-state-model.
        let loop_state = crate::cli::loop_state::parse_loop_state(&issue.body)
            .and_then(|s| serde_json::to_value(s).ok())
            .unwrap_or(serde_json::Value::Null);
        object.insert("loop_state".to_string(), loop_state);
    }
    Ok(value)
}

async fn run_show(args: ShowArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let (kind, repo, host) = resolve_backend(args.repo.clone(), &project_root)?;
    let backend = make_backend(&kind, &project_root, repo, host)?;

    let issue = backend.get(&args.id).await?;

    match issue {
        Some(issue) => {
            if args.human {
                print_detail(&issue);
            } else if args.pretty {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&issue_show_json(&issue)?)?
                );
            } else {
                println!("{}", serde_json::to_string(&issue_show_json(&issue)?)?);
            }
        }
        None => {
            // @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R7
            if args.human {
                eprintln!("issue '{}' not found in {}", args.id, backend.name());
                std::process::exit(1);
            } else {
                emit_json_error(
                    &format!("issue '{}' not found", args.id),
                    IssueErrorCode::NotFound,
                );
            }
        }
    }
    Ok(())
}

// `aw wi run <id>` -- retired (#1899 R3): `aw goal wi <id>` is the unified
// re-home. This clap leaf still parses (a stale agent gets a structured
// `emit_retired_verb_redirect` envelope instead of a bare clap usage error)
// but never re-enters the run engine.
async fn run_wi_run(args: WiRunArgs) -> Result<()> {
    let replacement = format!("aw goal wi {}", args.id);
    crate::cli::run::emit_retired_verb_redirect(
        "aw wi run",
        "wi",
        &args.id,
        &replacement,
        crate::cli::run::RunPrintOptions {
            human: args.human,
            pretty: args.pretty,
            goal: args.goal,
        },
    )
}

// ---------------------------------------------------------------------------
// CLI envelope (mainthread ↔ subagent ↔ hook loop)
// ---------------------------------------------------------------------------

// Envelope emitted by `aw wi` verbs that drive the author loop.
// See `apps/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md`.
///
// `Dispatch.agent` is optional — when `None`, mainthread runs `invoke.command`
// directly (used for approved → `aw wi merge`); when `Some`, mainthread
// spawns `Agent(subagent_type=agent)` with the envelope embedded in the prompt.
// @spec apps/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R1 #R2 #R12
#[derive(serde::Serialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum IssueEnvelope<'a> {
    Dispatch {
        #[serde(skip_serializing_if = "Option::is_none")]
        agent: Option<&'a str>,
        slug: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        artifact: Option<super::artifact_producer::ArtifactProducerContract>,
        invoke: Invoke<'a>,
    },
    #[allow(dead_code)] // emitted by `aw wi merge` (Phase D)
    Done {
        slug: &'a str,
    },
    Error {
        slug: &'a str,
        message: &'a str,
    },
}

#[derive(serde::Serialize)]
struct Invoke<'a> {
    command: &'a str,
    args: serde_json::Value,
}

fn print_envelope(env: &IssueEnvelope<'_>) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(env)?);
    Ok(())
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

fn maybe_switch_wi_branch_for_project(
    project_root: &Path,
    _project: &str,
    _tmp_id: &str,
) -> Result<String> {
    crate::branch_switch::current_branch(project_root)
}

fn yaml_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn markdown_table_cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn render_draft_issue_markdown(issue: &Issue, project: &str, tmp_id: &str) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("draft: true\n");
    out.push_str(&format!("tmp_id: {}\n", yaml_quote(tmp_id)));
    out.push_str(&format!("project: {}\n", yaml_quote(project)));
    out.push_str(&format!("type: {}\n", issue.issue_type.as_str()));
    out.push_str(&format!("title: {}\n", yaml_quote(&issue.title)));
    out.push_str("state: draft\n");
    out.push_str("draft_phase: created\n");
    if let Some(phase) = &issue.phase {
        out.push_str(&format!("phase: {}\n", yaml_quote(phase)));
    }
    if let Some(created_at) = &issue.created_at {
        out.push_str(&format!("created_at: {}\n", yaml_quote(created_at)));
    }
    if let Some(updated_at) = &issue.updated_at {
        out.push_str(&format!("updated_at: {}\n", yaml_quote(updated_at)));
    }
    if !issue.labels.is_empty() {
        out.push_str("labels:\n");
        for label in &issue.labels {
            out.push_str(&format!("- {}\n", yaml_quote(label)));
        }
    }
    out.push_str("---\n\n");
    out.push_str(issue.body.trim_start());
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn draft_tmp_id(path: &Path, meta: &DraftIssueFrontmatter) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("draft path has no file stem: {}", path.display()))?;
    if let Some(tmp_id) = meta.tmp_id.as_deref() {
        if tmp_id != stem {
            anyhow::bail!(
                "draft tmp_id '{}' does not match filename {}",
                tmp_id,
                path.display()
            );
        }
        Ok(tmp_id.to_string())
    } else {
        Ok(stem.to_string())
    }
}

fn validate_draft_issue(
    project_root: &Path,
    path: &Path,
    issue: &Issue,
    meta: &DraftIssueFrontmatter,
) -> Vec<String> {
    let mut errors = Vec::new();
    if !meta.draft {
        errors.push("frontmatter draft must be true".to_string());
    }
    if let Err(e) = draft_tmp_id(path, meta) {
        errors.push(e.to_string());
    }
    if let Err(e) = resolve_project_label(project_root, &meta.project) {
        errors.push(e.to_envelope_message());
    }
    if !looks_like_structured_attempt(&issue.body) {
        errors.push("body must contain structured work-item sections".to_string());
        return errors;
    }
    errors.extend(validate_publishable_issue_body(issue));
    errors
}

// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-draft-valid-by-construction.md#draft_authoring_contract
fn validate_publishable_issue_body(issue: &Issue) -> Vec<String> {
    let mut errors = Vec::new();
    if !looks_like_structured_attempt(&issue.body) {
        errors.push("body must contain structured work-item sections".to_string());
        return errors;
    }
    for section in [
        crate::issues::IssueSection::Problem,
        crate::issues::IssueSection::Requirements,
        crate::issues::IssueSection::Scope,
        crate::issues::IssueSection::ReferenceContext,
    ] {
        errors.extend(validate_section_format(&issue.body, section));
    }
    errors.extend(validate_planning_alignment(issue));
    if let Err(e) = validate_structured_issue(&issue.body, IssueState::Open) {
        errors.push(e.error);
    }
    errors
}

fn validate_draft_fill(
    project_root: &Path,
    path: &Path,
    issue: &Issue,
    meta: &DraftIssueFrontmatter,
    targets: &[crate::issues::IssueSection],
) -> Vec<String> {
    let mut errors = Vec::new();
    if !meta.draft {
        errors.push("frontmatter draft must be true".to_string());
    }
    if let Err(e) = draft_tmp_id(path, meta) {
        errors.push(e.to_string());
    }
    if let Err(e) = resolve_project_label(project_root, &meta.project) {
        errors.push(e.to_envelope_message());
    }
    for section in targets {
        errors.extend(validate_section_format(&issue.body, *section));
    }
    errors
}

fn write_file_atomically(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content).with_context(|| format!("failed to write {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| {
        format!(
            "failed to move draft {} into {}",
            tmp.display(),
            path.display()
        )
    })?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct DraftIssueFrontmatter {
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    tmp_id: Option<String>,
    project: String,
}

fn read_draft_issue(path: &Path) -> Result<(Issue, DraftIssueFrontmatter)> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read draft {}", path.display()))?;
    let meta_doc = parse_document::<DraftIssueFrontmatter>(&content)
        .with_context(|| format!("failed to parse draft metadata {}", path.display()))?;
    let issue_doc = parse_document::<Issue>(&content)
        .with_context(|| format!("failed to parse draft issue {}", path.display()))?;
    let mut issue = issue_doc.frontmatter;
    issue.body = issue_doc.body;
    issue.slug = issue.default_slug();
    Ok((issue, meta_doc.frontmatter))
}

async fn run_create_from_draft(args: CreateArgs) -> Result<()> {
    let draft_path = args
        .draft_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("draft path is required"))?;
    let project_root = crate::find_project_root()?;
    let (mut issue, draft) = read_draft_issue(draft_path)?;
    if !draft.draft {
        anyhow::bail!("{} is not marked as draft: true", draft_path.display());
    }
    if let Some(tmp_id) = draft.tmp_id.as_deref() {
        let stem = draft_path.file_stem().and_then(|s| s.to_str());
        if stem != Some(tmp_id) {
            anyhow::bail!(
                "draft tmp_id '{}' does not match filename {}",
                tmp_id,
                draft_path.display()
            );
        }
    }

    let project_label = resolve_project_label(&project_root, &draft.project)
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;
    if !issue.labels.iter().any(|label| label == &project_label) {
        issue.labels.push(project_label);
    }
    issue.state = IssueState::Open;
    issue
        .phase
        .get_or_insert_with(|| crate::issues::IssuePhase::Created.as_str().to_string());

    let validation_errors = validate_draft_issue(&project_root, draft_path, &issue, &draft);
    if !validation_errors.is_empty() {
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "error",
                    "path": draft_path,
                    "errors": validation_errors,
                }))?
            );
            std::process::exit(IssueErrorCode::Validation.exit_code());
        }
        anyhow::bail!(
            "draft is not valid for create: {}\n- {}",
            draft_path.display(),
            validation_errors.join("\n- ")
        );
    }

    let (kind, repo, host) = resolve_backend(args.repo.clone(), &project_root)?;
    if kind == "local" {
        anyhow::bail!(
            "aw wi create <draft> requires a tracker issue backend; aw.toml resolved to local"
        );
    }
    let remote = make_backend(&kind, &project_root, repo.clone(), host.clone())
        .context("Failed to create backend")?;
    let created = match remote.create(&issue).await {
        Ok(c) => c,
        Err(e) => {
            if args.json {
                emit_json_error(&e.to_string(), IssueErrorCode::Backend);
            }
            return Err(e);
        }
    };

    let cache = remote_read_cache_backend(&kind, repo.as_deref(), host.as_deref());
    cache.write(&created).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&created)?);
    } else {
        let id_str = created
            .github_id
            .or(created.gitlab_id)
            .map(|n| format!("#{}", n))
            .unwrap_or_default();
        println!("Created {} ({})", created.slug, id_str);
        if let Some(url) = &created.url {
            println!("{}", url);
        }
    }
    Ok(())
}

async fn run_draft_validate(args: DraftValidateArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let (issue, meta) = read_draft_issue(&args.draft_path)?;
    let errors = validate_draft_issue(&project_root, &args.draft_path, &issue, &meta);
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "passed": errors.is_empty(),
                "errors": errors,
                "path": args.draft_path,
            }))?
        );
    } else if errors.is_empty() {
        println!("Draft validation passed: {}", args.draft_path.display());
    } else {
        eprintln!("Draft validation failed: {}", args.draft_path.display());
        for error in &errors {
            eprintln!("  - {}", error);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        std::process::exit(2);
    }
}

async fn run_draft_fill(args: DraftFillArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let (mut issue, meta) = read_draft_issue(&args.draft_path)?;
    let tmp_id = draft_tmp_id(&args.draft_path, &meta)?;
    resolve_project_label(&project_root, &meta.project)
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;

    let payload_body = match (&args.body, &args.body_file) {
        (Some(body), None) => Some(body.clone()),
        (None, Some(path)) => Some(read_body_file(path)?),
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!("clap enforces body/body-file conflict"),
    };

    let Some(payload_body) = payload_body else {
        println!("# score-wi-draft-fill brief");
        println!();
        println!("Draft:   {}", args.draft_path.display());
        println!("Project: {}", meta.project);
        println!("Title:   {}", issue.title);
        println!("Section: {}", args.section);
        println!();
        println!("## Task");
        println!("Write a COMPLETE replacement markdown body for the requested section(s).");
        println!("Then run:");
        println!(
            "  aw wi draft fill {} --section {} --body-file <file>",
            args.draft_path.display(),
            args.section
        );
        return Ok(());
    };

    let is_all = section_arg_is_all(&args.section);
    let targets = if is_all {
        Vec::new()
    } else {
        parse_section_arg(&args.section)?
    };
    let merged_body = if is_all {
        merge_all_sections(&issue.body, &payload_body)
    } else {
        merge_sections(&issue.body, &payload_body, &targets)?
    };
    issue.body = normalize_known_draft_sections(&merged_body);
    let errors = if is_all {
        validate_draft_issue(&project_root, &args.draft_path, &issue, &meta)
    } else {
        validate_draft_fill(&project_root, &args.draft_path, &issue, &meta, &targets)
    };
    if !errors.is_empty() {
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "error",
                    "path": args.draft_path,
                    "errors": errors,
                }))?
            );
        } else {
            eprintln!(
                "Draft fill validation failed: {}",
                args.draft_path.display()
            );
            for error in &errors {
                eprintln!("  - {}", error);
            }
        }
        std::process::exit(2);
    }

    let content = render_draft_issue_markdown(&issue, &meta.project, &tmp_id);
    write_file_atomically(&args.draft_path, &content)?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "draft_filled",
                "project": meta.project,
                "tmp_id": tmp_id,
                "path": args.draft_path,
                "sections": targets.iter().map(|s| s.tag_name()).collect::<Vec<_>>(),
            }))?
        );
    } else {
        println!("Draft filled: {}", args.draft_path.display());
    }
    Ok(())
}

async fn run_draft_init(args: DraftInitArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let project = resolve_single_project_name(&project_root, args.project.as_deref())
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;
    let project_label = resolve_project_label(&project_root, &project)
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;

    let issue_type: IssueType = args.issue_type.into();
    let body = draft_body_from_inputs(&args.title, &args.body, &args.body_file)?;
    let type_label = format!("type:{}", issue_type.as_str());
    let priority_label_owned = args
        .priority
        .map(|p| format!("priority:{}", p.as_label_suffix()));
    let agent_label_owned = match args.agent.as_deref() {
        None => None,
        Some(name) => Some(
            resolve_agent_label(&project_root, name)
                .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?,
        ),
    };
    let labels = build_create_label_vec(
        &type_label,
        &[project_label],
        priority_label_owned.as_deref(),
        agent_label_owned.as_deref(),
    );

    let mut issue = Issue {
        issue_type,
        title: args.title.clone(),
        state: IssueState::Draft,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: String::new(),
        body,
        related: vec![],
        implements: vec![],
        phase: Some(crate::issues::IssuePhase::Created.as_str().to_string()),
        branch: None,
        target_branch: None,
        git_workflow: None,
        change_id: None,
        iteration: None,
        current_task_id: None,
        impl_spec_phase: None,
        task_revisions: None,
        revision_counts: None,
        last_action: None,
        session_id: None,
        validation_errors: vec![],
        review_count: None,
        flagged_sections: None,
        fill_retry_count: None,
        ship_status: None,
        ship_commit: None,
        regen_verified_at: None,
    };
    let slug = issue.default_slug();
    let stamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let tmp_id = format!("wi-{}-{}", stamp, slug);
    maybe_switch_wi_branch_for_project(&project_root, &project, &tmp_id)?;
    issue.slug = tmp_id.clone();

    let draft_dir = crate::shared::workspace::workitems_path(&project_root).join(&project);
    std::fs::create_dir_all(&draft_dir)
        .with_context(|| format!("failed to create {}", draft_dir.display()))?;
    let draft_path = draft_dir.join(format!("{}.md", tmp_id));
    let content = render_draft_issue_markdown(&issue, &project, &tmp_id);
    write_file_atomically(&draft_path, &content)?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "draft_initialized",
                "project": project,
                "tmp_id": tmp_id,
                "path": draft_path,
            }))?
        );
    } else {
        println!("{}", draft_path.display());
    }
    Ok(())
}

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R1
async fn run_create(args: CreateArgs) -> Result<()> {
    run_create_inner(args, true).await
}

/// Internal create service for callers that own stdout's outer protocol
/// envelope (for example explicit-file `aw td create --from-source`).
/// Backend behavior is identical to `aw wi create`; only nested CLI output is
/// suppressed so the caller can emit exactly one canonical JSON envelope.
pub(crate) async fn run_create_silent(args: CreateArgs) -> Result<()> {
    run_create_inner(args, false).await
}

async fn run_create_inner(args: CreateArgs, emit_output: bool) -> Result<()> {
    if args.draft_path.is_some() {
        return run_create_from_draft(args).await;
    }
    let project_root = crate::find_project_root()?;
    let title = args
        .title
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--title is required"))?;
    let issue_type_arg = args
        .issue_type
        .ok_or_else(|| anyhow::anyhow!("--type is required"))?;

    // Resolve body content — inject structured skeleton when no body provided.
    // The skeleton gives the issue-author subagent sections to fill.
    let body = draft_body_from_inputs(title, &args.body, &args.body_file)?;

    let issue_type: IssueType = issue_type_arg.into();

    // Cardinality: epic accepts 0 or 1, others require exactly 1.
    if let Err(e) = check_project_cardinality(issue_type, args.projects.len()) {
        emit_create_envelope_error(title, &e.to_envelope_message());
    }

    // Resolve --project names against [[projects]] in aw.toml.
    let mut project_labels: Vec<String> = Vec::new();
    for name in &args.projects {
        match resolve_project_label(&project_root, name) {
            Ok(label) => project_labels.push(label),
            Err(e) => emit_create_envelope_error(title, &e.to_envelope_message()),
        }
    }

    // Resolve --agent name against [[agents]] in aw.toml.
    let agent_label_owned: Option<String> = match args.agent.as_deref() {
        None => None,
        Some(name) => match resolve_agent_label(&project_root, name) {
            Ok(label) => Some(label),
            Err(e) => emit_create_envelope_error(title, &e.to_envelope_message()),
        },
    };

    let type_label = format!("type:{}", issue_type.as_str());
    let priority_label_owned: Option<String> = args
        .priority
        .map(|p| format!("priority:{}", p.as_label_suffix()));

    let labels = build_create_label_vec(
        &type_label,
        &project_labels,
        priority_label_owned.as_deref(),
        agent_label_owned.as_deref(),
    );

    let issue = Issue {
        issue_type,
        title: title.clone(),
        state: IssueState::Draft,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels,
        created_at: None,
        updated_at: None,
        slug: String::new(),
        body,
        related: vec![],
        implements: vec![],
        // WI authoring is linear: create a bounded skeleton, fill it, then
        // validate. The phase remains for downstream TD lifecycle routing.
        phase: Some(crate::issues::IssuePhase::Created.as_str().to_string()),
        branch: None,
        target_branch: None,
        git_workflow: None,
        change_id: None,
        iteration: None,
        current_task_id: None,
        impl_spec_phase: None,
        task_revisions: None,
        revision_counts: None,
        last_action: None,
        session_id: None,
        validation_errors: vec![],
        review_count: None,
        flagged_sections: None,
        fill_retry_count: None,
        ship_status: None,
        ship_commit: None,
        regen_verified_at: None,
    };

    let validation_errors = validate_publishable_issue_body(&issue);
    if !validation_errors.is_empty() {
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "error",
                    "title": title,
                    "errors": validation_errors,
                }))?
            );
            std::process::exit(IssueErrorCode::Validation.exit_code());
        }
        anyhow::bail!(
            "work-item body is not valid for create: {}\n- {}",
            title,
            validation_errors.join("\n- ")
        );
    }

    let (kind, repo, host) = resolve_backend(args.repo.clone(), &project_root)?;
    let _deprecated_remote_noop = args.remote;

    if create_uses_remote_backend(&kind) {
        // Push directly to the configured tracker, then cache the id/url
        // backfill outside the repository for fast read-through access.
        let remote = make_backend(&kind, &project_root, repo.clone(), host.clone())
            .context("Failed to create remote backend")?;
        let created = match remote.create(&issue).await {
            Ok(c) => c,
            Err(e) => {
                if args.json {
                    emit_json_error(&e.to_string(), IssueErrorCode::Backend);
                }
                return Err(e);
            }
        };

        let cache = remote_read_cache_backend(&kind, repo.as_deref(), host.as_deref());
        cache.write(&created).await?;

        if emit_output {
            if args.json {
                println!("{}", serde_json::to_string_pretty(&created)?);
            } else {
                let id_str = created
                    .github_id
                    .or(created.gitlab_id)
                    .map(|n| format!("#{}", n))
                    .unwrap_or_default();
                println!("Created {} ({})", created.slug, id_str);
                if let Some(url) = &created.url {
                    println!("{}", url);
                }
            }
        }
    } else {
        // In-place local draft.
        let slug = if issue.slug.is_empty() {
            issue.default_slug()
        } else {
            issue.slug.clone()
        };

        // Lock slug before handing to the backend so worktree branch and
        // file name share one source of truth (defends against future
        // drift in default_slug()).
        let mut issue = issue;
        issue.slug = slug.clone();
        let active_path = project_root.clone();

        // Write the issue into the temp issue working copy.
        let backend = LocalBackend::from_project_root(&active_path);
        let created = match backend.create(&issue).await {
            Ok(c) => c,
            Err(e) => {
                if args.json {
                    emit_json_error(&e.to_string(), IssueErrorCode::Backend);
                }
                return Err(e);
            }
        };

        // ---- Write-time structured-issue validation ----
        // @spec structured-issue#R3 R4 R6
        // Validates the temp-hosted issue file. Rollback removes the file
        // from the temp issue store; the branch stays in place so the user can
        // retry `aw wi create` without cleaning up manually.
        if looks_like_structured_attempt(&created.body) {
            if let Err(verr) = validate_structured_issue(&created.body, created.state) {
                let issue_path = backend.issue_path(&created);
                let _ = std::fs::remove_file(&issue_path);
                emit_validation_error(&verr);
            }
        }

        // @spec apps/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R1 #R2 #R3
        // Always emit the canonical JSON envelope on stdout — the
        // `--json` flag is retained above only as a deprecated no-op for
        // callers that still pass it. Mainthread reads this envelope and
        // dispatches the named subagent per CLAUDE.md protocol.
        // Linear fill dispatch: the create envelope kicks off ONE
        // author invocation that fills the full structured body, including
        // capability alignment, scope, and reference context gates. The
        // mainthread runs `--apply --section all`, then runs `validate` once
        // after the full-body merge.
        let payload = fill_section_payload_path(&active_path, &created.slug);
        let payload_initialized =
            initialize_payload_file(&payload, &fill_section_payload_template("all")?)?;
        let issue_path = backend.issue_path(&created);
        let artifact = super::artifact_producer::wi_contract(
            &created.slug,
            &issue_path.to_string_lossy(),
            &payload.to_string_lossy(),
            "all",
            true,
        )?;
        let envelope = IssueEnvelope::Dispatch {
            agent: None,
            slug: &created.slug,
            artifact: Some(artifact),
            invoke: Invoke {
                command: "aw wi fill-section",
                args: serde_json::json!({
                    "slug": created.slug,
                    "sections": ["all"],
                    "payload_path": payload,
                    "payload_initialized": payload_initialized,
                }),
            },
        };
        if emit_output {
            print_envelope(&envelope)?;
        }
    }

    Ok(())
}

// Heuristic: does the body contain any structured-issue marker?
///
// Used to gate write-time validation so that plain free-form issues
// continue to work without forcing the new section discipline.
// @spec structured-issue#R1
fn looks_like_structured_attempt(body: &str) -> bool {
    body.contains("## Problem") || body.contains("## Requirements")
}

// ---------------------------------------------------------------------------
// Fill-section (envelope loop: subagent round-trip via
// /tmp/aw/workspaces/<workspace>/payloads/wi/<slug>/body.md)
// ---------------------------------------------------------------------------

// Derive the issue workspace path for `<slug>` under the active workspace mode.
///
// Payload path where the subagent writes filled body for CLI to merge.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R5
fn fill_section_payload_path(project_root: &std::path::Path, slug: &str) -> std::path::PathBuf {
    crate::shared::workspace::payloads_path(project_root)
        .join("wi")
        .join(slug)
        .join("body.md")
}

fn initialize_payload_file(path: &Path, content: &str) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create payload directory {}", parent.display()))?;
    }
    write_file_atomically(path, content)?;
    Ok(true)
}

fn fill_section_payload_template(section_arg: &str) -> Result<String> {
    if section_arg_is_all(section_arg) {
        return Ok(concat!(
            "## Problem\n\n",
            "(fill)\n\n",
            "## Capability Alignment\n\n",
            "Capability: (fill)\n",
            "Capability Gap: (fill)\n",
            "Progress Evidence: (fill)\n\n",
            "## Requirements\n\n",
            "- R1: (fill)\n\n",
            "## Scope\n\n",
            "### In Scope\n",
            "- (fill)\n\n",
            "### Out of Scope\n",
            "- (fill)\n\n",
            "## Acceptance Criteria\n\n",
            "- AC1: (fill)\n\n",
            "## Reference Context\n\n",
            "### Related Specs\n",
            "| Spec | Relevance |\n",
            "|------|-----------|\n",
            "| (fill) | (fill) |\n\n",
            "### Spec Plan\n",
            "| Spec ID | Action | Main Spec Ref |\n",
            "|---------|--------|---------------|\n",
            "| (fill) | create | (fill) |\n",
        )
        .to_string());
    }

    let sections = parse_section_arg(section_arg)?;
    let mut out = String::new();
    for section in sections {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(fill_section_fragment_template(section));
    }
    Ok(out)
}

fn fill_section_fragment_template(section: crate::issues::IssueSection) -> &'static str {
    use crate::issues::IssueSection;
    match section {
        IssueSection::Problem => "## Problem\n\n(fill)\n",
        IssueSection::Requirements => "## Requirements\n\n- R1: (fill)\n",
        IssueSection::Scope => concat!(
            "## Scope\n\n",
            "### In Scope\n",
            "- (fill)\n\n",
            "### Out of Scope\n",
            "- (fill)\n",
        ),
        IssueSection::ReferenceContext => concat!(
            "## Reference Context\n\n",
            "### Related Specs\n",
            "| Spec | Relevance |\n",
            "|------|-----------|\n",
            "| (fill) | (fill) |\n\n",
            "### Spec Plan\n",
            "| Spec ID | Action | Main Spec Ref |\n",
            "|---------|--------|---------------|\n",
            "| (fill) | create | (fill) |\n",
        ),
    }
}

// @spec apps/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R4 #R5 #R8
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md#scenarios
async fn run_fill_section(args: FillSectionArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = args.slug.clone();
    let worktree_abs = project_root.clone();

    if args.apply {
        run_fill_section_apply(&project_root, &slug, &args.section, &worktree_abs).await
    } else {
        run_fill_section_brief(&slug, &args.section, &worktree_abs).await
    }
}

fn create_uses_remote_backend(kind: &str) -> bool {
    kind != "local"
}

// Brief mode: print a plain-text brief for mainthread to consume directly
// (post-Phase-2 mainthread-only model — no subagent dispatch).
///
// @spec apps/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R4
async fn run_fill_section_brief(
    slug: &str,
    section: &str,
    worktree_abs: &std::path::Path,
) -> Result<()> {
    let backend = LocalBackend::from_project_root(worktree_abs);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;

    let payload = fill_section_payload_path(worktree_abs, slug);
    let payload_created =
        initialize_payload_file(&payload, &fill_section_payload_template(section)?)?;

    println!("# score-issue-author brief");
    println!();
    println!("Issue:    {}  ({})", issue.slug, issue.title);
    println!("Section:  {}", section);
    println!("Checkout: {}", worktree_abs.display());
    println!("Issue file: {}", backend.issue_path(&issue).display());
    println!("Output:   {}", payload.display());
    println!(
        "Payload:  {}",
        if payload_created {
            "initialized"
        } else {
            "existing"
        }
    );
    println!();
    println!("## Task");
    println!();
    match section {
        "all" => {
            println!(
                "Fill every structured section (Problem, Capability Alignment, Requirements, Scope, Acceptance Criteria, Reference Context)."
            );
        }
        other => {
            println!(
                "Fill the `{}` section; leave other sections unchanged.",
                other
            );
        }
    }
    println!();
    println!("## Constraints");
    println!("- English only (see feedback_english_only_specs).");
    println!("- Each Requirements item MUST match `^R\\d+:` (e.g. `- R1: ...`).");
    println!(
        "- Capability Alignment MUST include Capability, Capability Gap, and Progress Evidence."
    );
    println!("- Scope MUST contain both `### In Scope` and `### Out of Scope`.");
    println!("- Acceptance Criteria MUST contain at least one real list item.");
    println!("- Reference Context MUST contain `### Related Specs` and `### Spec Plan` tables.");
    println!();
    println!("## Output contract");
    println!();
    println!("Write the COMPLETE replacement markdown body (no frontmatter) to:");
    println!("  {}", payload.display());
    println!();
    println!("Do NOT run `aw wi update` or `--apply` yourself — the");
    println!(
        "workflow hook/mainthread invokes `aw wi fill-section --slug {} --apply`",
        slug
    );
    println!("after you return.");

    Ok(())
}

// Parse the `--section` arg into a typed list. `"all"` (or empty) expands to
// every section the agent is allowed to write (Problem, Requirements, Scope,
// Reference Context). A comma-separated value like `"requirements,scope"`
// returns `[Requirements, Scope]`. Returns an error on unknown names.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md#scenarios
fn parse_section_arg(s: &str) -> Result<Vec<crate::issues::IssueSection>> {
    use crate::issues::IssueSection;
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
        return Ok(vec![
            IssueSection::Problem,
            IssueSection::Requirements,
            IssueSection::Scope,
            IssueSection::ReferenceContext,
        ]);
    }
    let mut out = Vec::new();
    for part in trimmed.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let sec = IssueSection::parse(p).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown section '{}'; valid: problem, requirements, scope, reference_context",
                p
            )
        })?;
        out.push(sec);
    }
    if out.is_empty() {
        anyhow::bail!("--section was empty after parsing");
    }
    Ok(out)
}

fn section_arg_is_all(s: &str) -> bool {
    let trimmed = s.trim();
    trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all")
}

// Split a markdown body into ordered (heading, content) pairs keyed by the
// H2 line. Content for each heading is everything from the line after the
// heading through the line before the next H2 (or EOF). Lines before the
// first H2 (e.g. `# Title` H1) are returned under the empty-string key so
// callers can re-emit them verbatim at the top.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md#scenarios
fn split_body_by_h2(body: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_heading = String::new();
    let mut current_content = String::new();
    for line in body.split_inclusive('\n') {
        if line.starts_with("## ") {
            sections.push((current_heading.clone(), current_content.clone()));
            current_heading = line.trim_end().to_string();
            current_content.clear();
        } else {
            current_content.push_str(line);
        }
    }
    sections.push((current_heading, current_content));
    sections
}

// Inverse of `split_body_by_h2`: serialize back to a body string. The
// heading line is re-emitted followed by its content (which already
// includes its trailing newlines).
fn join_body_from_sections(sections: &[(String, String)]) -> String {
    let mut out = String::new();
    for (heading, content) in sections {
        if !heading.is_empty() {
            out.push_str(heading);
            out.push('\n');
        }
        out.push_str(content);
        if !content.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

// Section-aware merge: replace ONLY the listed sections in `base_body` with
// the matching sections from `payload_body`. Sections not in `targets` keep
// their content from `base_body` even if `payload_body` provides different
// text — this protects the agent from accidentally regressing earlier
// sections by writing them differently.
///
// Returns an error if any target section is missing from `payload_body`
// (the agent omitted what it was asked to fill — refuse to silently leave
// stale content).
///
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md#scenarios
fn merge_sections(
    base_body: &str,
    payload_body: &str,
    targets: &[crate::issues::IssueSection],
) -> Result<String> {
    let payload_sections = split_body_by_h2(payload_body);
    let base_sections = split_body_by_h2(base_body);

    let payload_map: std::collections::HashMap<String, String> = payload_sections
        .iter()
        .filter(|(h, _)| !h.is_empty())
        .map(|(h, c)| (h.clone(), c.clone()))
        .collect();

    for target in targets {
        let key = target.heading();
        if !payload_map.contains_key(key) {
            anyhow::bail!(
                "payload missing section '{}' (target sections must all be present in the payload)",
                key
            );
        }
    }

    let target_headings: std::collections::HashSet<&'static str> =
        targets.iter().map(|t| t.heading()).collect();

    // Build the merged section list:
    //   1. Walk base sections in order, swapping in payload content for targets.
    //   2. Append any target headings missing from base (e.g. brand-new
    //      sections the skeleton didn't have) in the targets-list order.
    let base_headings: std::collections::HashSet<String> =
        base_sections.iter().map(|(h, _)| h.clone()).collect();

    let mut merged: Vec<(String, String)> = Vec::with_capacity(base_sections.len() + targets.len());
    for (heading, content) in &base_sections {
        if target_headings.contains(heading.as_str()) {
            let new_content = payload_map.get(heading).cloned().unwrap_or_default();
            merged.push((heading.clone(), new_content));
        } else {
            merged.push((heading.clone(), content.clone()));
        }
    }
    for target in targets {
        let key = target.heading();
        if !base_headings.contains(key) {
            let new_content = payload_map.get(key).cloned().unwrap_or_default();
            merged.push((key.to_string(), new_content));
        }
    }

    Ok(join_body_from_sections(&merged))
}

fn merge_all_sections(base_body: &str, payload_body: &str) -> String {
    let payload_sections = split_body_by_h2(payload_body);
    let base_sections = split_body_by_h2(base_body);
    let payload_map: std::collections::HashMap<String, String> = payload_sections
        .iter()
        .map(|(h, c)| (h.clone(), c.clone()))
        .collect();
    let base_order: std::collections::HashSet<String> =
        base_sections.iter().map(|(h, _)| h.clone()).collect();

    let mut merged = Vec::with_capacity(base_sections.len() + payload_sections.len());
    for (heading, content) in &base_sections {
        if let Some(replacement) = payload_map.get(heading) {
            merged.push((heading.clone(), replacement.clone()));
        } else {
            merged.push((heading.clone(), content.clone()));
        }
    }
    for (heading, content) in payload_sections {
        if !base_order.contains(&heading) {
            merged.push((heading, content));
        }
    }
    join_body_from_sections(&merged)
}

fn validate_wi_fill_payload_scope(
    section_arg: &str,
    payload_body: &str,
    targets: &[crate::issues::IssueSection],
) -> Result<()> {
    let parsed = split_body_by_h2(payload_body);
    if parsed
        .first()
        .is_some_and(|(heading, prefix)| heading.is_empty() && !prefix.trim().is_empty())
    {
        anyhow::bail!("payload contains content outside an allowed H2 fill slot");
    }
    let allowed = if section_arg_is_all(section_arg) {
        [
            "## Problem",
            "## Capability Alignment",
            "## Requirements",
            "## Scope",
            "## Acceptance Criteria",
            "## Reference Context",
        ]
        .into_iter()
        .collect::<BTreeSet<_>>()
    } else {
        targets
            .iter()
            .map(|target| target.heading())
            .collect::<BTreeSet<_>>()
    };
    let mut present = BTreeSet::new();
    for (heading, _) in parsed.iter().filter(|(heading, _)| !heading.is_empty()) {
        if !allowed.contains(heading.as_str()) {
            anyhow::bail!(
                "payload heading `{heading}` is outside requested slot `{section_arg}`; allowed headings: {}",
                allowed.iter().copied().collect::<Vec<_>>().join(", ")
            );
        }
        if !present.insert(heading.as_str()) {
            anyhow::bail!("payload contains duplicate fill-slot heading `{heading}`");
        }
    }
    let missing = allowed.difference(&present).copied().collect::<Vec<_>>();
    if !missing.is_empty() {
        anyhow::bail!(
            "payload is missing requested fill-slot heading(s): {}",
            missing.join(", ")
        );
    }
    Ok(())
}

// Apply mode: read the subagent's payload, merge ONLY the requested sections
// into the issue file, delete the payload, and dispatch mainthread to run
// `aw wi validate` next.
///
// Apply does not commit; WI state is projected through the configured issue
// backend. Format checks run before the merge reaches the issue body.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R5 #R8 #R9
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md#scenarios
async fn run_fill_section_apply(
    _project_root: &std::path::Path,
    slug: &str,
    section_arg: &str,
    worktree_abs: &std::path::Path,
) -> Result<()> {
    let payload = fill_section_payload_path(worktree_abs, slug);
    if !payload.exists() {
        print_envelope(&IssueEnvelope::Error {
            slug,
            message: &format!("payload not found: {}", payload.display()),
        })?;
        return Ok(());
    }

    let payload_body = std::fs::read_to_string(&payload)
        .with_context(|| format!("failed to read payload: {}", payload.display()))?;

    let is_all = section_arg_is_all(section_arg);
    let targets = if is_all {
        Vec::new()
    } else {
        match parse_section_arg(section_arg) {
            Ok(t) => t,
            Err(e) => {
                print_envelope(&IssueEnvelope::Error {
                    slug,
                    message: &e.to_string(),
                })?;
                return Ok(());
            }
        }
    };

    let backend = LocalBackend::from_project_root(worktree_abs);
    let existing = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;

    let artifact = super::artifact_producer::wi_contract(
        slug,
        &backend.issue_path(&existing).to_string_lossy(),
        &payload.to_string_lossy(),
        section_arg,
        true,
    )?;
    let payload_error = artifact
        .validate_slot_payload(section_arg, &payload_body)
        .err()
        .or_else(|| {
            validate_wi_fill_payload_scope(section_arg, &payload_body, &targets)
                .err()
                .map(|error| artifact.schema_violation(section_arg, error.to_string()))
        });
    if let Some(error) = payload_error {
        print_envelope(&IssueEnvelope::Error {
            slug,
            message: &error.to_string(),
        })?;
        return Ok(());
    }

    let merged_body = if is_all {
        merge_all_sections(&existing.body, &payload_body)
    } else {
        match merge_sections(&existing.body, &payload_body, &targets) {
            Ok(b) => b,
            Err(e) => {
                print_envelope(&IssueEnvelope::Error {
                    slug,
                    message: &e.to_string(),
                })?;
                return Ok(());
            }
        }
    };

    // R6 + R8: hard-reject malformed section bodies BEFORE writing into
    // the worktree. The check fires on the merged body so structural-type
    // sections (schema, changes, logic, …) without a matching fence or
    // placeholder, and `lang: mermaid` sections without Mermaid Plus
    // frontmatter, all bounce here instead of corrupting the issue file.
    // @spec apps/agentic-workflow/tech-design/core/validate/section-format-rule.md#requirements
    let sf_label = std::path::PathBuf::from(format!("{}.md", slug));
    let sf_findings =
        crate::services::issue_parser::check_issue_body_section_format(&sf_label, &merged_body);
    if !sf_findings.is_empty() {
        let detail = sf_findings
            .iter()
            .map(|f| f.format())
            .collect::<Vec<_>>()
            .join("; ");
        print_envelope(&IssueEnvelope::Error {
            slug,
            message: &format!(
                "section-format check failed ({} finding(s)): {}",
                sf_findings.len(),
                detail,
            ),
        })?;
        return Ok(());
    }

    let patch = IssuePatch {
        body: Some(merged_body),
        ..Default::default()
    };
    backend.update(slug, &patch).await?;

    let _ = std::fs::remove_file(&payload);
    if let Some(parent) = payload.parent() {
        let _ = std::fs::remove_dir(parent);
    }

    print_envelope(&IssueEnvelope::Dispatch {
        agent: None,
        slug,
        artifact: None,
        invoke: Invoke {
            command: "aw wi validate",
            args: serde_json::json!({ "slug": slug }),
        },
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
fn build_update_patch(
    args: &UpdateArgs,
    body: Option<String>,
    current: Option<&Issue>,
) -> Result<IssuePatch> {
    let mut patch = IssuePatch {
        title: args.title.clone(),
        state: args.state.map(Into::into),
        add_labels: args.add_labels.clone(),
        remove_labels: args.remove_labels.clone(),
        body,
        ..Default::default()
    };

    if patch.state == Some(IssueState::Closed) {
        patch.clear_phase = true;
        patch.clear_transient = true;
        patch.ship_status = Some(ShipStatus::Rejected);
        if patch.body.is_none() {
            if let Some(issue) = current {
                patch.body = super::workflow_guard::unlock_projection_for_closed_issue(
                    &issue.body,
                    &issue.slug,
                )?;
            }
        }
    }

    Ok(patch)
}

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
async fn run_update(args: UpdateArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;

    // Resolve body replacement
    let body = if let Some(bf) = &args.body_file {
        Some(read_body_file(bf)?)
    } else {
        None
    };

    // Update locally first when a mirror exists. For remote numeric IDs, the
    // local mirror may be absent; `--push` should still update the configured
    // issue platform and refresh the read-through cache.
    let local = make_backend("local", &project_root, None, None)?;
    let local_current = local.get(&args.id).await?;
    let patch = build_update_patch(&args, body.clone(), local_current.as_ref())?;
    let mut updated_from_remote = false;
    let updated = match local.update(&args.id, &patch).await {
        Ok(u) => u,
        Err(e) => {
            if args.push && e.to_string().contains("not found") {
                let (kind, repo, host) = resolve_backend(args.repo.clone(), &project_root)?;
                if kind != "local" {
                    let remote = make_backend(&kind, &project_root, repo.clone(), host.clone())
                        .context("Failed to create remote backend")?;
                    let remote_current = remote.get(&args.id).await?;
                    let remote_patch =
                        build_update_patch(&args, body.clone(), remote_current.as_ref())?;
                    let updated = match remote.update(&args.id, &remote_patch).await {
                        Ok(issue) => issue,
                        Err(e) => {
                            if args.json {
                                emit_json_error(&e.to_string(), IssueErrorCode::Backend);
                            }
                            return Err(e);
                        }
                    };
                    let cache = remote_read_cache_backend(&kind, repo.as_deref(), host.as_deref());
                    cache.write(&updated).await?;
                    updated_from_remote = true;
                    updated
                } else {
                    if args.json {
                        let msg = e.to_string();
                        emit_json_error(&msg, IssueErrorCode::NotFound);
                    }
                    return Err(e);
                }
            } else {
                if args.json {
                    let msg = e.to_string();
                    if msg.contains("not found") {
                        emit_json_error(&msg, IssueErrorCode::NotFound);
                    } else {
                        emit_json_error(&msg, IssueErrorCode::Backend);
                    }
                }
                return Err(e);
            }
        }
    };

    // Optionally push to remote
    if args.push && !updated_from_remote {
        if let Some(remote_id) = updated.github_id.or(updated.gitlab_id) {
            let remote = make_backend("github", &project_root, args.repo.clone(), None)
                .context("Failed to create remote backend")?;
            let remote_patch = build_update_patch(&args, body.clone(), Some(&updated))?;
            if let Err(e) = remote.update(&remote_id.to_string(), &remote_patch).await {
                if args.json {
                    emit_json_error(&e.to_string(), IssueErrorCode::Backend);
                }
                return Err(e);
            }
        }
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&updated)?);
    } else {
        println!("Updated {}", updated.slug);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Close
// ---------------------------------------------------------------------------

// @spec apps/agentic-workflow/tech-design/semantic/wi-close-remote-rehydration.md#R3
fn remote_close_not_found_message(kind: &str, repo: Option<&str>, id: &str) -> String {
    let repo_context = repo
        .map(|repo| format!(" for repository '{repo}'"))
        .unwrap_or_else(|| " for the configured repository".to_string());
    let repo_arg = repo
        .map(|repo| format!(" --repo {repo}"))
        .unwrap_or_default();
    format!(
        "issue '{id}' not found on {kind} backend{repo_context}; verify the tracker id and repository with `aw wi show {id}{repo_arg}`"
    )
}

// Resolve and close through the configured remote backend. The initial read is
// both the existence check and the state rehydration needed for idempotence:
// an already-closed issue is cached/output as closed without posting the
// reason or issuing a second close mutation.
// @spec apps/agentic-workflow/tech-design/semantic/wi-close-remote-rehydration.md#R1 #R2
async fn close_rehydrated_remote_issue(
    project_root: &Path,
    kind: &str,
    repo: Option<String>,
    host: Option<String>,
    id: &str,
    reason: Option<&str>,
) -> Result<Option<Issue>> {
    let remote = make_backend(kind, project_root, repo.clone(), host.clone())
        .context("Failed to create remote backend")?;
    let Some(mut issue) = remote
        .get(id)
        .await
        .with_context(|| format!("failed to resolve issue '{id}' on {kind} backend"))?
    else {
        return Ok(None);
    };

    if issue.state != IssueState::Closed {
        remote
            .close(id, reason)
            .await
            .with_context(|| format!("failed to close issue '{id}' on {kind} backend"))?;
        issue.state = IssueState::Closed;
    }

    let cache = remote_read_cache_backend(kind, repo.as_deref(), host.as_deref());
    cache
        .write(&issue)
        .await
        .with_context(|| format!("failed to cache closed {kind} issue '{id}'"))?;
    Ok(Some(issue))
}

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R3
// @spec apps/agentic-workflow/tech-design/semantic/wi-close-remote-rehydration.md#R1 #R2 #R3 #R4
async fn run_close(args: CloseArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;

    // Preserve the existing local lifecycle path whenever a mirror is present.
    // A missing numeric mirror is not terminal when --push names a configured
    // tracker: resolve and rehydrate the canonical remote issue instead.
    let local = make_backend("local", &project_root, None, None)?;
    let local_current = match local.get(&args.id).await {
        Ok(issue) => issue,
        Err(e) => {
            if args.json {
                emit_json_error(&e.to_string(), IssueErrorCode::Backend);
            }
            return Err(e);
        }
    };

    let mut remote_handled = false;
    let closed_issue = if local_current.is_some() {
        if let Err(e) = local.close(&args.id, args.reason.as_deref()).await {
            if args.json {
                let msg = e.to_string();
                if msg.contains("not found") {
                    emit_json_error(&msg, IssueErrorCode::NotFound);
                } else {
                    emit_json_error(&msg, IssueErrorCode::Backend);
                }
            }
            return Err(e);
        }
        local.get(&args.id).await?
    } else if args.push && args.id.parse::<u64>().is_ok() {
        let (kind, repo, host) = match resolve_backend(args.repo.clone(), &project_root) {
            Ok(resolved) => resolved,
            Err(e) => {
                if args.json {
                    emit_json_error(&e.to_string(), IssueErrorCode::Backend);
                }
                return Err(e);
            }
        };
        match close_rehydrated_remote_issue(
            &project_root,
            &kind,
            repo.clone(),
            host,
            &args.id,
            args.reason.as_deref(),
        )
        .await
        {
            Ok(Some(issue)) => {
                remote_handled = true;
                Some(issue)
            }
            Ok(None) => {
                let msg = remote_close_not_found_message(&kind, repo.as_deref(), &args.id);
                if args.json {
                    emit_json_error(&msg, IssueErrorCode::NotFound);
                }
                anyhow::bail!(msg);
            }
            Err(e) => {
                if args.json {
                    emit_json_error(&e.to_string(), IssueErrorCode::Backend);
                }
                return Err(e);
            }
        }
    } else {
        let msg = format!("issue '{}' not found", args.id);
        if args.json {
            emit_json_error(&msg, IssueErrorCode::NotFound);
        }
        anyhow::bail!(msg);
    };

    // A real local lifecycle mirror still pushes its platform identity after
    // the local close. Resolve the configured backend (GitHub or GitLab), not a
    // hard-coded GitHub backend, and reuse the same idempotent remote path.
    if args.push && !remote_handled {
        if let Some(ref issue) = closed_issue {
            let (kind, repo, host) = match resolve_backend(args.repo.clone(), &project_root) {
                Ok(resolved) => resolved,
                Err(e) => {
                    if args.json {
                        emit_json_error(&e.to_string(), IssueErrorCode::Backend);
                    }
                    return Err(e);
                }
            };
            let remote_id = match kind.as_str() {
                "github" => issue.github_id,
                "gitlab" => issue.gitlab_id,
                _ => issue.github_id.or(issue.gitlab_id),
            };
            if let Some(remote_id) = remote_id {
                match close_rehydrated_remote_issue(
                    &project_root,
                    &kind,
                    repo.clone(),
                    host,
                    &remote_id.to_string(),
                    args.reason.as_deref(),
                )
                .await
                {
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        let id = remote_id.to_string();
                        let msg = remote_close_not_found_message(&kind, repo.as_deref(), &id);
                        if args.json {
                            emit_json_error(&msg, IssueErrorCode::NotFound);
                        }
                        anyhow::bail!(msg);
                    }
                    Err(e) => {
                        if args.json {
                            emit_json_error(&e.to_string(), IssueErrorCode::Backend);
                        }
                        return Err(e);
                    }
                }
            }
        }
    }

    if args.json {
        if let Some(issue) = &closed_issue {
            println!("{}", serde_json::to_string_pretty(issue)?);
        } else {
            println!("null");
        }
    } else {
        println!("Closed {}", args.id);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Find
// ---------------------------------------------------------------------------

// @spec apps/agentic-workflow/tech-design/core/logic/issues-backend.md#R4
async fn run_find(args: FindArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let (kind, repo, host) = resolve_backend(args.repo.clone(), &project_root)?;
    let backend =
        make_backend(&kind, &project_root, repo, host).context("Failed to create backend")?;

    let issues = match backend.search(&args.query).await {
        Ok(i) => i,
        Err(e) => {
            if args.json {
                emit_json_error(&e.to_string(), IssueErrorCode::Backend);
            }
            return Err(e);
        }
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&issues)?);
    } else {
        if issues.is_empty() {
            println!("No issues matching '{}'", args.query);
        } else {
            print_table(&issues, backend.name());
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Epicize / prioritize
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapabilityRow {
    capability_id: String,
    capability: String,
    capability_type: String,
    surfaces: String,
    ec_dimensions: String,
    current_state: String,
    gaps: String,
    root_wi: String,
    active_wi: String,
    evidence: String,
    claim_id: Option<String>,
    claim_user_story: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapabilityMap {
    capability_count: usize,
    rows: Vec<CapabilityRow>,
    health_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityWiPlanReport {
    pub action: &'static str,
    pub kind: &'static str,
    pub project: String,
    pub backend: String,
    pub path: PathBuf,
    pub cap_path: PathBuf,
    pub capability_count: usize,
    pub planning_row_count: usize,
    pub issue_count: usize,
    pub candidate_count: usize,
    pub reconciliation_count: usize,
    pub resolved_wi_ref_count: usize,
    pub warnings: Vec<String>,
    pub status: String,
    pub requires_hitl: bool,
    pub hitl_status: String,
    pub review_backing: String,
    pub source_digest: String,
    pub review_path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_review_prompt: Option<String>,
    pub published_issue_count: usize,
    pub plan_command: String,
}

const CAPABILITY_PLAN_REVIEW_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
enum CapabilityPlanReviewDecision {
    #[default]
    Pending,
    Accepted,
    NeedsRevision,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct CapabilityPlanReviewChecklist {
    #[serde(default)]
    capability_claim_coverage: bool,
    #[serde(default)]
    bounded_candidates: bool,
    #[serde(default)]
    tracker_reconciliation: bool,
    #[serde(default)]
    verification_specific: bool,
    #[serde(default)]
    no_duplicate_wis: bool,
    #[serde(default)]
    publication_safe: bool,
}

impl CapabilityPlanReviewChecklist {
    fn all_satisfied(&self) -> bool {
        self.capability_claim_coverage
            && self.bounded_candidates
            && self.tracker_reconciliation
            && self.verification_specific
            && self.no_duplicate_wis
            && self.publication_safe
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CapabilityPlanReviewRecord {
    version: u8,
    project: String,
    plan_path: String,
    manifest_path: String,
    source_digest: String,
    #[serde(default)]
    decision: CapabilityPlanReviewDecision,
    #[serde(default)]
    reviewer_kind: String,
    #[serde(default)]
    reviewed_by: String,
    #[serde(default)]
    reviewed_at: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    checklist: CapabilityPlanReviewChecklist,
    #[serde(default)]
    findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CapabilityPlanAuthorRecord {
    version: u8,
    project: String,
    source_digest: String,
    author: String,
    recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CapabilityPlanManifest {
    version: u8,
    project: String,
    cap_path: String,
    candidates: Vec<CapabilityCandidate>,
    reconciliations: Vec<CapabilityTrackerReconciliation>,
}

#[derive(Deserialize, Default)]
struct CapabilityConfig {
    #[serde(default)]
    projects: Vec<CapabilityProjectRow>,
}

#[derive(Deserialize, Default)]
struct CapabilityProjectRow {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    cap_path: Option<String>,
}

pub(crate) async fn load_project_open_issues(
    project_root: &Path,
    project: &str,
    repo: Option<String>,
) -> Result<(String, String, Vec<Issue>)> {
    let (backend_name, project, issues, _) =
        load_project_open_issues_with_backend(project_root, project, repo).await?;
    Ok((backend_name, project, issues))
}

async fn load_project_open_issues_with_backend(
    project_root: &Path,
    project: &str,
    repo: Option<String>,
) -> Result<(String, String, Vec<Issue>, Box<dyn IssueBackend>)> {
    let project_label = resolve_project_label(project_root, project)
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;
    let (kind, repo, host) = resolve_backend(repo, project_root)?;
    let backend =
        make_backend(&kind, project_root, repo, host).context("Failed to create backend")?;
    let filter = IssueFilter {
        state: Some(IssueState::Open),
        issue_type: None,
        label: Some(project_label),
        author: None,
    };
    let mut issues = backend.list(&filter).await?;
    sort_work_items_for_planning(&mut issues);
    Ok((
        backend.name().to_string(),
        project.to_string(),
        issues,
        backend,
    ))
}

async fn run_plan(args: PlanArgs) -> Result<()> {
    let json = args.json;
    let report = build_capability_wi_plan_report(args).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", report.path.display());
    }
    Ok(())
}

pub(crate) async fn build_capability_wi_plan_report(
    args: PlanArgs,
) -> Result<CapabilityWiPlanReport> {
    let project_root = crate::find_project_root()?;
    let project = resolve_single_project_name(&project_root, args.project.as_deref())
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;
    let cap_path = resolve_capability_path(&project_root, &project, args.cap_path.as_deref())?;
    let cap_body = std::fs::read_to_string(&cap_path)
        .with_context(|| format!("failed to read capability map {}", cap_path.display()))?;
    let capability_document = crate::cli::capability::parse_capability_document(
        &cap_body, &cap_path,
    )
    .with_context(|| format!("failed to parse capability map from {}", cap_path.display()))?;
    let td_refs = crate::cli::capability::collect_td_capability_refs(
        &project_root,
        &project,
        &capability_document,
    )
    .unwrap_or_default();
    let capability_report = crate::cli::capability::build_capability_report(
        &project,
        args.cap_path.as_deref(),
        false,
        false,
    )
    .await?;
    let capability_rows = crate::cli::capability::capability_rows_for_wi_plan(
        &capability_document,
        &td_refs,
        &capability_report.capabilities,
    )?;
    let capability_map = CapabilityMap {
        capability_count: capability_document.capabilities.len(),
        rows: capability_rows
            .into_iter()
            .map(|row| CapabilityRow {
                capability_id: row.capability_id,
                capability: row.capability,
                capability_type: row.capability_type,
                surfaces: row.surfaces,
                ec_dimensions: row.ec_dimensions,
                current_state: row.current_state,
                gaps: row.gaps,
                root_wi: row.root_wi,
                active_wi: row.active_wi,
                evidence: row.evidence,
                claim_id: row.claim_id,
                claim_user_story: row.claim_user_story,
            })
            .collect(),
        health_note: extract_project_health_note(&cap_body),
    };
    let (backend_name, project, issues, backend, warnings) =
        match load_project_open_issues_with_backend(&project_root, &project, args.repo.clone())
            .await
        {
            Ok((backend_name, project, issues, backend)) => {
                (backend_name, project, issues, Some(backend), Vec::new())
            }
            Err(err) => (
                "unavailable".to_string(),
                project.clone(),
                Vec::new(),
                None,
                vec![format!("issue inventory unavailable: {err:#}")],
            ),
        };
    let title = args
        .title
        .clone()
        .unwrap_or_else(|| format!("{} capability WI plan", project));
    let candidates = capability_wi_candidates(&capability_map.rows, &issues);
    let resolved_wi_refs = match backend.as_deref() {
        Some(backend) => {
            resolve_capability_tracker_ref_lookups(&capability_map.rows, &issues, backend).await
        }
        None => BTreeMap::new(),
    };
    let reconciliations =
        capability_tracker_reconciliations(&capability_map.rows, &issues, &resolved_wi_refs);
    let review_backing = capability_plan_review_backing(&project_root, &project);
    let body = render_capability_wi_plan(
        &project,
        &title,
        &backend_name,
        &cap_path,
        &capability_map,
        &issues,
        &candidates,
        &resolved_wi_refs,
        &warnings,
        &review_backing,
    );
    let path = write_planning_artifact(
        &project_root,
        &project,
        "capability-plan",
        &title,
        args.output.as_deref(),
        &body,
    )?;

    let manifest = CapabilityPlanManifest {
        version: CAPABILITY_PLAN_REVIEW_VERSION,
        project: project.clone(),
        cap_path: cap_path.display().to_string(),
        candidates: candidates.clone(),
        reconciliations: reconciliations.clone(),
    };
    let manifest_path = capability_plan_sidecar_path(&path, "manifest.json");
    let manifest_body = format!("{}\n", serde_json::to_string_pretty(&manifest)?);
    write_file_atomically(&manifest_path, &manifest_body)?;
    let source_digest = capability_plan_source_digest(&body, &manifest_body);
    let review_path = capability_plan_sidecar_path(&path, "review.json");
    let author_path = capability_plan_sidecar_path(&path, "author.json");
    let author_record = CapabilityPlanAuthorRecord {
        version: CAPABILITY_PLAN_REVIEW_VERSION,
        project: project.clone(),
        source_digest: source_digest.clone(),
        author: current_plan_actor_identity(),
        recorded_at: chrono::Utc::now().to_rfc3339(),
    };
    write_file_atomically(
        &author_path,
        &format!("{}\n", serde_json::to_string_pretty(&author_record)?),
    )?;
    let payload_path = capability_plan_review_payload_path(&project_root, &project, &source_digest);
    if let Some(parent) = payload_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let pending_record = CapabilityPlanReviewRecord {
        version: CAPABILITY_PLAN_REVIEW_VERSION,
        project: project.clone(),
        plan_path: path.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        source_digest: source_digest.clone(),
        decision: CapabilityPlanReviewDecision::Pending,
        reviewer_kind: if review_backing == "human" {
            "human".to_string()
        } else {
            "agent".to_string()
        },
        reviewed_by: String::new(),
        reviewed_at: String::new(),
        summary: String::new(),
        checklist: CapabilityPlanReviewChecklist::default(),
        findings: Vec::new(),
    };
    write_file_atomically(
        &payload_path,
        &format!("{}\n", serde_json::to_string_pretty(&pending_record)?),
    )?;

    let plan_command = capability_wi_plan_command(&project, args.cap_path.as_deref());
    let review_command = capability_plan_review_command(&payload_path);
    let requires_hitl = review_backing == "human";
    Ok(CapabilityWiPlanReport {
        action: "planned",
        kind: "capability_plan",
        project: project.clone(),
        backend: backend_name,
        path: path.clone(),
        cap_path,
        capability_count: capability_map.capability_count,
        planning_row_count: capability_map.rows.len(),
        issue_count: issues.len(),
        candidate_count: candidates.len(),
        reconciliation_count: reconciliations.len(),
        resolved_wi_ref_count: resolved_wi_refs.len(),
        warnings,
        status: if requires_hitl {
            "pending_human_review".to_string()
        } else {
            "pending_agent_review".to_string()
        },
        requires_hitl,
        hitl_status: if requires_hitl {
            "pending_human".to_string()
        } else {
            "pending_agent_review".to_string()
        },
        review_backing: review_backing.clone(),
        source_digest: source_digest.clone(),
        review_path,
        payload_path: Some(payload_path.clone()),
        next: Some(review_command),
        agent_review_prompt: (!requires_hitl).then(|| {
            capability_plan_agent_review_prompt(&project, &path, &payload_path, &source_digest)
        }),
        published_issue_count: 0,
        plan_command,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingCapabilityPlanReview {
    pub command: String,
    pub payload_path: PathBuf,
    pub prompt: String,
    pub requires_hitl: bool,
}

pub(crate) fn pending_capability_plan_review(
    project_root: &Path,
    project: &str,
) -> Option<PendingCapabilityPlanReview> {
    let dir = crate::shared::workspace::workitems_path(project_root)
        .join(project)
        .join("capability-plan");
    let mut plans = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .collect::<Vec<_>>();
    plans.sort();
    for plan_path in plans.into_iter().rev() {
        let manifest_path = capability_plan_sidecar_path(&plan_path, "manifest.json");
        if !manifest_path.is_file() {
            continue;
        }
        let plan_body = std::fs::read_to_string(&plan_path).ok()?;
        let manifest_body = std::fs::read_to_string(&manifest_path).ok()?;
        let source_digest = capability_plan_source_digest(&plan_body, &manifest_body);
        let review_path = capability_plan_sidecar_path(&plan_path, "review.json");
        if let Ok(review_body) = std::fs::read_to_string(&review_path) {
            if serde_json::from_str::<CapabilityPlanReviewRecord>(&review_body)
                .ok()
                .is_some_and(|record| {
                    record.source_digest == source_digest
                        && record.decision == CapabilityPlanReviewDecision::Accepted
                })
            {
                return None;
            }
        }
        let payload_path =
            capability_plan_review_payload_path(project_root, project, &source_digest);
        if !payload_path.is_file() {
            return None;
        }
        let review_backing = capability_plan_review_backing(project_root, project);
        let requires_hitl = review_backing == "human";
        return Some(PendingCapabilityPlanReview {
            command: capability_plan_review_command(&payload_path),
            payload_path: payload_path.clone(),
            prompt: if requires_hitl {
                format!(
                    "Capability plan `{}` requires explicit human review under capability_plan_review_backing=human. Complete `{}` and run the review command.",
                    plan_path.display(),
                    payload_path.display()
                )
            } else {
                capability_plan_agent_review_prompt(
                    project,
                    &plan_path,
                    &payload_path,
                    &source_digest,
                )
            },
            requires_hitl,
        });
    }
    None
}

async fn run_plan_review(args: PlanReviewArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let evidence_body = std::fs::read_to_string(&args.evidence_file)
        .with_context(|| format!("failed to read {}", args.evidence_file.display()))?;
    let mut record: CapabilityPlanReviewRecord = serde_json::from_str(&evidence_body)
        .with_context(|| {
            format!(
                "invalid capability-plan review payload {}",
                args.evidence_file.display()
            )
        })?;
    validate_capability_plan_review_record(&project_root, &record)?;
    let plan_path = PathBuf::from(&record.plan_path);
    let review_path = capability_plan_sidecar_path(&plan_path, "review.json");
    record.reviewed_at = chrono::Utc::now().to_rfc3339();

    let (status, next, published_issue_count) = match record.decision {
        CapabilityPlanReviewDecision::Accepted => {
            let manifest_body = std::fs::read_to_string(&record.manifest_path)?;
            let manifest: CapabilityPlanManifest = serde_json::from_str(&manifest_body)?;
            let published = publish_capability_plan_candidates(&record.project, &manifest).await?;
            (
                "accepted",
                format!(
                    "aw goal capability --project {} --non-interactive",
                    record.project
                ),
                published,
            )
        }
        CapabilityPlanReviewDecision::NeedsRevision => (
            "needs_revision",
            format!("aw wi plan --project {}", record.project),
            0,
        ),
        CapabilityPlanReviewDecision::Pending => unreachable!("pending rejected by validation"),
    };
    // Persist acceptance only after every candidate has been published (or
    // independently deduplicated). A transient backend failure must leave the
    // plan pending so the exact same evidence can be retried safely.
    write_file_atomically(
        &review_path,
        &format!("{}\n", serde_json::to_string_pretty(&record)?),
    )?;
    let _ = std::fs::remove_file(&args.evidence_file);
    let output = serde_json::json!({
        "schema_version": "aw.cli.v1",
        "action": "capability_plan_review",
        "project": record.project,
        "status": status,
        "clean": status == "accepted",
        "requires_hitl": false,
        "source_digest": record.source_digest,
        "review_path": review_path,
        "backing": record.reviewer_kind,
        "findings": record.findings,
        "published_issue_count": published_issue_count,
        "next": { "kind": "run_command", "command": next },
    });
    if args.json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

fn validate_capability_plan_review_record(
    project_root: &Path,
    record: &CapabilityPlanReviewRecord,
) -> Result<()> {
    if record.version != CAPABILITY_PLAN_REVIEW_VERSION {
        anyhow::bail!(
            "capability-plan review version {} is unsupported; expected {}",
            record.version,
            CAPABILITY_PLAN_REVIEW_VERSION
        );
    }
    if record.project.trim().is_empty() || record.reviewed_by.trim().is_empty() {
        anyhow::bail!("capability-plan review requires project and reviewed_by identity");
    }
    if record.summary.trim().is_empty() {
        anyhow::bail!("capability-plan review requires a semantic review summary");
    }
    let plan_path = PathBuf::from(&record.plan_path);
    let manifest_path = PathBuf::from(&record.manifest_path);
    let expected_plan_root = crate::shared::workspace::workitems_path(project_root)
        .join(&record.project)
        .join("capability-plan");
    if !plan_path.starts_with(&expected_plan_root)
        || manifest_path != capability_plan_sidecar_path(&plan_path, "manifest.json")
    {
        anyhow::bail!(
            "capability-plan review paths must name a plan and its manifest under {}",
            expected_plan_root.display()
        );
    }
    resolve_project_label(project_root, &record.project)
        .map_err(|error| anyhow::anyhow!(error.to_envelope_message()))?;
    let plan_body = std::fs::read_to_string(&plan_path)
        .with_context(|| format!("read reviewed plan {}", plan_path.display()))?;
    let manifest_body = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("read reviewed manifest {}", manifest_path.display()))?;
    let actual_digest = capability_plan_source_digest(&plan_body, &manifest_body);
    if actual_digest != record.source_digest {
        anyhow::bail!(
            "capability-plan review evidence is stale; rerun `aw wi plan --project {}`",
            record.project
        );
    }
    let manifest: CapabilityPlanManifest = serde_json::from_str(&manifest_body)?;
    if manifest.project != record.project {
        anyhow::bail!("capability-plan review project does not match its manifest");
    }
    let backing = capability_plan_review_backing(project_root, &record.project);
    match record.reviewer_kind.as_str() {
        "human" => {}
        "agent" if backing != "human" => {
            let author_path = capability_plan_sidecar_path(&plan_path, "author.json");
            let author_body = std::fs::read_to_string(&author_path).with_context(|| {
                format!(
                    "agent capability-plan review requires author evidence {}",
                    author_path.display()
                )
            })?;
            let author: CapabilityPlanAuthorRecord = serde_json::from_str(&author_body)?;
            if author.source_digest != record.source_digest {
                anyhow::bail!("capability-plan author evidence is stale for the reviewed digest");
            }
            if author
                .author
                .trim()
                .eq_ignore_ascii_case(record.reviewed_by.trim())
            {
                anyhow::bail!(
                    "agent capability-plan review is not independent: reviewed_by matches the recorded plan author"
                );
            }
        }
        "agent" => anyhow::bail!(
            "project `{}` capability_plan_review_backing policy is human-only",
            record.project
        ),
        other => anyhow::bail!(
            "capability-plan reviewer_kind `{other}` is unsupported; expected human or agent"
        ),
    }
    match record.decision {
        CapabilityPlanReviewDecision::Accepted => {
            if !record.findings.is_empty() {
                anyhow::bail!("accepted capability-plan review must not contain findings");
            }
            if !record.checklist.all_satisfied() {
                anyhow::bail!("accepted capability-plan review requires every checklist item");
            }
        }
        CapabilityPlanReviewDecision::NeedsRevision => {
            if record.findings.is_empty() {
                anyhow::bail!("needs_revision capability-plan review requires findings");
            }
        }
        CapabilityPlanReviewDecision::Pending => {
            anyhow::bail!("capability-plan review decision is still pending")
        }
    }
    Ok(())
}

async fn publish_capability_plan_candidates(
    project: &str,
    manifest: &CapabilityPlanManifest,
) -> Result<usize> {
    let project_root = crate::find_project_root()?;
    let (_, _, mut open_issues, _) =
        load_project_open_issues_with_backend(&project_root, project, None).await?;
    let cap_path = PathBuf::from(&manifest.cap_path);
    let mut published = 0usize;
    for candidate in &manifest.candidates {
        if open_issues
            .iter()
            .any(|issue| open_issue_serves_capability_candidate(issue, candidate))
        {
            continue;
        }
        let issue_type = match candidate.issue_type.as_str() {
            "bug" => TypeFilter::Bug,
            "refactor" => TypeFilter::Refactor,
            "test" => TypeFilter::Test,
            _ => TypeFilter::Enhancement,
        };
        run_create_silent(CreateArgs {
            draft_path: None,
            title: Some(candidate.title.clone()),
            issue_type: Some(issue_type),
            body: Some(capability_candidate_wi_body(project, &cap_path, candidate)),
            body_file: None,
            projects: vec![project.to_string()],
            priority: Some(PriorityFilter::P1),
            agent: None,
            remote: false,
            json: false,
            repo: None,
        })
        .await?;
        published += 1;
        open_issues.push(Issue {
            issue_type: issue_type.into(),
            title: candidate.title.clone(),
            state: IssueState::Open,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: Vec::new(),
            created_at: None,
            updated_at: None,
            slug: candidate.title.clone(),
            body: String::new(),
            related: Vec::new(),
            implements: Vec::new(),
            phase: None,
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        });
    }
    Ok(published)
}

fn open_issue_serves_capability_candidate(issue: &Issue, candidate: &CapabilityCandidate) -> bool {
    if issue
        .title
        .trim()
        .eq_ignore_ascii_case(candidate.title.trim())
    {
        return true;
    }
    if issue.issue_type == IssueType::Epic {
        return false;
    }
    let declared = crate::cli::capability::wi_body_capability_alignment_ids(&issue.body);
    match candidate.claim_id.as_ref() {
        Some(claim_id) => declared.contains(claim_id),
        None => declared.contains(&candidate.source_capability_id),
    }
}

fn capability_plan_review_backing(project_root: &Path, project: &str) -> String {
    let configured =
        crate::services::project_registry::resolve_project_config_row(project_root, project)
            .ok()
            .and_then(|row| {
                let local = project_root.join(row.path).join("aw.toml");
                let value = std::fs::read_to_string(local)
                    .ok()?
                    .parse::<toml::Value>()
                    .ok()?;
                value
                    .get("capability_plan_review_backing")
                    .and_then(toml::Value::as_str)
                    .map(str::to_string)
            })
            .or_else(|| {
                let value = std::fs::read_to_string(project_root.join("aw.toml"))
                    .ok()?
                    .parse::<toml::Value>()
                    .ok()?;
                value
                    .get("projects")?
                    .as_array()?
                    .iter()
                    .find(|row| row.get("name").and_then(toml::Value::as_str) == Some(project))?
                    .get("capability_plan_review_backing")
                    .and_then(toml::Value::as_str)
                    .map(str::to_string)
            });
    match configured.as_deref().map(str::trim) {
        Some("human") => "human".to_string(),
        Some("agent") => "agent".to_string(),
        _ => "either".to_string(),
    }
}

fn capability_plan_source_digest(plan_body: &str, manifest_body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plan_body.as_bytes());
    hasher.update([0]);
    hasher.update(manifest_body.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn capability_plan_sidecar_path(plan_path: &Path, extension: &str) -> PathBuf {
    plan_path.with_extension(extension)
}

fn capability_plan_review_payload_path(
    project_root: &Path,
    project: &str,
    source_digest: &str,
) -> PathBuf {
    crate::shared::workspace::payloads_path(project_root)
        .join("capability-plan")
        .join(project)
        .join(source_digest)
        .join("review.json")
}

fn capability_plan_review_command(payload_path: &Path) -> String {
    format!(
        "aw wi plan-review --evidence-file {}",
        shell_quote(&payload_path.display().to_string())
    )
}

fn capability_plan_agent_review_prompt(
    project: &str,
    plan_path: &Path,
    payload_path: &Path,
    source_digest: &str,
) -> String {
    format!(
        "Independently review capability WI plan `{}` for project `{project}` at digest `{source_digest}`. Check capability_claim_coverage, bounded_candidates, tracker_reconciliation, verification_specific, no_duplicate_wis, and publication_safe. Fill `{}` with reviewer_kind=agent, an identity independent from the recorded author, decision=accepted with every checklist value true and no findings, or decision=needs_revision with concrete findings. Then run `{}`.",
        plan_path.display(),
        payload_path.display(),
        capability_plan_review_command(payload_path)
    )
}

fn current_plan_actor_identity() -> String {
    std::env::var("AW_AGENT_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("AGENT_ID")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "unknown-actor".to_string())
}

fn capability_wi_plan_command(project: &str, cap_path_override: Option<&Path>) -> String {
    let mut command = format!("aw wi plan --project {project}");
    if let Some(path) = cap_path_override {
        command.push_str(" --cap-path ");
        command.push_str(&shell_quote(&path.display().to_string()));
    }
    command
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

async fn run_epicize(args: EpicizeArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let project = resolve_single_project_name(&project_root, args.project.as_deref())
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;
    let (backend_name, project, issues) =
        load_project_open_issues(&project_root, &project, args.repo.clone()).await?;
    let capability_document = load_markdown_capability_document(&project_root, &project);
    let title = args
        .title
        .clone()
        .unwrap_or_else(|| format!("{} next phase", project));
    let body = render_epicize_plan(
        &project,
        &title,
        &backend_name,
        &issues,
        capability_document.as_ref(),
    );
    let path = write_planning_artifact(
        &project_root,
        &project,
        "epics",
        &title,
        args.output.as_deref(),
        &body,
    )?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "epicized",
                "project": project,
                "backend": backend_name,
                "path": path,
                "issue_count": issues.len(),
                "capability_count": capability_document
                    .as_ref()
                    .map(|document| document.capabilities.len())
                    .unwrap_or(0),
                "title": title,
                "requires_hitl": true,
                "hitl_status": "pending",
            }))?
        );
    } else {
        println!("{}", path.display());
    }
    Ok(())
}

fn load_markdown_capability_document(
    project_root: &Path,
    project: &str,
) -> Option<crate::cli::capability::CapabilityDocument> {
    let cap_path = resolve_capability_path(project_root, project, None).ok()?;
    let body = std::fs::read_to_string(&cap_path).ok()?;
    let document = crate::cli::capability::parse_capability_document(&body, &cap_path).ok()?;
    if document.requires_format_migration() {
        None
    } else {
        Some(document)
    }
}

async fn run_atomize(args: AtomizeArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let project = resolve_single_project_name(&project_root, args.project.as_deref())
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;
    let (backend_name, project, issues) =
        load_project_open_issues(&project_root, &project, args.repo.clone()).await?;
    let title = args
        .title
        .clone()
        .unwrap_or_else(|| format!("{} atomization", project));
    let candidates = atomize_candidates(&issues);
    let body = render_atomize_plan(&project, &title, &backend_name, &issues, &candidates);
    let path = write_planning_artifact(
        &project_root,
        &project,
        "atomize",
        &title,
        args.output.as_deref(),
        &body,
    )?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "atomized",
                "project": project,
                "backend": backend_name,
                "path": path,
                "issue_count": issues.len(),
                "candidate_count": candidates.len(),
                "requires_hitl": true,
                "hitl_status": "pending",
            }))?
        );
    } else {
        println!("{}", path.display());
    }
    Ok(())
}

async fn run_prioritize(args: PrioritizeArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let project = resolve_single_project_name(&project_root, args.project.as_deref())
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;
    let (backend_name, project, issues) =
        load_project_open_issues(&project_root, &project, args.repo.clone()).await?;
    let lanes = prioritize_lanes(&issues);
    let epic_count = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::Epic)
        .count();
    let title = args
        .title
        .clone()
        .unwrap_or_else(|| format!("{} priority plan", project));
    let body = render_prioritize_plan(&project, &title, &backend_name, &lanes, &issues);
    let path = write_planning_artifact(
        &project_root,
        &project,
        "priorities",
        &title,
        args.output.as_deref(),
        &body,
    )?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "prioritized",
                "project": project,
                "backend": backend_name,
                "path": path,
                "ready_now": issue_refs_json(&lanes.ready_now),
                "blocked_by_dependency": issue_refs_json(&lanes.blocked_by_dependency),
                "needs_atomize": issue_refs_json(&lanes.needs_atomize),
                "needs_triage": issue_refs_json(&lanes.needs_triage),
                "deferred": issue_refs_json(&lanes.deferred),
                "ready_now_count": lanes.ready_now.len(),
                "blocked_by_dependency_count": lanes.blocked_by_dependency.len(),
                "needs_atomize_count": lanes.needs_atomize.len(),
                "needs_triage_count": lanes.needs_triage.len(),
                "deferred_count": lanes.deferred.len(),
                "epic_count": epic_count,
                "issue_count": issues.len(),
                "requires_hitl": true,
                "hitl_status": "pending",
            }))?
        );
    } else {
        println!("{}", path.display());
    }
    Ok(())
}

fn sort_work_items_for_planning(issues: &mut [Issue]) {
    issues.sort_by(|a, b| {
        (
            priority_rank(a),
            type_rank(a.issue_type),
            a.github_id.or(a.gitlab_id).unwrap_or(u64::MAX),
            a.title.to_ascii_lowercase(),
        )
            .cmp(&(
                priority_rank(b),
                type_rank(b.issue_type),
                b.github_id.or(b.gitlab_id).unwrap_or(u64::MAX),
                b.title.to_ascii_lowercase(),
            ))
    });
}

// #1899 R7: `pub(crate)` so `aw goal backlog`'s priority-first WI ordering
// (`crate::cli::run::list_open_project_issues`) shares the exact same rank
// function `aw wi prioritize`/`aw wi plan` use, instead of a second copy
// that could silently drift.
pub(crate) fn priority_rank(issue: &Issue) -> u8 {
    for label in &issue.labels {
        match label.as_str() {
            "priority:p0" => return 0,
            "priority:p1" => return 1,
            "priority:p2" => return 2,
            "priority:p3" => return 3,
            _ => {}
        }
    }
    4
}

fn priority_label(issue: &Issue) -> &'static str {
    match priority_rank(issue) {
        0 => "p0",
        1 => "p1",
        2 => "p2",
        3 => "p3",
        _ => "none",
    }
}

fn type_rank(issue_type: IssueType) -> u8 {
    match issue_type {
        IssueType::Bug => 0,
        IssueType::Enhancement => 1,
        IssueType::Refactor => 2,
        IssueType::Test => 3,
        IssueType::Epic => 4,
    }
}

fn body_field_value(body: &str, key: &str) -> Option<String> {
    let key_lower = key.to_ascii_lowercase();
    for line in body.lines() {
        let trimmed = line
            .trim()
            .trim_start_matches("- ")
            .trim_start_matches("* ")
            .trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with(&(key_lower.clone() + ":")) {
            let (_, value) = trimmed.split_once(':')?;
            return Some(value.trim().trim_matches('`').to_string());
        }
    }
    None
}

fn section_content(body: &str, heading: &str) -> Option<String> {
    split_body_by_h2(body)
        .into_iter()
        .find(|(h, _)| h == heading)
        .map(|(_, c)| c)
}

fn has_real_value(body: &str, key: &str) -> bool {
    body_field_value(body, key)
        .map(|v| is_real_planning_value(&v))
        .unwrap_or(false)
}

fn is_real_planning_value(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    !matches!(
        lower.as_str(),
        "(fill)" | "(replace-this)" | "tbd" | "todo" | "maybe" | "unclear" | "uncertain"
    )
}

fn section_has_real_list_item(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
            return false;
        }
        is_real_planning_value(trimmed.trim_start_matches("- ").trim_start_matches("* "))
    })
}

fn validate_planning_alignment(issue: &Issue) -> Vec<String> {
    if issue.issue_type == IssueType::Epic {
        return Vec::new();
    }

    let mut errors = Vec::new();
    if looks_too_large_for_atomic_wi(issue) {
        errors.push(
            "too-large: non-epic work-item appears roadmap-sized; run `aw wi atomize` or create `--type epic` first".to_string(),
        );
    }

    match section_content(&issue.body, "## Capability Alignment") {
        Some(content) => {
            for field in ["Capability", "Capability Gap", "Progress Evidence"] {
                if !has_real_value(&content, field) {
                    errors.push(format!(
                        "alignment: ## Capability Alignment missing real `{}` value",
                        field
                    ));
                }
            }
        }
        None => errors.push(
            "alignment: missing ## Capability Alignment section with Capability, Capability Gap, and Progress Evidence".to_string(),
        ),
    }

    match section_content(&issue.body, "## Acceptance Criteria") {
        Some(content) if section_has_real_list_item(&content) => {}
        Some(_) => errors.push(
            "not-testable: ## Acceptance Criteria must contain at least one real list item"
                .to_string(),
        ),
        None => errors.push("not-testable: missing ## Acceptance Criteria section".to_string()),
    }

    errors
}

// Multi-word phrases that are unambiguous roadmap-scale signals on their own
// (no surrounding-noun context needed): they either name a whole product
// ("google maps"), or already pair a scale word with a big-scope noun/verb
// so a raw substring match cannot collide with a hyphenated technical term.
const TOO_LARGE_HARD_PHRASES: &[&str] = &[
    "google map",
    "google maps",
    "full platform",
    "complete platform",
    "from scratch",
    "end-to-end product",
    "rewrite all",
    "rewrite everything",
    "all projects",
    "every project",
    "every crate",
    "across the fleet",
];

// Bare scale words that only signal roadmap scope when they sit next to a
// big-scope noun ("the whole platform", "rewrite the entire codebase").
// Matched against standalone word tokens, so hyphenated compounds like
// "whole-doc" or "always-send-everything" never trigger them.
const TOO_LARGE_CONTEXT_SCALE_WORDS: &[&str] = &["entire", "whole", "everything"];

const TOO_LARGE_SCOPE_NOUNS: &[&str] = &[
    "project",
    "projects",
    "codebase",
    "codebases",
    "repo",
    "repos",
    "repository",
    "repositories",
    "platform",
    "platforms",
    "system",
    "systems",
    "product",
    "products",
    "application",
    "applications",
    "app",
    "apps",
    "service",
    "services",
    "monorepo",
    "monorepos",
    "ecosystem",
    "ecosystems",
    "organization",
    "organizations",
    "org",
    "orgs",
    "roadmap",
    "roadmaps",
    "stack",
    "stacks",
    "suite",
    "suites",
    "fleet",
    "fleets",
    "company",
    "companies",
    "business",
    "businesses",
];

/// Split text into standalone word tokens, keeping internal hyphens intact
/// so a compound like "whole-doc" or "always-send-everything" stays one
/// token distinct from the bare word "whole"/"everything".
fn too_large_word_tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| c.is_whitespace())
        .map(|raw| raw.trim_matches(|c: char| !c.is_alphanumeric() && c != '-'))
        .filter(|w| !w.is_empty())
        .collect()
}

/// True when the scale word at `idx` has a big-scope noun within a small
/// window around it, e.g. "the whole platform" or "rewrite the entire
/// codebase" -- but not "own the whole row".
fn too_large_scale_word_in_scope_context(words: &[&str], idx: usize) -> bool {
    let window_start = idx.saturating_sub(2);
    let window_end = (idx + 4).min(words.len());
    words[window_start..window_end]
        .iter()
        .any(|word| TOO_LARGE_SCOPE_NOUNS.contains(word))
}

fn looks_too_large_for_atomic_wi(issue: &Issue) -> bool {
    let text = format!("{}\n{}", issue.title, issue.body).to_ascii_lowercase();

    if TOO_LARGE_HARD_PHRASES
        .iter()
        .any(|phrase| text.contains(phrase))
    {
        return true;
    }

    let words = too_large_word_tokens(&text);
    words.iter().enumerate().any(|(idx, word)| {
        TOO_LARGE_CONTEXT_SCALE_WORDS.contains(word)
            && too_large_scale_word_in_scope_context(&words, idx)
    })
}

#[derive(Debug, Clone, Default)]
/// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub(crate) struct PrioritizeLanes {
    pub(crate) ready_now: Vec<Issue>,
    pub(crate) blocked_by_dependency: Vec<Issue>,
    pub(crate) needs_atomize: Vec<Issue>,
    pub(crate) needs_triage: Vec<Issue>,
    pub(crate) deferred: Vec<Issue>,
}

/// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub(crate) fn prioritize_lanes(issues: &[Issue]) -> PrioritizeLanes {
    let open_numbers = issues
        .iter()
        .filter_map(|issue| issue.github_id.or(issue.gitlab_id))
        .collect::<std::collections::HashSet<_>>();
    let mut lanes = PrioritizeLanes::default();

    for issue in issues {
        if is_deferred_issue(issue) {
            lanes.deferred.push(issue.clone());
            continue;
        }
        if issue.issue_type == IssueType::Epic {
            lanes.needs_atomize.push(issue.clone());
            continue;
        }

        let alignment_errors = validate_planning_alignment(issue);
        let has_triage_error = alignment_errors
            .iter()
            .any(|error| error.starts_with("alignment:") || error.starts_with("not-testable:"));
        if has_triage_error {
            lanes.needs_triage.push(issue.clone());
            continue;
        }

        if alignment_errors
            .iter()
            .any(|error| error.starts_with("too-large:"))
        {
            lanes.needs_atomize.push(issue.clone());
            continue;
        }

        if has_open_dependency(issue, &open_numbers) {
            lanes.blocked_by_dependency.push(issue.clone());
        } else {
            lanes.ready_now.push(issue.clone());
        }
    }

    lanes
}

fn is_deferred_issue(issue: &Issue) -> bool {
    issue
        .labels
        .iter()
        .any(|label| label.eq_ignore_ascii_case("deferred") || label.ends_with(":deferred"))
        || body_field_value(&issue.body, "status")
            .is_some_and(|value| value.eq_ignore_ascii_case("deferred"))
}

fn has_open_dependency(issue: &Issue, open_numbers: &std::collections::HashSet<u64>) -> bool {
    dependency_numbers(issue)
        .iter()
        .any(|number| open_numbers.contains(number))
}

fn dependency_numbers(issue: &Issue) -> std::collections::HashSet<u64> {
    let mut numbers = std::collections::HashSet::new();
    for line in issue.body.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("depends on")
            || lower.contains("dependency")
            || lower.contains("dependencies")
            || lower.contains("blocked by")
            || lower.contains("requires #")
        {
            numbers.extend(extract_hash_numbers(line));
        }
    }
    numbers
}

fn issue_refs_json(issues: &[Issue]) -> Vec<String> {
    issues.iter().map(issue_ref).collect()
}

fn issue_ref(issue: &Issue) -> String {
    issue
        .github_id
        .or(issue.gitlab_id)
        .map(|id| format!("#{}", id))
        .unwrap_or_else(|| issue.slug.clone())
}

fn issue_line(issue: &Issue) -> String {
    format!(
        "- [{}] {} `{}` {} ({})",
        issue.issue_type.as_str(),
        issue_ref(issue),
        priority_label(issue),
        issue.title.trim(),
        issue.state.as_str()
    )
}

struct EpicizeGroups<'a> {
    existing_epics: Vec<&'a Issue>,
    urgent_fixes: Vec<&'a Issue>,
    capability_work: Vec<&'a Issue>,
    maintenance: Vec<&'a Issue>,
    quality: Vec<&'a Issue>,
    needs_triage: Vec<&'a Issue>,
}

fn group_issues_for_epicize(issues: &[Issue]) -> EpicizeGroups<'_> {
    let mut groups = EpicizeGroups {
        existing_epics: Vec::new(),
        urgent_fixes: Vec::new(),
        capability_work: Vec::new(),
        maintenance: Vec::new(),
        quality: Vec::new(),
        needs_triage: Vec::new(),
    };

    for issue in issues {
        match issue.issue_type {
            IssueType::Epic => groups.existing_epics.push(issue),
            IssueType::Bug if priority_rank(issue) <= 1 => groups.urgent_fixes.push(issue),
            IssueType::Bug => groups.quality.push(issue),
            IssueType::Enhancement => groups.capability_work.push(issue),
            IssueType::Refactor => groups.maintenance.push(issue),
            IssueType::Test => groups.quality.push(issue),
        }
    }

    for issue in issues {
        if issue.issue_type != IssueType::Epic
            && issue.body.trim().is_empty()
            && !groups.needs_triage.iter().any(|i| i.slug == issue.slug)
        {
            groups.needs_triage.push(issue);
        }
    }

    groups
}

fn push_issue_group(out: &mut String, title: &str, issues: &[&Issue]) {
    out.push_str(&format!("### {}\n\n", title));
    if issues.is_empty() {
        out.push_str("- none\n\n");
        return;
    }
    for issue in issues {
        out.push_str(&issue_line(issue));
        out.push('\n');
    }
    out.push('\n');
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CapabilityCandidate {
    title: String,
    issue_type: String,
    source_capability_id: String,
    source_capability: String,
    related_capabilities: Vec<String>,
    claim_id: Option<String>,
    capability_gap: String,
    first_gate: String,
    expected_result: String,
    parent_wi_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CapabilityTrackerReconciliation {
    capability: String,
    claim: String,
    active_wi: String,
    tracker_lookup: String,
    capability_gap: String,
    next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapabilityTrackerRefLookup {
    reference: String,
    status: String,
    title: String,
    labels: String,
    url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapabilityPlanSummaryRow {
    capability: String,
    candidate_count: usize,
    existing_wi_refs: Vec<String>,
    next_operator: String,
    first_action: String,
}

#[cfg(test)]
struct CapabilityColumnIndices {
    capability: usize,
    capability_type: Option<usize>,
    surfaces: Option<usize>,
    ec_dimensions: Option<usize>,
    current_state: usize,
    gaps: usize,
    active_wi: usize,
    evidence: usize,
}

fn resolve_capability_path(
    project_root: &Path,
    project: &str,
    override_path: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(if path.is_absolute() {
            path.to_path_buf()
        } else {
            project_root.join(path)
        });
    }

    let config_file = project_root.join("aw.toml");
    let content = std::fs::read_to_string(&config_file)
        .with_context(|| format!("reading {}", config_file.display()))?;
    let parsed: CapabilityConfig =
        toml::from_str(&content).with_context(|| format!("parsing {}", config_file.display()))?;
    let row = parsed
        .projects
        .iter()
        .find(|row| row.name == project || row.aliases.iter().any(|alias| alias == project))
        .ok_or_else(|| anyhow::anyhow!("project '{}' has no [[projects]] entry", project))?;

    let path = if let Some(cap_path) = row.cap_path.as_deref() {
        PathBuf::from(cap_path)
    } else if let Some(project_path) = row.path.as_deref() {
        PathBuf::from(project_path).join("README.md")
    } else {
        anyhow::bail!(
            "project '{}' must declare [[projects]].cap_path or [[projects]].path",
            project
        );
    };

    Ok(if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    })
}

#[cfg(test)]
fn parse_capability_map(body: &str) -> Result<CapabilityMap> {
    let lines = body.lines().collect::<Vec<_>>();
    for (header_idx, line) in lines.iter().enumerate() {
        let Some(header_cells) = parse_markdown_table_row(line) else {
            continue;
        };
        let Some(indices) = capability_column_indices(&header_cells) else {
            continue;
        };

        let mut row_idx = header_idx + 1;
        if row_idx < lines.len() {
            if let Some(cells) = parse_markdown_table_row(lines[row_idx]) {
                if is_markdown_separator_row(&cells) {
                    row_idx += 1;
                }
            }
        }

        let mut rows = Vec::new();
        while row_idx < lines.len() {
            let Some(cells) = parse_markdown_table_row(lines[row_idx]) else {
                break;
            };
            if is_markdown_separator_row(&cells) {
                row_idx += 1;
                continue;
            }
            if cells.iter().all(|cell| cell.trim().is_empty()) {
                row_idx += 1;
                continue;
            }
            rows.push(CapabilityRow {
                capability_id: planning_slug(&table_cell(&cells, indices.capability)),
                capability: table_cell(&cells, indices.capability),
                capability_type: indices
                    .capability_type
                    .map(|idx| table_cell(&cells, idx))
                    .unwrap_or_else(|| "-".to_string()),
                surfaces: indices
                    .surfaces
                    .map(|idx| table_cell(&cells, idx))
                    .unwrap_or_else(|| "-".to_string()),
                ec_dimensions: indices
                    .ec_dimensions
                    .map(|idx| table_cell(&cells, idx))
                    .unwrap_or_else(|| "-".to_string()),
                current_state: table_cell(&cells, indices.current_state),
                gaps: table_cell(&cells, indices.gaps),
                root_wi: "-".to_string(),
                active_wi: table_cell(&cells, indices.active_wi),
                evidence: table_cell(&cells, indices.evidence),
                claim_id: None,
                claim_user_story: None,
            });
            row_idx += 1;
        }

        if rows.is_empty() {
            anyhow::bail!("capability map table is present but contains no capability rows");
        }
        return Ok(CapabilityMap {
            capability_count: rows.len(),
            rows,
            health_note: extract_project_health_note(body),
        });
    }

    anyhow::bail!(
        "no capability map table found; expected markdown header `| Capability | Current State | Gaps | Active WI | Evidence |`"
    )
}

#[cfg(test)]
fn parse_markdown_table_row(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed[1..].contains('|') {
        return None;
    }
    let inner = trimmed.trim_matches('|');
    Some(
        inner
            .split('|')
            .map(|cell| cell.trim().replace("\\|", "|"))
            .collect(),
    )
}

#[cfg(test)]
fn table_cell(cells: &[String], idx: usize) -> String {
    cells
        .get(idx)
        .map(|cell| cell.trim().to_string())
        .filter(|cell| !cell.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
fn capability_column_indices(cells: &[String]) -> Option<CapabilityColumnIndices> {
    let capability = find_table_column(cells, &["capability"])?;
    let capability_type = find_table_column(cells, &["type", "capabilitytype"]);
    let surfaces = find_table_column(cells, &["surface", "surfaces"]);
    let ec_dimensions = find_table_column(cells, &["ecdimensions", "dimensions"]);
    let current_state = find_table_column(cells, &["currentstate", "state"])?;
    let gaps = find_table_column(cells, &["gaps", "gap"])?;
    let active_wi = find_table_column(cells, &["activewi", "activeworkitem", "activeworkitems"])?;
    let evidence = find_table_column(cells, &["evidence", "progress", "proof"])?;
    Some(CapabilityColumnIndices {
        capability,
        capability_type,
        surfaces,
        ec_dimensions,
        current_state,
        gaps,
        active_wi,
        evidence,
    })
}

#[cfg(test)]
fn find_table_column(cells: &[String], aliases: &[&str]) -> Option<usize> {
    cells.iter().position(|cell| {
        let normalized = normalize_table_header(cell);
        aliases.iter().any(|alias| normalized == *alias)
    })
}

#[cfg(test)]
fn normalize_table_header(cell: &str) -> String {
    cell.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

#[cfg(test)]
fn is_markdown_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            !trimmed.is_empty()
                && trimmed.chars().all(|c| matches!(c, '-' | ':' | ' '))
                && trimmed.chars().any(|c| c == '-')
        })
}

fn extract_project_health_note(body: &str) -> Option<String> {
    let mut capturing = false;
    let mut lines = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("##") {
            let heading = trimmed.trim_start_matches('#').trim().to_ascii_lowercase();
            if capturing {
                break;
            }
            if heading == "project health note" || heading == "project health" {
                capturing = true;
                continue;
            }
        }
        if capturing {
            lines.push(line);
        }
    }
    let note = lines.join("\n").trim().to_string();
    if note.is_empty() {
        None
    } else {
        Some(note)
    }
}

fn has_active_wi_ref(row: &CapabilityRow) -> bool {
    !capability_wi_summary_refs(row).is_empty()
}

fn active_wi_refs_text(row: &CapabilityRow) -> String {
    let refs = capability_wi_summary_refs(row);
    if refs.is_empty() {
        summary_cell(&row.active_wi)
    } else {
        refs.join(", ")
    }
}

fn capability_tracker_reconciliations(
    rows: &[CapabilityRow],
    issues: &[Issue],
    resolved_wi_refs: &BTreeMap<String, CapabilityTrackerRefLookup>,
) -> Vec<CapabilityTrackerReconciliation> {
    rows.iter()
        .filter_map(|row| {
            let missing_refs = missing_wi_refs_for_row(row, issues);
            if !(has_actionable_gap(row) && !missing_refs.is_empty()) {
                return None;
            }
            Some(CapabilityTrackerReconciliation {
                capability: row.capability.clone(),
                claim: row
                    .claim_id
                    .as_deref()
                    .or(row.claim_user_story.as_deref())
                    .unwrap_or("-")
                    .to_string(),
                active_wi: missing_refs.join(", "),
                tracker_lookup: tracker_lookup_summary(&missing_refs, resolved_wi_refs),
                capability_gap: row.gaps.clone(),
                next_action: "keep closed/missing refs as advisory history; repair any still-open mislabeled ref, otherwise publish the agent-reviewed bounded replacement"
                    .to_string(),
            })
        })
        .collect()
}

async fn resolve_capability_tracker_ref_lookups(
    rows: &[CapabilityRow],
    issues: &[Issue],
    backend: &dyn IssueBackend,
) -> BTreeMap<String, CapabilityTrackerRefLookup> {
    let mut refs = BTreeSet::new();
    for row in rows {
        if !has_actionable_gap(row) {
            continue;
        }
        for reference in missing_wi_refs_for_row(row, issues) {
            refs.insert(reference);
        }
    }

    let mut lookups = BTreeMap::new();
    for reference in refs {
        let lookup = match tracker_ref_lookup_id(&reference) {
            Some(id) => match backend.get(&id).await {
                Ok(Some(issue)) => CapabilityTrackerRefLookup {
                    reference: reference.clone(),
                    status: issue.state.as_str().to_string(),
                    title: issue.title,
                    labels: if issue.labels.is_empty() {
                        "-".to_string()
                    } else {
                        issue.labels.join(", ")
                    },
                    url: issue.url.unwrap_or_else(|| "-".to_string()),
                },
                Ok(None) => CapabilityTrackerRefLookup {
                    reference: reference.clone(),
                    status: "not_found".to_string(),
                    title: "-".to_string(),
                    labels: "-".to_string(),
                    url: "-".to_string(),
                },
                Err(err) => CapabilityTrackerRefLookup {
                    reference: reference.clone(),
                    status: "lookup_error".to_string(),
                    title: summary_cell(&err.to_string()),
                    labels: "-".to_string(),
                    url: "-".to_string(),
                },
            },
            None => CapabilityTrackerRefLookup {
                reference: reference.clone(),
                status: "unparsed".to_string(),
                title: "-".to_string(),
                labels: "-".to_string(),
                url: "-".to_string(),
            },
        };
        lookups.insert(reference, lookup);
    }
    lookups
}

fn tracker_ref_lookup_id(reference: &str) -> Option<String> {
    let trimmed = reference.trim().trim_start_matches('#').trim();
    if trimmed.chars().all(|ch| ch.is_ascii_digit()) && !trimmed.is_empty() {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn tracker_lookup_summary(
    references: &[String],
    resolved_wi_refs: &BTreeMap<String, CapabilityTrackerRefLookup>,
) -> String {
    let summaries = references
        .iter()
        .map(|reference| match resolved_wi_refs.get(reference) {
            Some(lookup) => {
                if lookup.title == "-" {
                    format!("{}: {}", reference, lookup.status)
                } else {
                    format!("{}: {} - {}", reference, lookup.status, lookup.title)
                }
            }
            None => format!("{}: lookup skipped", reference),
        })
        .collect::<Vec<_>>();
    if summaries.is_empty() {
        "-".to_string()
    } else {
        summaries.join("<br>")
    }
}

fn capability_wi_candidates(rows: &[CapabilityRow], issues: &[Issue]) -> Vec<CapabilityCandidate> {
    let mut candidates: Vec<CapabilityCandidate> = Vec::new();
    for row in rows {
        if !has_actionable_gap(row) {
            continue;
        }
        let mut matches = matching_issues_for_capability(row, issues);
        if let Some(claim_id) = row.claim_id.as_deref() {
            for related_row in rows
                .iter()
                .filter(|related| related.claim_id.as_deref() == Some(claim_id))
            {
                for issue in matching_issues_for_capability(related_row, issues) {
                    if !matches.iter().any(|matched| matched.slug == issue.slug) {
                        matches.push(issue);
                    }
                }
            }
        }
        let claim_has_bounded_wi = row.claim_id.as_deref().is_some_and(|claim_id| {
            matches.iter().any(|issue| {
                issue.issue_type != IssueType::Epic
                    && format!("{}\n{}", issue.title, issue.body)
                        .to_ascii_lowercase()
                        .contains(&claim_id.to_ascii_lowercase())
            })
        });
        if row.claim_id.is_none() && (has_active_wi_ref(row) || !matches.is_empty())
            || row.claim_id.is_some()
                && (claim_has_bounded_wi
                    || matches
                        .iter()
                        .any(|issue| issue.issue_type != IssueType::Epic))
        {
            continue;
        }
        if let Some(existing) = row.claim_id.as_deref().and_then(|claim_id| {
            candidates
                .iter_mut()
                .find(|candidate| candidate.claim_id.as_deref() == Some(claim_id))
        }) {
            let alignment = format!("{} ({})", row.capability_id, row.capability.trim());
            if alignment
                != format!(
                    "{} ({})",
                    existing.source_capability_id,
                    existing.source_capability.trim()
                )
                && !existing.related_capabilities.contains(&alignment)
            {
                existing.related_capabilities.push(alignment.clone());
                existing.expected_result.push_str(&format!(
                    "; related alignment {alignment} declares evidence: {}",
                    row.evidence.trim()
                ));
            }
            continue;
        }
        let mut parent_wi_refs = matches
            .iter()
            .filter(|issue| issue.issue_type == IssueType::Epic)
            .map(|issue| issue_ref(issue))
            .collect::<Vec<_>>();
        parent_wi_refs.sort();
        parent_wi_refs.dedup();
        candidates.push(CapabilityCandidate {
            title: if let Some(claim_id) = row.claim_id.as_deref() {
                format!(
                    "Close capability claim: {} / {}",
                    row.capability.trim(),
                    claim_id
                )
            } else {
                format!("Close capability gap: {}", row.capability.trim())
            },
            issue_type: infer_candidate_issue_type(&row.gaps).to_string(),
            source_capability_id: row.capability_id.clone(),
            source_capability: row.capability.clone(),
            related_capabilities: Vec::new(),
            claim_id: row.claim_id.clone(),
            capability_gap: row.gaps.clone(),
            first_gate: if row.claim_id.is_some() {
                capability_candidate_first_gate(row)
            } else {
                "Create one bounded WI with acceptance criteria and a concrete verification command."
                    .to_string()
            },
            expected_result: capability_candidate_expected_result(row),
            parent_wi_refs,
        });
    }
    candidates
}

fn capability_candidate_first_gate(row: &CapabilityRow) -> String {
    [&row.evidence]
        .into_iter()
        .find_map(|text| {
            text.split('`')
                .enumerate()
                .filter(|(index, _)| index % 2 == 1)
                .map(|(_, code)| code.trim())
                .find(|code| looks_like_runnable_verification_command(code))
                .map(str::to_string)
        })
        .or_else(|| capability_evidence_command(&row.evidence))
        .or_else(|| {
            row.evidence
                .strip_prefix("claim gate:")
                .map(str::trim)
                .filter(|command| looks_like_runnable_verification_command(command))
                .map(str::to_string)
        })
        .or_else(|| {
            row.ec_dimensions
                .split('`')
                .enumerate()
                .filter(|(index, _)| index % 2 == 1)
                .map(|(_, code)| code.trim())
                .find(|code| looks_like_runnable_verification_command(code))
                .map(str::to_string)
        })
        .unwrap_or_else(|| row.evidence.clone())
}

fn capability_evidence_command(evidence: &str) -> Option<String> {
    let mut test_project: Option<String> = None;
    let mut tests = Vec::new();
    let mut script = None;
    for token in evidence.split(|character: char| {
        character.is_whitespace() || matches!(character, ';' | ',' | '(' | ')')
    }) {
        let path = token.trim_matches(|character: char| {
            !character.is_ascii_alphanumeric() && !matches!(character, '/' | '.' | '-' | '_')
        });
        let segments = path.split('/').collect::<Vec<_>>();
        if segments.len() >= 4
            && segments[0] == "apps"
            && segments[2] == "tests"
            && segments.last().is_some_and(|leaf| leaf.ends_with(".rs"))
        {
            let project = segments[1].to_string();
            if test_project
                .as_ref()
                .is_none_or(|current| current == &project)
            {
                test_project.get_or_insert(project);
                let test = segments.last()?.trim_end_matches(".rs").to_string();
                if !tests.contains(&test) {
                    tests.push(test);
                }
            }
        }
        if segments.len() >= 4
            && segments[0] == "apps"
            && segments[2] == "scripts"
            && segments.last().is_some_and(|leaf| leaf.ends_with(".sh"))
        {
            script.get_or_insert_with(|| format!("bash {path}"));
        }
    }
    if let Some(project) = test_project {
        let tests = tests
            .iter()
            .map(|test| format!("--test {test}"))
            .collect::<Vec<_>>()
            .join(" ");
        return Some(format!("cargo test -p {project} {tests}"));
    }
    script
}

fn capability_candidate_expected_result(row: &CapabilityRow) -> String {
    let claim = row
        .claim_id
        .as_deref()
        .unwrap_or(row.capability_id.as_str());
    let observable = claim.replace('-', " ");
    let gate = capability_candidate_first_gate(row);
    let evidence = row
        .evidence
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "`{gate}` exits 0 and observes `{observable}` for claim `{claim}` through the declared oracle; a missing oracle, zero matches, or an unobserved claim fails. Oracle and evidence: {evidence}"
    )
}

fn looks_like_runnable_verification_command(command: &str) -> bool {
    let command = command.trim();
    const PREFIXES: &[&str] = &[
        "./",
        "aw ",
        "bash ",
        "cargo ",
        "cd ",
        "docker ",
        "guard ",
        "kind ",
        "kubectl ",
        "kustomize ",
        "make ",
        "meter ",
        "npm ",
        "pnpm ",
        "python ",
        "python3 ",
        "terraform ",
        "vat ",
    ];
    PREFIXES.iter().any(|prefix| command.starts_with(prefix))
        || command
            .split_whitespace()
            .next()
            .is_some_and(|token| token.contains('='))
            && command.split_whitespace().count() > 1
}

fn capability_plan_summary_rows(
    rows: &[CapabilityRow],
    issues: &[Issue],
    candidates: &[CapabilityCandidate],
) -> Vec<CapabilityPlanSummaryRow> {
    let mut summaries: Vec<CapabilityPlanSummaryRow> = Vec::new();
    for row in rows {
        let matches = matching_issues_for_capability(row, issues);
        let row_candidates = candidates
            .iter()
            .filter(|candidate| {
                candidate.source_capability == row.capability
                    && candidate.capability_gap == row.gaps
            })
            .collect::<Vec<_>>();
        if row_candidates.is_empty() && matches.is_empty() && !has_actionable_gap(row) {
            continue;
        }

        let position = summaries
            .iter()
            .position(|summary| summary.capability == row.capability)
            .unwrap_or_else(|| {
                summaries.push(CapabilityPlanSummaryRow {
                    capability: row.capability.clone(),
                    candidate_count: 0,
                    existing_wi_refs: Vec::new(),
                    next_operator: "monitor".to_string(),
                    first_action: "monitor".to_string(),
                });
                summaries.len() - 1
            });
        let summary = &mut summaries[position];
        summary.candidate_count += row_candidates.len();
        for reference in capability_wi_summary_refs(row) {
            if !summary.existing_wi_refs.contains(&reference) {
                summary.existing_wi_refs.push(reference);
            }
        }
        for issue in &matches {
            let reference = issue_ref(issue);
            if !summary.existing_wi_refs.contains(&reference) {
                summary.existing_wi_refs.push(reference);
            }
        }
        let operator = if row_candidates.is_empty() {
            suggested_capability_operator(row, issues)
        } else {
            "epicize -> atomize"
        };
        summary.next_operator = merge_capability_plan_operator(&summary.next_operator, operator);
        if summary.first_action == "monitor" {
            if let Some(candidate) = row_candidates.first() {
                summary.first_action = candidate.title.clone();
            } else {
                let missing_refs = missing_wi_refs_for_row(row, issues);
                if has_actionable_gap(row) && !missing_refs.is_empty() {
                    summary.first_action =
                        format!("Reconcile WI reference: {}", missing_refs.join(", "));
                } else if has_actionable_gap(row) {
                    summary.first_action = row.gaps.clone();
                } else if !summary.existing_wi_refs.is_empty() {
                    summary.first_action = "confirm existing WI linkage".to_string();
                }
            }
        }
    }
    summaries
}

fn capability_wi_summary_refs(row: &CapabilityRow) -> Vec<String> {
    let mut refs = wi_summary_refs_from_text(&row.root_wi);
    for reference in wi_summary_refs_from_text(&row.active_wi) {
        if !refs.contains(&reference) {
            refs.push(reference);
        }
    }
    refs
}

fn missing_wi_refs_for_row(row: &CapabilityRow, issues: &[Issue]) -> Vec<String> {
    capability_wi_summary_refs(row)
        .into_iter()
        .filter(|reference| {
            !issues
                .iter()
                .any(|issue| issue_matches_wi_reference(reference, issue))
        })
        .collect()
}

fn issue_matches_wi_reference(reference: &str, issue: &Issue) -> bool {
    if let Some(id) = tracker_ref_lookup_id(reference).and_then(|id| id.parse::<u64>().ok()) {
        return issue.github_id.or(issue.gitlab_id) == Some(id);
    }
    let reference_lower = reference.to_ascii_lowercase();
    let issue_ref = issue_ref(issue).to_ascii_lowercase();
    let issue_slug = issue.slug.to_ascii_lowercase();
    let title = issue.title.to_ascii_lowercase();
    reference_lower.contains(&issue_ref)
        || reference_lower.contains(&issue_slug)
        || (!title.is_empty() && reference_lower.contains(&title))
}

fn wi_summary_refs_from_text(text: &str) -> Vec<String> {
    if is_empty_active_wi(&text.to_ascii_lowercase()) {
        return Vec::new();
    }
    let mut numbers = extract_hash_numbers(text).into_iter().collect::<Vec<_>>();
    if numbers.is_empty() {
        numbers = extract_active_wi_numbers(text);
    }
    numbers.sort_unstable();
    if numbers.is_empty() {
        vec![summary_cell(text)]
    } else {
        numbers
            .into_iter()
            .map(|number| format!("#{number}"))
            .collect()
    }
}

fn extract_active_wi_numbers(text: &str) -> Vec<u64> {
    let mut numbers = std::collections::HashSet::new();
    let mut digits = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        if !digits.is_empty() {
            if let Ok(number) = digits.parse::<u64>() {
                numbers.insert(number);
            }
            digits.clear();
        }
    }
    if !digits.is_empty() {
        if let Ok(number) = digits.parse::<u64>() {
            numbers.insert(number);
        }
    }
    let mut sorted = numbers.into_iter().collect::<Vec<_>>();
    sorted.sort_unstable();
    sorted
}

fn merge_capability_plan_operator(current: &str, next: &str) -> String {
    let priority = |operator: &str| match operator {
        "epicize -> atomize" => 4,
        "atomize -> prioritize" => 3,
        "prioritize" => 2,
        "reconcile tracker ref" => 2,
        "monitor" => 1,
        _ => 0,
    };
    if priority(next) > priority(current) {
        next.to_string()
    } else {
        current.to_string()
    }
}

fn summary_cell(text: &str) -> String {
    const LIMIT: usize = 140;
    let trimmed = text.trim();
    if trimmed.chars().count() <= LIMIT {
        return trimmed.to_string();
    }
    let mut truncated = trimmed.chars().take(LIMIT).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn has_actionable_gap(row: &CapabilityRow) -> bool {
    let gap = row.gaps.trim();
    if gap.is_empty() {
        return false;
    }
    let normalized = gap
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>();
    !matches!(
        normalized.as_str(),
        "" | "none" | "nogap" | "na" | "n/a" | "complete" | "covered" | "done" | "closed"
    )
}

fn infer_candidate_issue_type(gap: &str) -> &'static str {
    let lower = gap.to_ascii_lowercase();
    if lower.contains("test") || lower.contains("coverage") || lower.contains("verify") {
        "test"
    } else if lower.contains("refactor") || lower.contains("rename") || lower.contains("migrate") {
        "refactor"
    } else {
        "enhancement"
    }
}

fn matching_issues_for_capability<'a>(row: &CapabilityRow, issues: &'a [Issue]) -> Vec<&'a Issue> {
    let mut explicit_active_numbers = extract_hash_numbers(&row.active_wi);
    explicit_active_numbers.extend(extract_active_wi_numbers(&row.active_wi));
    let mut explicit_root_numbers = extract_hash_numbers(&row.root_wi);
    explicit_root_numbers.extend(extract_active_wi_numbers(&row.root_wi));
    let active_wi_lower = row.active_wi.to_ascii_lowercase();
    let root_wi_lower = row.root_wi.to_ascii_lowercase();
    let keywords = capability_keywords(row);

    issues
        .iter()
        .filter(|issue| {
            let issue_id = issue.github_id.or(issue.gitlab_id);
            if issue_id.is_some_and(|id| explicit_active_numbers.contains(&id)) {
                return true;
            }
            if issue_id.is_some_and(|id| explicit_root_numbers.contains(&id))
                && (row.claim_id.is_none() || issue.issue_type == IssueType::Epic)
            {
                return true;
            }

            if !is_empty_active_wi(&active_wi_lower) || !is_empty_active_wi(&root_wi_lower) {
                let issue_ref = issue_ref(issue).to_ascii_lowercase();
                let title = issue.title.to_ascii_lowercase();
                if active_wi_lower.contains(&issue_ref)
                    || active_wi_lower.contains(&issue.slug.to_ascii_lowercase())
                    || (!title.is_empty() && active_wi_lower.contains(&title))
                {
                    return true;
                }
                if (row.claim_id.is_none() || issue.issue_type == IssueType::Epic)
                    && (root_wi_lower.contains(&issue_ref)
                        || root_wi_lower.contains(&issue.slug.to_ascii_lowercase())
                        || (!title.is_empty() && root_wi_lower.contains(&title)))
                {
                    return true;
                }
            }

            let search = format!("{}\n{}", issue.title, issue.body).to_ascii_lowercase();
            if let Some(claim_id) = row.claim_id.as_deref() {
                return search.contains(&claim_id.to_ascii_lowercase());
            }
            if keywords.is_empty() {
                return false;
            }
            let hits = keywords
                .iter()
                .filter(|keyword| search.contains(keyword.as_str()))
                .count();
            hits >= 3
        })
        .collect()
}

fn extract_hash_numbers(text: &str) -> std::collections::HashSet<u64> {
    let mut numbers = std::collections::HashSet::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '#' {
            continue;
        }
        let mut digits = String::new();
        while let Some(next) = chars.peek() {
            if next.is_ascii_digit() {
                digits.push(*next);
                chars.next();
            } else {
                break;
            }
        }
        if let Ok(number) = digits.parse::<u64>() {
            numbers.insert(number);
        }
    }
    numbers
}

fn is_empty_active_wi(active_wi_lower: &str) -> bool {
    let normalized = active_wi_lower
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>();
    normalized.is_empty()
        || matches!(
            normalized.as_str(),
            "none" | "na" | "n/a" | "tbd" | "todo" | "notyet"
        )
}

fn capability_keywords(row: &CapabilityRow) -> Vec<String> {
    let stopwords = [
        "active",
        "agent",
        "and",
        "capability",
        "current",
        "from",
        "into",
        "none",
        "project",
        "state",
        "that",
        "this",
        "with",
    ];
    let mut keywords = format!(
        "{} {} {} {}",
        row.capability,
        row.gaps,
        row.claim_id.as_deref().unwrap_or_default(),
        row.claim_user_story.as_deref().unwrap_or_default()
    )
    .split(|c: char| !c.is_ascii_alphanumeric())
    .filter_map(|token| {
        let token = token.trim().to_ascii_lowercase();
        if token.len() < 4 || stopwords.contains(&token.as_str()) {
            None
        } else {
            Some(token)
        }
    })
    .collect::<Vec<_>>();
    keywords.sort();
    keywords.dedup();
    keywords
}

fn suggested_capability_operator(row: &CapabilityRow, issues: &[Issue]) -> &'static str {
    let matches = matching_issues_for_capability(row, issues);
    if !has_actionable_gap(row) {
        "monitor"
    } else if !missing_wi_refs_for_row(row, issues).is_empty() {
        "reconcile tracker ref"
    } else if matches.is_empty() {
        "epicize -> atomize"
    } else if matches
        .iter()
        .any(|issue| issue.issue_type == IssueType::Epic || looks_too_large_for_atomic_wi(issue))
    {
        "atomize -> prioritize"
    } else {
        "prioritize"
    }
}

fn issue_refs(matches: &[&Issue]) -> String {
    if matches.is_empty() {
        "none".to_string()
    } else {
        matches
            .iter()
            .map(|issue| issue_ref(issue))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn push_epic_candidate(out: &mut String, name: &str, goal: &str, issues: &[&Issue]) {
    if issues.is_empty() {
        return;
    }
    out.push_str(&format!("### {}\n\n", name));
    out.push_str("Goal:\n");
    out.push_str(&format!("- {}\n\n", goal));
    out.push_str("Included work-items:\n");
    for issue in issues {
        out.push_str(&issue_line(issue));
        out.push('\n');
    }
    out.push_str("\nAcceptance criteria draft:\n");
    out.push_str("- The included work-items are deduplicated and ordered.\n");
    out.push_str(
        "- Every included work-item has clear scope and reference context before prioritize.\n",
    );
    out.push_str("- Deferred work is explicitly listed outside this epic candidate.\n\n");
}

fn render_capability_wi_plan(
    project: &str,
    title: &str,
    backend_name: &str,
    cap_path: &Path,
    capability_map: &CapabilityMap,
    issues: &[Issue],
    candidates: &[CapabilityCandidate],
    resolved_wi_refs: &BTreeMap<String, CapabilityTrackerRefLookup>,
    warnings: &[String],
    review_backing: &str,
) -> String {
    let mut out = String::new();
    let reconciliations =
        capability_tracker_reconciliations(&capability_map.rows, issues, resolved_wi_refs);
    out.push_str("---\n");
    out.push_str("draft: true\n");
    out.push_str("kind: capability_plan\n");
    out.push_str(&format!("project: {}\n", yaml_quote(project)));
    out.push_str(&format!("title: {}\n", yaml_quote(title)));
    out.push_str(&format!("backend: {}\n", yaml_quote(backend_name)));
    out.push_str(&format!(
        "cap_path: {}\n",
        yaml_quote(&cap_path.display().to_string())
    ));
    out.push_str(&format!(
        "capability_count: {}\n",
        capability_map.capability_count
    ));
    out.push_str(&format!(
        "planning_row_count: {}\n",
        capability_map.rows.len()
    ));
    out.push_str(&format!("issue_count: {}\n", issues.len()));
    out.push_str(&format!("candidate_count: {}\n", candidates.len()));
    out.push_str(&format!(
        "reconciliation_count: {}\n",
        reconciliations.len()
    ));
    out.push_str(&format!(
        "resolved_wi_ref_count: {}\n",
        resolved_wi_refs.len()
    ));
    if !warnings.is_empty() {
        out.push_str("warnings:\n");
        for warning in warnings {
            out.push_str(&format!("  - {}\n", yaml_quote(warning)));
        }
    }
    let requires_hitl = review_backing == "human";
    out.push_str(&format!("review_backing: {}\n", yaml_quote(review_backing)));
    out.push_str(&format!("requires_hitl: {requires_hitl}\n"));
    out.push_str(if requires_hitl {
        "hitl_status: pending_human\n"
    } else {
        "hitl_status: pending_agent_review\n"
    });
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n\n", title));
    out.push_str("## Purpose\n\n");
    out.push_str("- Translate the confirmed project capability map into WI planning inputs.\n");
    out.push_str("- Cross-check capability gaps against the current open work-item inventory.\n");
    out.push_str("- Keep this artifact local until an independent digest-bound review accepts its WI drafts and tracker reconciliations.\n\n");

    out.push_str("## Source\n\n");
    out.push_str(&format!("- Capability map: `{}`\n", cap_path.display()));
    out.push_str(&format!("- Issue backend: `{}`\n", backend_name));
    out.push_str(&format!("- Open work-items scanned: `{}`\n", issues.len()));
    if !warnings.is_empty() {
        out.push_str("\n### Planning Warnings\n\n");
        for warning in warnings {
            out.push_str(&format!("- {}\n", warning));
        }
        out.push('\n');
    }
    if let Some(note) = &capability_map.health_note {
        out.push_str("\n### Project Health Note\n\n");
        out.push_str(note);
        out.push_str("\n\n");
    } else {
        out.push('\n');
    }

    out.push_str("## Confirmation Summary\n\n");
    out.push_str("| Capability | Candidate WIs | Existing WI | Next operator | First action |\n");
    out.push_str("|------------|--------------:|-------------|---------------|--------------|\n");
    let summary_rows = capability_plan_summary_rows(&capability_map.rows, issues, candidates);
    if summary_rows.is_empty() {
        out.push_str("| none | 0 | none | monitor | monitor |\n\n");
    } else {
        for row in summary_rows {
            let refs = if row.existing_wi_refs.is_empty() {
                "none".to_string()
            } else {
                row.existing_wi_refs.join(", ")
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                markdown_table_cell(&row.capability),
                row.candidate_count,
                markdown_table_cell(&refs),
                row.next_operator,
                markdown_table_cell(&summary_cell(&row.first_action))
            ));
        }
        out.push('\n');
    }

    if !reconciliations.is_empty() {
        out.push_str("## Existing WI Refs Not In Open Inventory\n\n");
        out.push_str("These rows already have README WI references, but the current open issue inventory did not contain those IDs. Treat them as tracker reconciliation work, not automatic new WI candidates.\n\n");
        out.push_str(
            "| Capability | Claim | Active WI | Tracker lookup | Capability gap | Next action |\n",
        );
        out.push_str(
            "|------------|-------|-----------|----------------|----------------|-------------|\n",
        );
        for reconciliation in &reconciliations {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                markdown_table_cell(&reconciliation.capability),
                markdown_table_cell(&reconciliation.claim),
                markdown_table_cell(&reconciliation.active_wi),
                markdown_table_cell(&reconciliation.tracker_lookup),
                markdown_table_cell(&reconciliation.capability_gap),
                markdown_table_cell(&reconciliation.next_action)
            ));
        }
        out.push('\n');
    }

    if !resolved_wi_refs.is_empty() {
        out.push_str("## Tracker WI Ref Lookups\n\n");
        out.push_str("These lookups resolve capability WI refs that were absent from the current open issue inventory. Closed refs remain derived provenance; `not_found` and `lookup_error` entries require explicit review before tracker state changes.\n\n");
        out.push_str("| WI | Lookup status | Title | Labels | URL |\n");
        out.push_str("|----|---------------|-------|--------|-----|\n");
        for lookup in resolved_wi_refs.values() {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                markdown_table_cell(&lookup.reference),
                markdown_table_cell(&lookup.status),
                markdown_table_cell(&lookup.title),
                markdown_table_cell(&lookup.labels),
                markdown_table_cell(&lookup.url)
            ));
        }
        out.push('\n');
    }

    out.push_str("## Capability Planning Matrix\n\n");
    out.push_str("| Capability | Type | Surfaces | EC Dimensions | Claim | Current state | Gap | Active WI | Matching open WI | Next planning operator | Evidence |\n");
    out.push_str("|------------|------|----------|---------------|-------|---------------|-----|-----------|------------------|------------------------|----------|\n");
    for row in &capability_map.rows {
        let matches = matching_issues_for_capability(row, issues);
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            markdown_table_cell(&row.capability),
            markdown_table_cell(&row.capability_type),
            markdown_table_cell(&row.surfaces),
            markdown_table_cell(&row.ec_dimensions),
            markdown_table_cell(
                row.claim_id
                    .as_deref()
                    .or(row.claim_user_story.as_deref())
                    .unwrap_or("-")
            ),
            markdown_table_cell(&row.current_state),
            markdown_table_cell(&row.gaps),
            markdown_table_cell(&active_wi_refs_text(row)),
            markdown_table_cell(&issue_refs(&matches)),
            suggested_capability_operator(row, issues),
            markdown_table_cell(&row.evidence)
        ));
    }

    out.push_str("\n## New WI Candidates\n\n");
    out.push_str("| Candidate title | Type | Source capability | Capability gap | First gate |\n");
    out.push_str("|-----------------|------|-------------------|----------------|------------|\n");
    if candidates.is_empty() {
        out.push_str("| none | - | - | - | - |\n");
    } else {
        for candidate in candidates {
            let source_capability = if candidate.related_capabilities.is_empty() {
                candidate.source_capability.clone()
            } else {
                format!(
                    "{}<br>{}",
                    candidate.source_capability,
                    candidate.related_capabilities.join("<br>")
                )
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                markdown_table_cell(&candidate.title),
                candidate.issue_type,
                markdown_table_cell(&source_capability),
                markdown_table_cell(&candidate.capability_gap),
                markdown_table_cell(&candidate.first_gate)
            ));
        }
    }

    if !candidates.is_empty() {
        out.push_str("\n## Candidate WI Drafts\n\n");
        out.push_str("Review these bounded local draft bodies before publishing tracker issues. Acceptance must be independent and digest-bound.\n\n");
        for (index, candidate) in candidates.iter().enumerate() {
            render_capability_candidate_wi_draft(&mut out, project, cap_path, index + 1, candidate);
        }
    }

    out.push_str("\n## Existing WI Follow-up\n\n");
    let mut wrote_follow_up = false;
    for row in &capability_map.rows {
        let matches = matching_issues_for_capability(row, issues);
        if matches.is_empty() {
            continue;
        }
        wrote_follow_up = true;
        out.push_str(&format!("### {}\n\n", row.capability.trim()));
        for issue in matches {
            out.push_str(&issue_line(issue));
            out.push('\n');
        }
        out.push('\n');
    }
    if !wrote_follow_up {
        out.push_str("- none\n");
    }

    out.push_str("\n## Recommended CLI Sequence\n\n");
    out.push_str("1. Complete the digest-bound review payload emitted by `aw wi plan`.\n");
    out.push_str("2. Run the emitted `aw wi plan-review --evidence-file <path>` command; accepted review publishes only bounded, deduplicated candidates.\n");
    out.push_str(&format!(
        "3. `aw goal capability --project {} --non-interactive`\n",
        project
    ));

    out.push_str("\n## Confirmation Guardrails\n\n");
    out.push_str("- Treat README capability rows as the confirmed anchor; if the direction changed, rerun `/aw:capability` before publishing WIs.\n");
    out.push_str("- Accepted independent review authorizes publication of this digest's bounded candidates; needs_revision publishes nothing.\n");
    out.push_str(
        "- Closed WI refs remain derived provenance; not_found and lookup_error refs require an explicit review finding before replacement.\n",
    );
    out.push_str("- Non-epic WIs still need Capability Alignment, Scope, Acceptance Criteria, and Reference Context before `aw td`.\n");
    out
}

fn render_capability_candidate_wi_draft(
    out: &mut String,
    project: &str,
    cap_path: &Path,
    index: usize,
    candidate: &CapabilityCandidate,
) {
    out.push_str(&format!(
        "### Candidate {}: {}\n\n",
        index,
        candidate.title.trim()
    ));
    out.push_str(&format!("- Type: `{}`\n", candidate.issue_type));
    out.push_str(&format!(
        "- Source capability: `{}`\n",
        candidate.source_capability.trim()
    ));
    out.push_str(&format!(
        "- Capability gap: `{}`\n\n",
        candidate.capability_gap.trim()
    ));
    out.push_str("```md\n");
    out.push_str(&capability_candidate_wi_body(project, cap_path, candidate));
    out.push_str("```\n\n");
}

fn capability_candidate_wi_body(
    project: &str,
    cap_path: &Path,
    candidate: &CapabilityCandidate,
) -> String {
    let claim_or_gap = candidate
        .claim_id
        .as_deref()
        .unwrap_or(candidate.capability_gap.trim());
    let mut out = format!("# {}\n\n", candidate.title.trim());
    out.push_str("## Problem\n\n");
    out.push_str(&format!(
        "The `{}` capability has a confirmed gap or claim that is not backed by a bounded active work item in the current issue inventory.\n\n",
        candidate.source_capability_id
    ));
    out.push_str("## Capability Alignment\n\n");
    out.push_str(&format!(
        "Capability: `{}` ({})\n",
        candidate.source_capability_id,
        candidate.source_capability.trim()
    ));
    if !candidate.related_capabilities.is_empty() {
        out.push_str("Related Capability Alignments:\n");
        for alignment in &candidate.related_capabilities {
            out.push_str(&format!("- {alignment}\n"));
        }
    }
    out.push_str(&format!("Capability Gap: `{claim_or_gap}`\n"));
    out.push_str(&format!(
        "Progress Evidence: {}\n\n",
        candidate.first_gate.trim()
    ));
    out.push_str("## Requirements\n\n");
    out.push_str("- R1: Implement the scoped capability behavior and make its declared verification gate pass; linkage or prose alone is not closure.\n");
    out.push_str("- R2: Keep the capability contract, work item, and TD/CB evidence aligned.\n\n");
    out.push_str("## Scope\n\n### In Scope\n\n");
    out.push_str("- Resolve this specific capability gap or claim.\n");
    out.push_str(
        "- Add, repair, or link the required verification evidence from the capability map.\n\n",
    );
    out.push_str("### Out of Scope\n\n");
    out.push_str("- Unrelated capability promise changes.\n");
    out.push_str("- Publishing capability status changes without passing the declared gates.\n\n");
    out.push_str("## Acceptance Criteria\n\n");
    out.push_str(&format!(
        "- AC1: `aw capability check --project {project}` no longer reports this candidate as a planning blocker.\n"
    ));
    out.push_str("- AC2: The work item links primary TD/CB evidence that defines the implementation edge and the independent observable oracle; a documentation-only artifact is insufficient.\n");
    out.push_str("- AC3: The declared verification gate exits 0 and produces the Expected Result below. Downgrade or deferral requires a separate explicit capability-contract decision and does not close this WI.\n\n");
    if looks_like_runnable_verification_command(&candidate.first_gate) {
        out.push_str("\n### Verification Gate\n\n");
        out.push_str(&format!("`{}`\n\n", candidate.first_gate.trim()));
        out.push_str("Expected Result:\n");
        out.push_str(candidate.expected_result.trim());
        out.push_str(".\n\n");
    }
    out.push_str("## Reference Context\n\n### Related Specs\n\n");
    out.push_str("| Spec | Relevance |\n|------|-----------|\n");
    out.push_str(&format!(
        "| {} | capability source anchor |\n\n",
        cap_path.display()
    ));
    if !candidate.parent_wi_refs.is_empty() {
        out.push_str("Parent WI: ");
        out.push_str(&candidate.parent_wi_refs.join(", "));
        out.push_str("\n\n");
    }
    out.push_str("### Spec Plan\n\n");
    out.push_str("| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n");
    out.push_str(&format!(
        "| {} | create | {} |\n",
        capability_candidate_spec_id(candidate),
        cap_path.display()
    ));
    out
}

fn capability_candidate_spec_id(candidate: &CapabilityCandidate) -> String {
    let mut slug = candidate
        .title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}

fn render_epicize_plan(
    project: &str,
    title: &str,
    backend_name: &str,
    issues: &[Issue],
    capability_document: Option<&crate::cli::capability::CapabilityDocument>,
) -> String {
    let groups = group_issues_for_epicize(issues);
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("draft: true\n");
    out.push_str("kind: epicize\n");
    out.push_str(&format!("project: {}\n", yaml_quote(project)));
    out.push_str(&format!("title: {}\n", yaml_quote(title)));
    out.push_str(&format!("backend: {}\n", yaml_quote(backend_name)));
    out.push_str(&format!("issue_count: {}\n", issues.len()));
    out.push_str("requires_hitl: true\n");
    out.push_str("hitl_status: pending\n");
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n\n", title));
    out.push_str("## Purpose\n\n");
    out.push_str("- Classify the project work-item inventory into epic candidates.\n");
    out.push_str("- Convert README capability roots into confirmed epic/subepic candidates when a Markdown capability map is available.\n");
    out.push_str(
        "- Identify duplicate, underspecified, or deferred requirements before prioritize.\n",
    );
    out.push_str("- Keep this artifact local until a human confirms the candidate epics.\n\n");
    if let Some(document) = capability_document {
        push_capability_epic_candidates(&mut out, document);
    }
    out.push_str("## Existing Epics\n\n");
    if groups.existing_epics.is_empty() {
        out.push_str("- none\n");
    } else {
        for issue in &groups.existing_epics {
            out.push_str(&issue_line(issue));
            out.push('\n');
        }
    }
    out.push_str("\n## Requirement Groups\n\n");
    push_issue_group(&mut out, "Urgent fixes", &groups.urgent_fixes);
    push_issue_group(&mut out, "Capability work", &groups.capability_work);
    push_issue_group(&mut out, "Maintenance / refactor", &groups.maintenance);
    push_issue_group(&mut out, "Quality / tests", &groups.quality);
    push_issue_group(&mut out, "Needs triage", &groups.needs_triage);
    out.push_str("## Epic Candidates\n\n");
    push_epic_candidate(
        &mut out,
        "Stabilize Current Behavior",
        "Resolve high-priority defects and correctness risks before larger feature work.",
        &groups.urgent_fixes,
    );
    push_epic_candidate(
        &mut out,
        "Expand Project Capability",
        "Deliver user-visible enhancements that share the same project context.",
        &groups.capability_work,
    );
    push_epic_candidate(
        &mut out,
        "Improve Maintainability",
        "Reduce implementation friction and prepare the codebase for later work.",
        &groups.maintenance,
    );
    push_epic_candidate(
        &mut out,
        "Raise Quality Bar",
        "Close test and validation gaps that reduce confidence in future changes.",
        &groups.quality,
    );
    if groups.urgent_fixes.is_empty()
        && groups.capability_work.is_empty()
        && groups.maintenance.is_empty()
        && groups.quality.is_empty()
    {
        out.push_str("- none\n\n");
    }
    out.push_str("## Required HITL Brief\n\n");
    out.push_str(
        "This epic draft requires human confirmation before publishing tracker changes.\n\n",
    );
    out.push_str(
        "- Merge groups that are clearly one outcome; split groups that mix unrelated goals.\n",
    );
    out.push_str("- Mark duplicate work-items and choose one canonical issue per duplicate set.\n");
    out.push_str("- For each accepted epic candidate, produce title, problem statement, acceptance criteria, included issues, deferred issues, and execution order.\n");
    out.push_str(
        "- Do not publish tracker changes from this artifact without human confirmation.\n",
    );
    out
}

fn push_capability_epic_candidates(
    out: &mut String,
    document: &crate::cli::capability::CapabilityDocument,
) {
    out.push_str("## Capability Epic Candidates\n\n");
    out.push_str("| Work Root | Kind | Source Capability | WI | Status | Promise / Scope |\n");
    out.push_str("|---|---|---|---:|---|---|\n");
    for capability in &document.capabilities {
        out.push_str(&format!(
            "| {} | epic | {} | {} | {} | {} |\n",
            markdown_table_cell(&capability.title),
            markdown_table_cell(&capability.id),
            markdown_table_cell(&capability_root_wi(capability)),
            markdown_table_cell(capability.status.as_str()),
            markdown_table_cell(&capability.promise),
        ));
        for gap in &capability.gaps {
            out.push_str(&format!(
                "| {} | subepic | {} | {} | {} | {} |\n",
                markdown_table_cell(&gap.summary),
                markdown_table_cell(&capability.id),
                markdown_table_cell(gap.active_wi.as_deref().unwrap_or("-")),
                markdown_table_cell(gap.status.as_str()),
                markdown_table_cell(&gap.id),
            ));
        }
    }
    out.push_str("\n## Capability Epicization Rules\n\n");
    out.push_str("- Every capability heading maps to an epic/subepic root candidate.\n");
    out.push_str("- Every capability work-root row maps to one WI root candidate, defaulting to epic/subepic granularity.\n");
    out.push_str(
        "- Atomic change WIs are created by `aw wi atomize` after these roots are confirmed.\n\n",
    );
}

fn capability_root_wi(capability: &crate::cli::capability::CapabilitySection) -> String {
    capability
        .gaps
        .iter()
        .find_map(|gap| gap.active_wi.as_deref())
        .filter(|wi| !wi.trim().is_empty() && *wi != "-")
        .unwrap_or("-")
        .to_string()
}

#[derive(Debug, Clone)]
struct AtomicCandidate {
    source_ref: String,
    title: String,
    capability_gap: String,
    verification: String,
}

fn atomize_candidates(issues: &[Issue]) -> Vec<AtomicCandidate> {
    let mut candidates = Vec::new();
    for issue in issues {
        if issue.issue_type == IssueType::Epic || looks_too_large_for_atomic_wi(issue) {
            let title = issue.title.trim();
            candidates.push(AtomicCandidate {
                source_ref: issue_ref(issue),
                title: format!("Clarify the first bounded slice for {}", title),
                capability_gap: "Human-confirm the capability gap and choose one visible outcome."
                    .to_string(),
                verification:
                    "Produces one non-epic WI with acceptance criteria and a concrete verification gate."
                        .to_string(),
            });
            candidates.push(AtomicCandidate {
                source_ref: issue_ref(issue),
                title: format!("Deliver the smallest testable increment for {}", title),
                capability_gap:
                    "Implement only the first independently verifiable behavior; defer the rest."
                        .to_string(),
                verification:
                    "A single command or fixture proves the increment without relying on future roadmap work."
                        .to_string(),
            });
        }
    }
    candidates
}

fn render_atomize_plan(
    project: &str,
    title: &str,
    backend_name: &str,
    issues: &[Issue],
    candidates: &[AtomicCandidate],
) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("draft: true\n");
    out.push_str("kind: atomize\n");
    out.push_str(&format!("project: {}\n", yaml_quote(project)));
    out.push_str(&format!("title: {}\n", yaml_quote(title)));
    out.push_str(&format!("backend: {}\n", yaml_quote(backend_name)));
    out.push_str(&format!("issue_count: {}\n", issues.len()));
    out.push_str(&format!("candidate_count: {}\n", candidates.len()));
    out.push_str("requires_hitl: true\n");
    out.push_str("hitl_status: pending\n");
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n\n", title));
    out.push_str("## Purpose\n\n");
    out.push_str("- Split epic or roadmap-sized work into atomic work-item candidates.\n");
    out.push_str(
        "- Keep this artifact local until a human confirms which candidates should publish.\n",
    );
    out.push_str("- Atomic candidates must have one visible outcome, one main workspace/module, and one verification gate.\n\n");

    out.push_str("## Source Work Items That Need Atomization\n\n");
    let mut any_source = false;
    for issue in issues {
        if issue.issue_type == IssueType::Epic || looks_too_large_for_atomic_wi(issue) {
            any_source = true;
            out.push_str(&format!(
                "- {} — {} ({})\n",
                issue_ref(issue),
                issue.title.trim(),
                if issue.issue_type == IssueType::Epic {
                    "epic"
                } else {
                    "roadmap-sized"
                },
            ));
        }
    }
    if !any_source {
        out.push_str("- none\n");
    }

    out.push_str("\n## Atomic WI Candidates\n\n");
    out.push_str("| Source | Candidate title | Capability gap | Verification |\n");
    out.push_str("|--------|-----------------|----------------|--------------|\n");
    if candidates.is_empty() {
        out.push_str("| none | - | - | - |\n");
    } else {
        for candidate in candidates {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                candidate.source_ref,
                markdown_table_cell(&candidate.title),
                markdown_table_cell(&candidate.capability_gap),
                markdown_table_cell(&candidate.verification)
            ));
        }
    }

    out.push_str("\n## Required Human Confirmation\n\n");
    out.push_str("- Choose which candidates become local `aw wi draft` artifacts.\n");
    out.push_str("- Rewrite generic candidates into concrete titles before publishing.\n");
    out.push_str(
        "- Do not publish tracker changes from this artifact without human confirmation.\n",
    );
    out
}

fn render_prioritize_plan(
    project: &str,
    title: &str,
    backend_name: &str,
    lanes: &PrioritizeLanes,
    issues: &[Issue],
) -> String {
    let groups = group_issues_for_epicize(issues);
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("draft: true\n");
    out.push_str("kind: prioritize\n");
    out.push_str(&format!("project: {}\n", yaml_quote(project)));
    out.push_str(&format!("title: {}\n", yaml_quote(title)));
    out.push_str(&format!("backend: {}\n", yaml_quote(backend_name)));
    out.push_str(&format!("issue_count: {}\n", issues.len()));
    out.push_str(&format!("epic_count: {}\n", groups.existing_epics.len()));
    out.push_str(&format!("ready_now_count: {}\n", lanes.ready_now.len()));
    out.push_str(&format!(
        "blocked_by_dependency_count: {}\n",
        lanes.blocked_by_dependency.len()
    ));
    out.push_str(&format!(
        "needs_atomize_count: {}\n",
        lanes.needs_atomize.len()
    ));
    out.push_str(&format!(
        "needs_triage_count: {}\n",
        lanes.needs_triage.len()
    ));
    out.push_str(&format!("deferred_count: {}\n", lanes.deferred.len()));
    out.push_str("requires_hitl: true\n");
    out.push_str("hitl_status: pending\n");
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n\n", title));
    out.push_str("## Purpose\n\n");
    out.push_str("- Re-rank issue backlog by priority, dependency, and readiness.\n");
    out.push_str("- Identify ready work, blocked dependencies, atomization needs, triage blockers, and deferred work before tracker updates.\n");
    out.push_str("- Keep this artifact local until a human confirms the proposed ordering.\n\n");

    push_prioritize_lane(&mut out, "Ready Now", &lanes.ready_now);
    push_prioritize_lane(
        &mut out,
        "Blocked By Dependency",
        &lanes.blocked_by_dependency,
    );
    push_prioritize_lane(&mut out, "Needs Atomize", &lanes.needs_atomize);
    push_prioritize_lane(&mut out, "Needs Triage", &lanes.needs_triage);
    push_prioritize_lane(&mut out, "Deferred", &lanes.deferred);

    out.push_str("\n## Priority Confirmation Matrix\n\n");
    out.push_str("| Work item | Current priority | Proposed priority | Reason |\n");
    out.push_str("|-----------|------------------|-------------------|--------|\n");
    if issues.is_empty() {
        out.push_str("| none | - | - | - |\n");
    } else {
        for issue in issues {
            out.push_str(&format!(
                "| {} | {} | TBD | Human confirmation required |\n",
                issue_ref(issue),
                priority_label(issue)
            ));
        }
    }

    out.push_str("\n## Required HITL Brief\n\n");
    out.push_str(
        "This priority draft requires human confirmation before publishing tracker changes.\n\n",
    );
    out.push_str("- Reorder ready work only when dependency or urgency overrides deterministic priority ordering.\n");
    out.push_str(
        "- Keep dependency-blocked work out of the ready lane until the blocker closes.\n",
    );
    out.push_str(
        "- Recommend concrete priority label changes in the matrix with one short reason each.\n",
    );
    out.push_str(
        "- Do not publish tracker changes from this artifact without human confirmation.\n",
    );
    out
}

fn push_prioritize_lane(out: &mut String, title: &str, issues: &[Issue]) {
    out.push_str(&format!("## {}\n\n", title));
    if issues.is_empty() {
        out.push_str("- none\n\n");
        return;
    }
    for issue in issues {
        out.push_str(&issue_line(issue));
        out.push('\n');
    }
    out.push('\n');
}

fn write_planning_artifact(
    project_root: &Path,
    project: &str,
    bucket: &str,
    title: &str,
    output: Option<&Path>,
    body: &str,
) -> Result<PathBuf> {
    let path = if let Some(path) = output {
        ensure_planning_output_path_is_explicit(project_root, project, bucket, path)?;
        path.to_path_buf()
    } else {
        let stamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let dir = crate::shared::workspace::workitems_path(project_root)
            .join(project)
            .join(bucket);
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create {}", dir.display()))?;
        dir.join(format!("{}-{}.md", stamp, planning_slug(title)))
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    write_file_atomically(&path, body)?;
    Ok(path)
}

fn ensure_planning_output_path_is_explicit(
    project_root: &Path,
    project: &str,
    bucket: &str,
    path: &Path,
) -> Result<()> {
    if !is_tmp_root_file(path) {
        return Ok(());
    }

    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("plan.md");
    let suggested = crate::shared::workspace::workitems_path(project_root)
        .join(project)
        .join(bucket)
        .join(filename);
    anyhow::bail!(
        "ambiguous planning artifact output `{}`; write WI planning artifacts under \
         /tmp/aw/workspaces/<workspace>/workitems/<project>/<kind>/ so agents can discover and confirm them. Use `{}`.",
        path.display(),
        suggested.display()
    );
}

fn is_tmp_root_file(path: &Path) -> bool {
    matches!(
        path.parent().and_then(|parent| parent.to_str()),
        Some("/tmp" | "/private/tmp")
    )
}

fn planning_slug(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for c in title.chars().flat_map(|c| c.to_lowercase()) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "plan".to_string()
    } else {
        trimmed.chars().take(60).collect()
    }
}

// ---------------------------------------------------------------------------
// Formatters
// ---------------------------------------------------------------------------

fn print_table(issues: &[Issue], backend_name: &str) {
    if issues.is_empty() {
        println!("No issues found");
        return;
    }

    println!("{} issue(s) from {}", issues.len(), backend_name);
    println!();

    for issue in issues {
        let type_tag = format!("[{}]", issue.issue_type.as_str());
        let state_tag = colorize_state(issue.state);
        let id_part = issue
            .github_id
            .or(issue.gitlab_id)
            .map(|n| format!("#{}", n))
            .unwrap_or_else(|| "(draft)".to_string());

        println!(
            "  {} {} {} {}",
            type_tag.bright_black(),
            state_tag,
            id_part.cyan(),
            issue.title.trim()
        );
        println!("    {}", issue.slug.dimmed());
    }
}

fn print_detail(issue: &Issue) {
    let state_tag = colorize_state(issue.state);
    let type_tag = format!("[{}]", issue.issue_type.as_str()).bright_black();
    let id = issue
        .github_id
        .or(issue.gitlab_id)
        .map(|n| format!("#{}", n))
        .unwrap_or_else(|| "(draft)".to_string());

    println!(
        "{} {} {} {}",
        type_tag,
        state_tag,
        id.cyan(),
        issue.title.bold()
    );
    if let Some(url) = &issue.url {
        println!("{}", url.dimmed());
    }
    println!();
    println!("{}: {}", "slug".bright_black(), issue.slug);
    if let Some(a) = &issue.author {
        println!("{}: {}", "author".bright_black(), a);
    }
    if !issue.labels.is_empty() {
        println!("{}: {}", "labels".bright_black(), issue.labels.join(", "));
    }
    if !issue.related.is_empty() {
        println!("{}: {}", "related".bright_black(), issue.related.join(", "));
    }
    if !issue.implements.is_empty() {
        println!(
            "{}: {}",
            "implements".bright_black(),
            issue.implements.join(", ")
        );
    }
    if let (Some(c), Some(u)) = (&issue.created_at, &issue.updated_at) {
        println!("{}: {} (updated {})", "created".bright_black(), c, u);
    }
    println!();
    println!("{}", "---".dimmed());
    println!("{}", issue.body);
}

fn colorize_state(state: IssueState) -> colored::ColoredString {
    match state {
        IssueState::Open => "open".green(),
        IssueState::Closed => "closed".red(),
        IssueState::Draft => "draft".yellow(),
    }
}

// ---------------------------------------------------------------------------
// Enrich — fill Reference Context via agent
// ---------------------------------------------------------------------------

// REQ: structured-issue#R7
///
// Prints the issue slug and a brief telling mainthread to fill / update the
// Reference Context section directly (post-Phase-2 mainthread-only model —
// no subagent dispatch). This subcommand exists so
// `aw wi enrich <slug>` is a valid CLI entry point for scripting and
// cron-driven workflows.
async fn run_enrich(args: EnrichArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let backend = make_backend("local", &project_root, None, None)
        .context("Failed to create local backend")?;

    let issue = backend
        .get(&args.slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Issue '{}' not found in local store", args.slug))?;

    let has_ref_ctx = issue.body.contains("## Reference Context");

    let result = serde_json::json!({
        "slug": issue.slug,
        "title": issue.title,
        "has_reference_context": has_ref_ctx,
        "action": if has_ref_ctx { "update" } else { "create" },
        "message": format!(
            "Mainthread: {} the ## Reference Context section in the temp issue working copy for {}.",
            if has_ref_ctx { "update" } else { "fill" },
            args.slug,
        ),
    });

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

// ---------------------------------------------------------------------------
// Validate — deterministic WI quality gate
// ---------------------------------------------------------------------------

// Validate work-item quality and commit the pending current-checkout change
// and emit the next-step dispatch envelope.
///
// Fill applies the authored payload and validate deterministically admits or
// rejects the bounded WI. Product ambiguity is HITL; there is no WI semantic
// review, revise, or arbitration state machine.
///
// When invoked outside a lifecycle branch (legacy CLI use, no pending changes), it
// behaves as before — quality check + auto-promote draft→open + text/json
// output, no commit, no envelope.
// Quality checks cover R-id, scope, spec-plan, and ambiguity rules.
async fn run_validate(mut args: ValidateArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let requested_slug = args.slug.clone();
    let (kind, repo, host) = resolve_validate_backend(args.repo.clone(), &project_root)?;
    let selected_backend = make_backend(&kind, &project_root, repo, host)
        .context("Failed to create backend for validate")?;
    let selected_issue = selected_backend.get(&requested_slug).await?;
    if let Some(issue) = &selected_issue {
        args.slug = issue.slug.clone();
    }

    let issue = match selected_issue {
        Some(i) => i,
        None => anyhow::bail!("Issue '{}' not found", requested_slug),
    };

    // Soft check on every validate: warn if app/lib label count doesn't
    // match the one-issue-one-project convention (epics excepted).
    check_project_labels(&project_root, &issue.labels, issue.issue_type, &issue.slug);

    run_validate_legacy(&args, selected_backend.as_ref(), &issue).await
}

fn resolve_validate_backend(
    repo_override: Option<String>,
    project_root: &std::path::Path,
) -> Result<(String, Option<String>, Option<String>)> {
    match resolve_backend(repo_override, project_root) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(("local".to_string(), None, None)),
    }
}

// Global quality check + draft→open promotion. Used when
// validate is invoked against the main repo (no worktree exists for this
// slug). Preserved so older `/aw:issue update <slug>` flows still work.
async fn run_validate_legacy(
    args: &ValidateArgs,
    backend: &dyn IssueBackend,
    issue: &Issue,
) -> Result<()> {
    let quality = crate::services::issue_parser::validate_issue_quality(&issue.body);
    let mut quality_errors = quality.errors.clone();
    quality_errors.extend(validate_planning_alignment(issue));

    if !quality_errors.is_empty() {
        let patch = IssuePatch {
            validation_errors: Some(quality_errors.clone()),
            ..Default::default()
        };
        backend.update(&args.slug, &patch).await?;

        if args.human {
            eprintln!("Validation failed for '{}':", args.slug);
            for err in &quality_errors {
                eprintln!("  - {}", err);
            }
            std::process::exit(2);
        } else {
            let result = serde_json::json!({
                "passed": false,
                "errors": quality_errors,
                "state_promoted": false,
            });
            if args.pretty {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("{}", serde_json::to_string(&result)?);
            }
            std::process::exit(2);
        }
    }

    let was_draft = issue.state == IssueState::Draft;
    if was_draft {
        let patch = IssuePatch {
            state: Some(IssueState::Open),
            validation_errors: Some(vec![]),
            ..Default::default()
        };
        backend.update(&args.slug, &patch).await?;
    }

    if args.human {
        if was_draft {
            println!(
                "Validation passed. Issue '{}' promoted: draft -> open",
                args.slug
            );
        } else {
            println!("Validation passed for '{}'.", args.slug);
        }
    } else {
        let result = serde_json::json!({
            "passed": true,
            "errors": [],
            "state_promoted": was_draft,
            "new_state": if was_draft { "open" } else { issue.state.as_str() },
        });
        if args.pretty {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{}", serde_json::to_string(&result)?);
        }
    }
    Ok(())
}

// Placeholder substrings that author skeletons use to mark sections
// they haven't filled yet. `validate_section_format` rejects any section
// whose content contains one of these markers — this prevents the
// linear authoring flow from advancing past a section that was structurally well-formed
// but semantically empty.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/issue-validator-placeholder-rejection.md#R5
const PLACEHOLDER_MARKERS: &[&str] = &["(fill)", "(replace-this)"];

// Per-section format check used by `validate` after each Fill-* milestone.
// Returns an empty vec on pass, or one error per problem.
///
// Mirrors the per-section rules in
// `crate::services::issue_parser::validate_issue_quality` but scoped to a
// single section so intermediate Fill stages don't fail just because later
// sections aren't filled yet.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-wi-crrr-removal.md#scenarios
// @spec apps/agentic-workflow/tech-design/surface/specs/issue-validator-placeholder-rejection.md
fn validate_section_format(body: &str, section: crate::issues::IssueSection) -> Vec<String> {
    use crate::issues::IssueSection;
    let sections = split_body_by_h2(body);
    let key = section.heading();
    let content: String = sections
        .iter()
        .find(|(h, _)| h == key)
        .map(|(_, c)| c.clone())
        .unwrap_or_default();

    if content.trim().is_empty() {
        return vec![format!("section '{}' missing or empty", key)];
    }

    // R1: reject any placeholder marker before per-section dispatch so
    // structurally-well-formed-but-empty content is surfaced as the
    // proximate cause rather than masked as a downstream structural error.
    for marker in PLACEHOLDER_MARKERS {
        if content.contains(marker) {
            return vec![format!(
                "section '{}' contains '{}' placeholder; replace with real content",
                key, marker
            )];
        }
    }

    match section {
        IssueSection::Problem => Vec::new(),
        IssueSection::Requirements => {
            let mut errors = Vec::new();
            let mut has_items = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
                    continue;
                }
                has_items = true;
                let item = trimmed.trim_start_matches("- ").trim_start_matches("* ");
                let id_ok = item.strip_prefix('R').and_then(|rest| {
                    let colon = rest.find(':')?;
                    let num = &rest[..colon];
                    if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) {
                        Some(())
                    } else {
                        None
                    }
                });
                if id_ok.is_none() {
                    let preview: String = item.chars().take(60).collect();
                    errors.push(format!("requirement missing R-id format: '{}'", preview));
                }
                let lower = item.to_ascii_lowercase();
                for ambiguous in &["tbd", "todo", "maybe", "unclear", "uncertain"] {
                    if lower.contains(ambiguous) {
                        let preview: String = item.chars().take(60).collect();
                        errors.push(format!(
                            "ambiguous requirement contains '{}': '{}'",
                            ambiguous, preview
                        ));
                    }
                }
            }
            if !has_items {
                errors.push("Requirements section is empty".to_string());
            }
            errors
        }
        IssueSection::Scope => {
            // R2: require both '### In Scope' and '### Out of Scope' headings.
            let lower = content.to_ascii_lowercase();
            let has_in = lower.contains("### in scope") || lower.contains("### in-scope");
            let has_out = lower.contains("### out of scope") || lower.contains("### out-of-scope");
            let mut errors = Vec::new();
            if !has_in {
                errors.push("Scope missing '### In Scope' sub-section".to_string());
            }
            if !has_out {
                errors.push("Scope missing '### Out of Scope' sub-section".to_string());
            }
            errors
        }
        IssueSection::ReferenceContext => {
            // R3: require both table headings AND each table must have at
            // least one row whose first cell is real content (not placeholder
            // and not a separator/header).
            let mut errors = Vec::new();
            if !content.contains("### Related Specs") {
                errors.push("Reference Context missing '### Related Specs' table".to_string());
            }
            if !content.contains("### Spec Plan") {
                errors.push("Reference Context missing '### Spec Plan' table".to_string());
            }
            // Extract data rows under each subsection heading and verify at
            // least one has a non-placeholder first cell.
            for (heading, label) in [
                ("### Related Specs", "Related Specs"),
                ("### Spec Plan", "Spec Plan"),
            ] {
                if let Some(start) = content.find(heading) {
                    let after = &content[start + heading.len()..];
                    let block = match after.find("\n### ") {
                        Some(end) => &after[..end],
                        None => after,
                    };
                    let mut saw_real = false;
                    for line in block.lines() {
                        let trimmed = line.trim();
                        if !trimmed.starts_with('|') {
                            continue;
                        }
                        // Skip header / separator rows.
                        if trimmed.contains("---") {
                            continue;
                        }
                        let first = trimmed
                            .trim_start_matches('|')
                            .split('|')
                            .next()
                            .unwrap_or("")
                            .trim();
                        if first.is_empty() {
                            continue;
                        }
                        let lower = first.to_ascii_lowercase();
                        // Skip the column header row.
                        if lower == "spec" || lower == "spec id" {
                            continue;
                        }
                        saw_real = true;
                        break;
                    }
                    if !saw_real {
                        errors.push(format!(
                            "Reference Context '{}' table has no real rows",
                            label
                        ));
                    }
                }
            }
            errors
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issues::IssueSection;

    fn body_with(scope: &str, ref_ctx: &str) -> String {
        format!(
            "## Problem\n\nP1\n\n## Requirements\n\n- R1: real content\n\n## Scope\n\n{}\n\n## Reference Context\n\n{}\n",
            scope, ref_ctx
        )
    }

    fn capability_plan_review_fixture(
        backing: Option<&str>,
        author: &str,
        reviewer: &str,
        decision: CapabilityPlanReviewDecision,
    ) -> (tempfile::TempDir, CapabilityPlanReviewRecord) {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("apps/jet")).unwrap();
        std::fs::write(
            tmp.path().join("aw.toml"),
            r#"[[projects]]
name = "jet"
path = "apps/jet"
label = "app:jet"
"#,
        )
        .unwrap();
        if let Some(backing) = backing {
            std::fs::write(
                tmp.path().join("apps/jet/aw.toml"),
                format!("capability_plan_review_backing = \"{backing}\"\n"),
            )
            .unwrap();
        }
        let plan_path = crate::shared::workspace::workitems_path(tmp.path())
            .join("jet")
            .join("capability-plan")
            .join("plan.md");
        std::fs::create_dir_all(plan_path.parent().unwrap()).unwrap();
        let manifest_path = capability_plan_sidecar_path(&plan_path, "manifest.json");
        let plan_body = "---\nkind: capability_plan\n---\n# plan\n";
        let manifest = CapabilityPlanManifest {
            version: CAPABILITY_PLAN_REVIEW_VERSION,
            project: "jet".to_string(),
            cap_path: "apps/jet/CAPABILITIES.md".to_string(),
            candidates: Vec::new(),
            reconciliations: Vec::new(),
        };
        let manifest_body = format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap());
        std::fs::write(&plan_path, plan_body).unwrap();
        std::fs::write(&manifest_path, &manifest_body).unwrap();
        let source_digest = capability_plan_source_digest(plan_body, &manifest_body);
        let author_record = CapabilityPlanAuthorRecord {
            version: CAPABILITY_PLAN_REVIEW_VERSION,
            project: "jet".to_string(),
            source_digest: source_digest.clone(),
            author: author.to_string(),
            recorded_at: "2026-07-21T00:00:00Z".to_string(),
        };
        std::fs::write(
            capability_plan_sidecar_path(&plan_path, "author.json"),
            serde_json::to_string_pretty(&author_record).unwrap(),
        )
        .unwrap();
        let accepted = decision == CapabilityPlanReviewDecision::Accepted;
        let record = CapabilityPlanReviewRecord {
            version: CAPABILITY_PLAN_REVIEW_VERSION,
            project: "jet".to_string(),
            plan_path: plan_path.display().to_string(),
            manifest_path: manifest_path.display().to_string(),
            source_digest,
            decision,
            reviewer_kind: "agent".to_string(),
            reviewed_by: reviewer.to_string(),
            reviewed_at: String::new(),
            summary: "Independent semantic review completed.".to_string(),
            checklist: CapabilityPlanReviewChecklist {
                capability_claim_coverage: accepted,
                bounded_candidates: accepted,
                tracker_reconciliation: accepted,
                verification_specific: accepted,
                no_duplicate_wis: accepted,
                publication_safe: accepted,
            },
            findings: if accepted {
                Vec::new()
            } else {
                vec!["Candidate scope is not bounded.".to_string()]
            },
        };
        (tmp, record)
    }

    fn test_issue_with_phase(phase: Option<&str>) -> Issue {
        Issue {
            issue_type: IssueType::Bug,
            title: "demo".to_string(),
            state: IssueState::Open,
            id: None,
            github_id: Some(1234),
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec!["type:bug".to_string(), "app:agentic-workflow".to_string()],
            created_at: None,
            updated_at: None,
            slug: "1234".to_string(),
            body: body_with(
                "### In Scope\n- real scope item\n\n### Out of Scope\n- explicit exclusion",
                "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| foo.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| foo | create | foo.md |",
            ),
            related: vec![],
            implements: vec![],
            phase: phase.map(str::to_string),
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: vec![],
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    fn planning_issue(
        issue_type: IssueType,
        title: &str,
        priority: Option<&str>,
        id: u64,
    ) -> Issue {
        let mut issue = test_issue_with_phase(None);
        issue.issue_type = issue_type;
        issue.title = title.to_string();
        issue.github_id = Some(id);
        issue.slug = id.to_string();
        issue.body = format!(
            "## Problem\n\n{title}\n\n## Capability Alignment\n\nCapability: Config correctness\nCapability Gap: parser diagnostics are incomplete\nProgress Evidence: validation fixture covers the behavior\n\n## Requirements\n\n- R1: Deliver {title}.\n\n## Scope\n\n### In Scope\n- {title}.\n\n### Out of Scope\n- Unrelated work.\n\n## Acceptance Criteria\n\n- AC1: {title} is implemented and verified.\n\n## Reference Context\n\n### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| foo.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| foo | update | foo.md |\n"
        );
        issue.labels = vec![
            format!("type:{}", issue_type.as_str()),
            "app:agentic-workflow".to_string(),
        ];
        if let Some(priority) = priority {
            issue.labels.push(format!("priority:{}", priority));
        }
        issue
    }

    #[test]
    fn update_closed_patch_clears_active_workflow_state() {
        let mut issue = test_issue_with_phase(Some("td_created"));
        issue.labels.extend([
            "phase:td_inited".to_string(),
            "score:locked".to_string(),
            "score:lock:td".to_string(),
        ]);
        let projection = crate::cli::workflow_guard::WorkflowProjection {
            version: 1,
            issue_id: issue.slug.clone(),
            locked: true,
            owner: Some("td".to_string()),
            active_phase: Some("td_created".to_string()),
            expected_command: Some("aw td validate 1234".to_string()),
            ..Default::default()
        };
        issue.body = crate::cli::workflow_guard::upsert_projection("Body", &projection).unwrap();
        let args = UpdateArgs {
            id: issue.slug.clone(),
            title: None,
            state: Some(StateFilter::Closed),
            add_labels: vec![],
            remove_labels: vec![],
            body_file: None,
            push: false,
            json: false,
            repo: None,
        };

        let patch = build_update_patch(&args, None, Some(&issue)).unwrap();
        assert!(patch.clear_phase);
        assert!(patch.clear_transient);
        assert_eq!(patch.ship_status, Some(ShipStatus::Rejected));

        let mut updated = issue;
        patch.apply(&mut updated);
        let projection = crate::cli::workflow_guard::parse_projection(&updated.body).unwrap();
        assert_eq!(updated.phase, None);
        assert!(!projection.locked);
        assert_eq!(projection.active_phase, None);
        assert_eq!(projection.expected_command, None);
        assert!(!updated
            .labels
            .iter()
            .any(|label| label.starts_with("phase:")));
        assert!(!updated.labels.iter().any(|label| label == "score:locked"));
        assert!(updated.labels.iter().any(|label| label == "ship:rejected"));
    }

    #[test]
    fn epicize_groups_requirements_into_candidates() {
        let issues = vec![
            planning_issue(IssueType::Bug, "urgent", Some("p1"), 1),
            planning_issue(IssueType::Enhancement, "new capability", Some("p2"), 2),
            planning_issue(IssueType::Refactor, "cleanup", None, 3),
            planning_issue(IssueType::Test, "coverage", None, 4),
            planning_issue(IssueType::Epic, "existing phase", None, 5),
        ];
        let groups = group_issues_for_epicize(&issues);
        assert_eq!(groups.existing_epics.len(), 1);
        assert_eq!(groups.urgent_fixes.len(), 1);
        assert_eq!(groups.capability_work.len(), 1);
        assert_eq!(groups.maintenance.len(), 1);
        assert_eq!(groups.quality.len(), 1);
    }

    #[test]
    fn epicize_artifact_requires_hitl() {
        let issues = vec![planning_issue(
            IssueType::Enhancement,
            "new capability",
            Some("p1"),
            1,
        )];
        let body = render_epicize_plan("score", "Score phase", "github", &issues, None);
        assert!(body.contains("requires_hitl: true"));
        assert!(body.contains("hitl_status: pending"));
        assert!(body.contains("## Required HITL Brief"));
    }

    #[test]
    fn epicize_artifact_includes_markdown_capability_roots() {
        let cap_body = r#"# demo

## Package Manager

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| package-manager | #3779 | auditing | Replace package manager flows. | smoke | apps/jet/validation/pkg-manager.toml |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Lockfile parity | epic | #3779 | partial | planned | smoke | apps/jet/validation/pkg-manager.toml |
"#;
        let document =
            crate::cli::capability::parse_capability_document(cap_body, Path::new("README.md"))
                .unwrap();
        let body = render_epicize_plan("jet", "Jet epics", "github", &[], Some(&document));
        assert!(body.contains("## Capability Epic Candidates"));
        assert!(body.contains("| Package Manager | epic | package-manager | #3779 | auditing |"));
        assert!(
            body.contains("| Lockfile parity | subepic | package-manager | #3779 | in_progress |")
        );
        assert!(body.contains("Atomic change WIs are created by `aw wi atomize`"));
    }

    #[test]
    fn prioritize_artifact_requires_hitl_and_orders_all_layers() {
        let issues = vec![
            planning_issue(IssueType::Epic, "phase", Some("p1"), 3),
            planning_issue(IssueType::Bug, "urgent", Some("p0"), 1),
            planning_issue(IssueType::Enhancement, "capability", Some("p2"), 2),
        ];
        let lanes = prioritize_lanes(&issues);
        let body = render_prioritize_plan("score", "Score priorities", "github", &lanes, &issues);
        assert!(body.contains("kind: prioritize"));
        assert!(body.contains("requires_hitl: true"));
        assert!(body.contains("## Ready Now"));
        assert!(body.contains("## Blocked By Dependency"));
        assert!(body.contains("## Needs Atomize"));
        assert!(body.contains("## Needs Triage"));
        assert!(body.contains("## Priority Confirmation Matrix"));
    }

    #[test]
    fn prioritize_lanes_put_bounded_bug_in_ready_now() {
        let issues = vec![planning_issue(IssueType::Bug, "urgent", Some("p0"), 1)];
        let lanes = prioritize_lanes(&issues);
        assert_eq!(lanes.ready_now.len(), 1);
        assert_eq!(lanes.ready_now[0].title, "urgent");
        assert!(lanes.needs_atomize.is_empty());
        assert!(lanes.needs_triage.is_empty());
    }

    #[test]
    fn prioritize_lanes_block_open_dependency() {
        let blocker = planning_issue(IssueType::Bug, "blocker", Some("p0"), 1);
        let mut dependent = planning_issue(IssueType::Enhancement, "dependent", Some("p1"), 2);
        dependent
            .body
            .push_str("\n## Dependencies\n\n- Depends on #1 before implementation.\n");
        let lanes = prioritize_lanes(&[blocker, dependent]);
        assert!(lanes.ready_now.iter().any(|issue| issue.title == "blocker"));
        assert!(lanes
            .blocked_by_dependency
            .iter()
            .any(|issue| issue.title == "dependent"));
    }

    #[test]
    fn wi_remove_agent_estimate_prioritize_output_omits_estimate_fields() {
        let issues = vec![planning_issue(IssueType::Bug, "ready bug", Some("p1"), 9)];
        let lanes = prioritize_lanes(&issues);
        let body = render_prioritize_plan("score", "Score priorities", "github", &lanes, &issues);
        assert!(body.contains("## Ready Now"));
        assert!(!body.contains("Agent Estimate"));
        assert!(!body.contains("agent_minutes"));
        assert!(!body.contains("human_attention"));
    }

    #[test]
    fn prioritize_lanes_send_split_required_to_needs_atomize() {
        let issues = vec![
            planning_issue(
                IssueType::Enhancement,
                "Build Google Maps in Rust",
                Some("p0"),
                1,
            ),
            planning_issue(IssueType::Bug, "small bug", Some("p1"), 2),
        ];
        let lanes = prioritize_lanes(&issues);
        assert!(lanes
            .needs_atomize
            .iter()
            .any(|issue| issue.title.contains("Google Maps")));
        assert!(lanes
            .ready_now
            .iter()
            .any(|issue| issue.title == "small bug"));
    }

    #[test]
    fn capability_map_parser_reads_confirmed_table_and_health_note() {
        let body = r#"
# jet

## Capability Map

| Capability | Current State | Gaps | Active WI | Evidence |
|------------|---------------|------|-----------|----------|
| Package manager | Lockfile works | peer dep drift | #42 | README |
| Dev server | HMR works | none | none | tests |

## Project Health Note

Generator ownership is complete; package-manager roadmap remains open.

## Other
"#;
        let map = parse_capability_map(body).unwrap();
        assert_eq!(map.rows.len(), 2);
        assert_eq!(map.rows[0].capability, "Package manager");
        assert_eq!(map.rows[0].active_wi, "#42");
        assert!(map
            .health_note
            .as_deref()
            .unwrap()
            .contains("Generator ownership is complete"));
    }

    #[test]
    fn capability_plan_marks_unmatched_gaps_as_wi_candidates() {
        let map = CapabilityMap {
            capability_count: 2,
            rows: vec![
                CapabilityRow {
                    capability_id: "package-manager".to_string(),
                    capability: "Package manager".to_string(),
                    capability_type: "DeveloperTool".to_string(),
                    surfaces: "CLI: `jet install` - install dependencies".to_string(),
                    ec_dimensions: "behavior: `jet test` - package manager conformance<br>efficiency: `meter` - install profile".to_string(),
                    current_state: "Works for lockfile installs".to_string(),
                    gaps: "peer dependency roadmap missing".to_string(),
                    root_wi: "none".to_string(),
                    active_wi: "none".to_string(),
                    evidence: "README".to_string(),
                    claim_id: None,
                    claim_user_story: None,
                },
                CapabilityRow {
                    capability_id: "dev-server".to_string(),
                    capability: "Dev server".to_string(),
                    capability_type: "-".to_string(),
                    surfaces: "-".to_string(),
                    ec_dimensions: "-".to_string(),
                    current_state: "HMR works".to_string(),
                    gaps: "none".to_string(),
                    root_wi: "none".to_string(),
                    active_wi: "none".to_string(),
                    evidence: "tests".to_string(),
                    claim_id: None,
                    claim_user_story: None,
                },
            ],
            health_note: None,
        };
        let candidates = capability_wi_candidates(&map.rows, &[]);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].issue_type, "enhancement");
        let body = render_capability_wi_plan(
            "jet",
            "Jet capability plan",
            "github",
            Path::new("/repo/apps/jet/README.md"),
            &map,
            &[],
            &candidates,
            &BTreeMap::new(),
            &[],
            "either",
        );
        assert!(body.contains("kind: capability_plan"));
        assert!(body.contains("capability_count: 2"));
        assert!(body.contains("planning_row_count: 2"));
        assert!(body.contains("reconciliation_count: 0"));
        assert!(body.contains("## Confirmation Summary"));
        assert!(body.contains("| Package manager | 1 | none | epicize -> atomize | Close capability gap: Package manager |"));
        assert!(body.contains("| Capability | Type | Surfaces | EC Dimensions | Claim |"));
        assert!(body.contains("DeveloperTool"));
        assert!(body.contains("CLI: `jet install` - install dependencies"));
        assert!(body.contains("efficiency: `meter` - install profile"));
        assert!(body.contains("Close capability gap: Package manager"));
        assert!(body.contains("## Candidate WI Drafts"));
        assert!(body.contains("### Candidate 1: Close capability gap: Package manager"));
        assert!(body.contains("## Capability Alignment"));
        assert!(body.contains("Capability: `package-manager` (Package manager)"));
        assert!(body.contains("Capability Gap: `peer dependency roadmap missing`"));
        assert!(body.contains("## Acceptance Criteria"));
        assert!(body.contains("aw capability check --project jet"));
        assert!(body.contains("## Reference Context"));
        assert!(body.contains("## Recommended CLI Sequence"));
        assert!(body.contains("Accepted independent review authorizes publication"));

        let warned = render_capability_wi_plan(
            "jet",
            "Jet capability plan",
            "unavailable",
            Path::new("/repo/apps/jet/README.md"),
            &map,
            &[],
            &candidates,
            &BTreeMap::new(),
            &["issue inventory unavailable: gh auth missing".to_string()],
            "either",
        );
        assert!(warned.contains("warnings:"));
        assert!(warned.contains("## Source"));
        assert!(warned.contains("### Planning Warnings"));
        assert!(warned.contains("issue inventory unavailable: gh auth missing"));
    }

    #[test]
    fn capability_plan_claim_under_open_epic_becomes_bounded_child_candidate() {
        let row = CapabilityRow {
            capability_id: "cli-interface".to_string(),
            capability: "CLI Interface".to_string(),
            capability_type: "RuntimeTool".to_string(),
            surfaces: "CLI: `defer`".to_string(),
            ec_dimensions: "behavior: `cargo test -p defer --test cli_contract`".to_string(),
            current_state: "CLI exists".to_string(),
            gaps: "claim defer-cli-convention: CLI claim needs primary TD linkage".to_string(),
            root_wi: "#766".to_string(),
            active_wi: "#766".to_string(),
            evidence: "claim gate: cargo test -p defer --test cli_contract".to_string(),
            claim_id: Some("defer-cli-convention".to_string()),
            claim_user_story: Some("As an agent, I need a stable CLI.".to_string()),
        };
        let epic = planning_issue(
            IssueType::Epic,
            "defer: delayed task service",
            Some("p1"),
            766,
        );

        let candidates = capability_wi_candidates(&[row], &[epic]);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].parent_wi_refs, vec!["#766"]);
        assert_eq!(
            candidates[0].first_gate,
            "cargo test -p defer --test cli_contract"
        );
        let body = capability_candidate_wi_body(
            "defer",
            Path::new("apps/defer/CAPABILITIES.md"),
            &candidates[0],
        );
        assert!(body.contains("Capability: `cli-interface`"));
        assert!(body.contains("Capability Gap: `defer-cli-convention`"));
        assert!(body.contains("Parent WI: #766"));
        assert!(body.contains("### Verification Gate"));
        assert!(body.contains("`cargo test -p defer --test cli_contract`"));
        assert!(body.contains("Expected Result:"));
        assert!(body.contains("observes `defer cli convention`"));
        assert!(body.contains("linkage or prose alone is not closure"));
        assert!(body.contains("a documentation-only artifact is insufficient"));
        assert!(body.contains("does not close this WI"));
    }

    #[test]
    fn capability_evidence_command_keeps_all_same_project_test_targets() {
        assert_eq!(
            capability_evidence_command(
                "apps/relay/tests/deploy_cli.rs; apps/relay/tests/spec_cli.rs"
            )
            .as_deref(),
            Some("cargo test -p relay --test deploy_cli --test spec_cli")
        );
    }

    #[test]
    fn capability_plan_deduplicates_one_claim_across_capabilities() {
        let first = CapabilityRow {
            capability_id: "competitor-feature-parity".to_string(),
            capability: "Competitive Broker Feature Parity".to_string(),
            capability_type: "RuntimeTool".to_string(),
            surfaces: "Rust API: Relay".to_string(),
            ec_dimensions: "behavior: `cargo test -p relay --test raft_core`".to_string(),
            current_state: "Raft converges".to_string(),
            gaps: "claim in-process-raft-convergence: primary evidence required".to_string(),
            root_wi: "#108".to_string(),
            active_wi: "#108".to_string(),
            evidence: "apps/relay/tests/raft_core.rs".to_string(),
            claim_id: Some("in-process-raft-convergence".to_string()),
            claim_user_story: None,
        };
        let mut second = first.clone();
        second.capability_id = "raft-ha".to_string();
        second.capability = "Raft HA".to_string();
        second.root_wi = "#1207".to_string();
        second.active_wi = "#1207".to_string();

        let candidates = capability_wi_candidates(&[first, second], &[]);

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].claim_id.as_deref(),
            Some("in-process-raft-convergence")
        );
        assert_eq!(
            candidates[0].related_capabilities,
            vec!["raft-ha (Raft HA)"]
        );
        assert_eq!(
            candidates[0].first_gate,
            "cargo test -p relay --test raft_core"
        );
    }

    #[test]
    fn capability_plan_deduplicates_claim_against_explicit_open_wi_ref() {
        let row = CapabilityRow {
            capability_id: "work-queue-lifecycle".to_string(),
            capability: "Work Queue Lifecycle".to_string(),
            capability_type: "RuntimeTool".to_string(),
            surfaces: "HTTP: consume".to_string(),
            ec_dimensions: "behavior: `cargo test -p relay --test work_queue_api`".to_string(),
            current_state: "Lease lifecycle exists".to_string(),
            gaps: "claim lease-heartbeat-ack-lifecycle: committed lifecycle evidence required"
                .to_string(),
            root_wi: "none".to_string(),
            active_wi: "#1850".to_string(),
            evidence: "apps/relay/tests/work_queue_api.rs".to_string(),
            claim_id: Some("lease-heartbeat-ack-lifecycle".to_string()),
            claim_user_story: None,
        };
        let mut issue = planning_issue(
            IssueType::Bug,
            "relay: replicate lease lifecycle and fence consume ownership",
            Some("p0"),
            1850,
        );
        issue.body = "Lease grant, heartbeat, acknowledgement, expiry, reclaim, and the work-queue lifecycle must be committed and fenced."
            .to_string();

        let candidates = capability_wi_candidates(&[row], &[issue]);

        assert!(candidates.is_empty());
    }

    #[test]
    fn capability_plan_does_not_treat_non_epic_root_wi_as_claim_wi() {
        let row = CapabilityRow {
            capability_id: "competitor-feature-parity".to_string(),
            capability: "Competitive Broker Feature Parity".to_string(),
            capability_type: "RuntimeTool".to_string(),
            surfaces: "HTTP: publish and consume".to_string(),
            ec_dimensions: "behavior: `cargo test -p relay --test raft_core`".to_string(),
            current_state: "Feature breadth exists".to_string(),
            gaps: "claim durable-raft-hard-state-restore: persistence evidence required"
                .to_string(),
            root_wi: "#1850".to_string(),
            active_wi: "none".to_string(),
            evidence: "apps/relay/tests/raft_persistence.rs".to_string(),
            claim_id: Some("durable-raft-hard-state-restore".to_string()),
            claim_user_story: None,
        };
        let mut issue = planning_issue(
            IssueType::Bug,
            "relay: replicate lease lifecycle and fence consume ownership",
            Some("p0"),
            1850,
        );
        issue.body =
            "Lease, acknowledgement, expiry, and reclaim mutations are committed.".to_string();

        let candidates = capability_wi_candidates(&[row], &[issue]);

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].claim_id.as_deref(),
            Some("durable-raft-hard-state-restore")
        );
    }

    #[test]
    fn capability_plan_does_not_guess_open_wi_from_partial_claim_terms() {
        let row = CapabilityRow {
            capability_id: "cli-interface".to_string(),
            capability: "CLI Interface".to_string(),
            capability_type: "RuntimeTool".to_string(),
            surfaces: "CLI: relay".to_string(),
            ec_dimensions: "behavior: `cargo test -p relay --test raft_config`".to_string(),
            current_state: "Auto mode exists".to_string(),
            gaps: "claim auto-mode-raft-node-entrypoint: entrypoint evidence required".to_string(),
            root_wi: "#1207".to_string(),
            active_wi: "#1207".to_string(),
            evidence: "apps/relay/tests/raft_config.rs".to_string(),
            claim_id: Some("auto-mode-raft-node-entrypoint".to_string()),
            claim_user_story: None,
        };
        let mut issue = planning_issue(
            IssueType::Bug,
            "relay: replicate lease lifecycle and fence consume ownership",
            Some("p0"),
            1850,
        );
        issue.body = "The raft node must commit the lease lifecycle before delivery.".to_string();

        let candidates = capability_wi_candidates(&[row], &[issue]);

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].claim_id.as_deref(),
            Some("auto-mode-raft-node-entrypoint")
        );
    }

    #[test]
    fn capability_plan_publication_deduplicates_the_same_claim_only() {
        let mut issue = planning_issue(
            IssueType::Enhancement,
            "Implement one CLI claim",
            Some("p1"),
            812,
        );
        issue.body = "## Capability Alignment\n\nCapability: `cli-interface`\nCapability Gap: `defer-cli-convention`\n".to_string();
        let mut candidate = CapabilityCandidate {
            title: "Close capability claim: CLI / defer-cli-convention".to_string(),
            issue_type: "enhancement".to_string(),
            source_capability_id: "cli-interface".to_string(),
            source_capability: "CLI Interface".to_string(),
            related_capabilities: Vec::new(),
            claim_id: Some("defer-cli-convention".to_string()),
            capability_gap: "missing primary TD".to_string(),
            first_gate: "cargo test -p defer --test cli_contract".to_string(),
            expected_result: "The gate observes `defer cli convention` for `defer-cli-convention`"
                .to_string(),
            parent_wi_refs: vec!["#766".to_string()],
        };

        assert!(open_issue_serves_capability_candidate(&issue, &candidate));
        candidate.claim_id = Some("defer-cli-next-command".to_string());
        assert!(!open_issue_serves_capability_candidate(&issue, &candidate));
    }

    #[test]
    fn capability_plan_accepts_independent_agent_review_by_default() {
        let (tmp, record) = capability_plan_review_fixture(
            None,
            "author-agent",
            "reviewer-agent",
            CapabilityPlanReviewDecision::Accepted,
        );

        validate_capability_plan_review_record(tmp.path(), &record).unwrap();
    }

    #[test]
    fn capability_plan_rejects_same_agent_self_review() {
        let (tmp, record) = capability_plan_review_fixture(
            None,
            "same-agent",
            "same-agent",
            CapabilityPlanReviewDecision::Accepted,
        );

        let error = validate_capability_plan_review_record(tmp.path(), &record).unwrap_err();
        assert!(error.to_string().contains("not independent"));
    }

    #[test]
    fn capability_plan_human_only_policy_rejects_agent_evidence() {
        let (tmp, record) = capability_plan_review_fixture(
            Some("human"),
            "author-agent",
            "reviewer-agent",
            CapabilityPlanReviewDecision::Accepted,
        );

        let error = validate_capability_plan_review_record(tmp.path(), &record).unwrap_err();
        assert!(error.to_string().contains("human-only"));
    }

    #[test]
    fn capability_plan_needs_revision_requires_and_accepts_findings() {
        let (tmp, record) = capability_plan_review_fixture(
            None,
            "author-agent",
            "reviewer-agent",
            CapabilityPlanReviewDecision::NeedsRevision,
        );

        validate_capability_plan_review_record(tmp.path(), &record).unwrap();
    }

    #[test]
    fn capability_plan_review_is_bound_to_exact_digest() {
        let (tmp, record) = capability_plan_review_fixture(
            None,
            "author-agent",
            "reviewer-agent",
            CapabilityPlanReviewDecision::Accepted,
        );
        std::fs::write(&record.plan_path, "drifted plan").unwrap();

        let error = validate_capability_plan_review_record(tmp.path(), &record).unwrap_err();
        assert!(error.to_string().contains("stale"));
    }

    #[test]
    fn legacy_human_only_plan_does_not_resurrect_as_a_review_blocker() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = crate::shared::workspace::workitems_path(tmp.path())
            .join("jet")
            .join("capability-plan");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("20260720000000-legacy.md"),
            "---\nkind: capability_plan\nrequires_hitl: true\nhitl_status: pending\n---\n# legacy\n",
        )
        .unwrap();

        assert!(pending_capability_plan_review(tmp.path(), "jet").is_none());
    }

    #[test]
    fn capability_plan_summary_groups_stale_active_wi_refs_as_reconciliations() {
        let rows = vec![
            CapabilityRow {
                capability_id: "package-manager".to_string(),
                capability: "Package Manager".to_string(),
                capability_type: "DeveloperTool".to_string(),
                surfaces: "CLI: `jet install`".to_string(),
                ec_dimensions: "behavior: `jet test`".to_string(),
                current_state: "Install surface exists".to_string(),
                gaps: "claim package-manager-readiness: package readiness needs proof".to_string(),
                root_wi: "none".to_string(),
                active_wi: "#3779".to_string(),
                evidence: "claim gate: cargo test -p jet --lib pkg_manager".to_string(),
                claim_id: Some("package-manager-readiness".to_string()),
                claim_user_story: None,
            },
            CapabilityRow {
                capability_id: "package-manager".to_string(),
                capability: "Package Manager".to_string(),
                capability_type: "DeveloperTool".to_string(),
                surfaces: "CLI: `jet install`".to_string(),
                ec_dimensions: "behavior: `jet test`".to_string(),
                current_state: "Workspace support exists".to_string(),
                gaps: "claim package-manager-workspace-parity: workspace parity needs proof"
                    .to_string(),
                root_wi: "none".to_string(),
                active_wi: "3780".to_string(),
                evidence: "claim gate: cargo test -p jet --lib pkg_manager::workspace".to_string(),
                claim_id: Some("package-manager-workspace-parity".to_string()),
                claim_user_story: None,
            },
        ];

        let candidates = capability_wi_candidates(&rows, &[]);
        let mut resolved_wi_refs = BTreeMap::new();
        resolved_wi_refs.insert(
            "#3779".to_string(),
            CapabilityTrackerRefLookup {
                reference: "#3779".to_string(),
                status: "closed".to_string(),
                title: "jet package manager readiness".to_string(),
                labels: "app:jet, type:epic".to_string(),
                url: "https://github.example/issues/3779".to_string(),
            },
        );
        let reconciliations = capability_tracker_reconciliations(&rows, &[], &resolved_wi_refs);
        let summary = capability_plan_summary_rows(&rows, &[], &candidates);

        assert_eq!(candidates.len(), 2);
        assert_eq!(reconciliations.len(), 2);
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].capability, "Package Manager");
        assert_eq!(summary[0].candidate_count, 2);
        assert_eq!(summary[0].existing_wi_refs, vec!["#3779", "#3780"]);
        assert_eq!(summary[0].next_operator, "epicize -> atomize");
        assert_eq!(
            summary[0].first_action,
            "Close capability claim: Package Manager / package-manager-readiness"
        );

        let map = CapabilityMap {
            capability_count: 1,
            rows,
            health_note: None,
        };
        let body = render_capability_wi_plan(
            "jet",
            "Jet capability plan",
            "github",
            Path::new("/repo/apps/jet/README.md"),
            &map,
            &[],
            &candidates,
            &resolved_wi_refs,
            &[],
            "either",
        );
        assert!(body.contains("candidate_count: 2"));
        assert!(body.contains("reconciliation_count: 2"));
        assert!(body.contains("resolved_wi_ref_count: 1"));
        assert!(body.contains("## Existing WI Refs Not In Open Inventory"));
        assert!(body.contains("| Package Manager | package-manager-readiness | #3779 | #3779: closed - jet package manager readiness |"));
        assert!(body.contains("## Tracker WI Ref Lookups"));
        assert!(body.contains("| #3779 | closed | jet package manager readiness | app:jet, type:epic | https://github.example/issues/3779 |"));
        assert!(body.contains("## Candidate WI Drafts"));
        assert!(body.contains("Complete the digest-bound review payload"));
        assert!(body.contains("`aw goal capability --project jet --non-interactive`"));
    }

    #[test]
    fn capability_plan_reconciles_stale_root_wi_refs() {
        let row = CapabilityRow {
            capability_id: "py3-12-functional-parity".to_string(),
            capability: "C1. Py3.12 functional parity".to_string(),
            capability_type: "RuntimeTool".to_string(),
            surfaces: "CLI: `mamba test`".to_string(),
            ec_dimensions: "behavior: `cargo test -p mamba`".to_string(),
            current_state: "Root WI: #3331; Gate inventory: cargo test -p mamba".to_string(),
            gaps: "claim python-3-12-parity-gate: Python 3.12 parity gate".to_string(),
            root_wi: "#3331".to_string(),
            active_wi: "#31, #33".to_string(),
            evidence: "claim gate: cargo test -p mamba".to_string(),
            claim_id: Some("python-3-12-parity-gate".to_string()),
            claim_user_story: None,
        };
        let issues = vec![
            planning_issue(
                IssueType::Enhancement,
                "mamba: collections.abc mixin 方法合成",
                Some("p1"),
                31,
            ),
            planning_issue(
                IssueType::Enhancement,
                "mamba: 動態 dispatch 的 user-function kwargs binding",
                Some("p2"),
                33,
            ),
        ];
        let candidates = capability_wi_candidates(std::slice::from_ref(&row), &issues);
        let mut resolved_wi_refs = BTreeMap::new();
        resolved_wi_refs.insert(
            "#3331".to_string(),
            CapabilityTrackerRefLookup {
                reference: "#3331".to_string(),
                status: "not_found".to_string(),
                title: "-".to_string(),
                labels: "-".to_string(),
                url: "-".to_string(),
            },
        );
        let reconciliations = capability_tracker_reconciliations(
            std::slice::from_ref(&row),
            &issues,
            &resolved_wi_refs,
        );
        let summary =
            capability_plan_summary_rows(std::slice::from_ref(&row), &issues, &candidates);

        assert!(candidates.is_empty());
        assert_eq!(reconciliations.len(), 1);
        assert_eq!(reconciliations[0].active_wi, "#3331");
        assert_eq!(reconciliations[0].tracker_lookup, "#3331: not_found");
        assert_eq!(summary[0].existing_wi_refs, vec!["#3331", "#31", "#33"]);
        assert_eq!(summary[0].next_operator, "reconcile tracker ref");
        assert_eq!(summary[0].first_action, "Reconcile WI reference: #3331");
    }

    #[test]
    fn capability_wi_plan_command_preserves_cap_path_override() {
        let command = capability_wi_plan_command(
            "lumen",
            Some(Path::new("/tmp/aw/test/plan path/lumen README.md")),
        );

        assert_eq!(
            command,
            "aw wi plan --project lumen --cap-path '/tmp/aw/test/plan path/lumen README.md'"
        );
    }

    #[test]
    fn capability_matching_uses_active_wi_reference_before_creating_candidate() {
        let row = CapabilityRow {
            capability_id: "package-manager".to_string(),
            capability: "Package manager".to_string(),
            capability_type: "DeveloperTool".to_string(),
            surfaces: "CLI: `jet install`".to_string(),
            ec_dimensions: "behavior: `jet test`".to_string(),
            current_state: "Works for lockfile installs".to_string(),
            gaps: "peer dependency roadmap missing".to_string(),
            root_wi: "none".to_string(),
            active_wi: "#42".to_string(),
            evidence: "README".to_string(),
            claim_id: None,
            claim_user_story: None,
        };
        let issue = planning_issue(IssueType::Enhancement, "peer dependency support", None, 42);
        let issues = vec![issue];
        let matches = matching_issues_for_capability(&row, &issues);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].github_id, Some(42));
        let candidates = capability_wi_candidates(&[row], &issues);
        assert!(candidates.is_empty());
    }

    #[test]
    fn capability_matching_uses_plain_numeric_active_wi_reference() {
        let row = CapabilityRow {
            capability_id: "service-query-path".to_string(),
            capability: "Service query path".to_string(),
            capability_type: "Service".to_string(),
            surfaces: "API: `/query`".to_string(),
            ec_dimensions: "behavior: `rig`".to_string(),
            current_state: "Query path exists".to_string(),
            gaps: "deep-page chain needs proof".to_string(),
            root_wi: "none".to_string(),
            active_wi: "4141".to_string(),
            evidence: "README".to_string(),
            claim_id: None,
            claim_user_story: None,
        };
        let issue = planning_issue(IssueType::Enhancement, "deep-page proof", None, 4141);
        let issues = vec![issue];

        let matches = matching_issues_for_capability(&row, &issues);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].github_id, Some(4141));
        assert!(capability_wi_candidates(&[row], &issues).is_empty());
    }

    #[test]
    fn capability_claim_rows_do_not_match_broad_epic_by_keywords_only() {
        let row = CapabilityRow {
            capability_id: "package-manager".to_string(),
            capability: "Package Manager".to_string(),
            capability_type: "DeveloperTool".to_string(),
            surfaces: "CLI: `jet install`".to_string(),
            ec_dimensions: "behavior: `jet test`".to_string(),
            current_state: "Install surface exists".to_string(),
            gaps: "claim package-manager-workspace-parity: workspace package discovery needs a bounded verification WI".to_string(),
            root_wi: "none".to_string(),
            active_wi: "#3779".to_string(),
            evidence: "claim gate: cargo test -p jet pkg_manager::workspace".to_string(),
            claim_id: Some("package-manager-workspace-parity".to_string()),
            claim_user_story: Some(
                "As a monorepo maintainer, I want workspace package discovery parity."
                    .to_string(),
            ),
        };
        let mut epic = planning_issue(
            IssueType::Epic,
            "epic(jet): production replacement readiness",
            Some("p1"),
            3778,
        );
        epic.body = "Package manager workspace readiness is one child area of the broader production replacement epic.".to_string();
        let issues = vec![epic];

        assert!(matching_issues_for_capability(&row, &issues).is_empty());
        let candidates = capability_wi_candidates(&[row.clone()], &issues);
        let reconciliations = capability_tracker_reconciliations(&[row], &issues, &BTreeMap::new());

        assert!(candidates.is_empty());
        assert_eq!(reconciliations.len(), 1);
        assert_eq!(reconciliations[0].active_wi, "#3779");
    }

    #[test]
    fn resolve_capability_path_uses_cap_path_or_project_readme() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        std::fs::write(
            tmp.path().join("aw.toml"),
            r#"
[[projects]]
name = "jet"
aliases = ["j"]
path = "apps/jet"
label = "app:jet"

[[projects]]
name = "score"
path = "projects/score"
cap_path = "docs/score-cap.md"
label = "app:score"
"#,
        )
        .unwrap();
        assert_eq!(
            resolve_capability_path(tmp.path(), "j", None).unwrap(),
            tmp.path().join("apps/jet/README.md")
        );
        assert_eq!(
            resolve_capability_path(tmp.path(), "score", None).unwrap(),
            tmp.path().join("docs/score-cap.md")
        );
    }

    #[test]
    fn planning_slug_is_filesystem_safe() {
        assert_eq!(planning_slug("Score: Next Run!"), "score-next-run");
        assert_eq!(planning_slug("   "), "plan");
    }

    #[test]
    fn planning_artifact_rejects_tmp_root_output() {
        let tmp = tempfile::TempDir::new().unwrap();
        let err = write_planning_artifact(
            tmp.path(),
            "lumen",
            "capability-plan",
            "Lumen operator reshard policy",
            Some(Path::new("/private/tmp/lumen-operator-reshard-policy.md")),
            "# draft\n",
        )
        .unwrap_err();
        let message = format!("{err:#}");

        assert!(message.contains("ambiguous planning artifact output"));
        assert!(message.contains("/private/tmp/lumen-operator-reshard-policy.md"));
        assert!(message.contains("/tmp/aw/workspaces/"));
        assert!(
            message.contains("/workitems/lumen/capability-plan/lumen-operator-reshard-policy.md")
        );
    }

    #[test]
    fn validate_backend_resolution_falls_back_to_local_without_config() {
        let tmp = tempfile::tempdir().unwrap();
        let (kind, repo, host) = resolve_validate_backend(None, tmp.path()).unwrap();
        assert_eq!(kind, "local");
        assert!(repo.is_none());
        assert!(host.is_none());
    }

    /// R4(a): `(fill)` in Scope is rejected by the new placeholder check.
    #[test]
    fn placeholder_in_scope_rejected() {
        let body = body_with(
            "### In Scope\n- (fill)\n\n### Out of Scope\n- nothing",
            "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| foo.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| foo | create | foo.md |",
        );
        let errs = validate_section_format(&body, IssueSection::Scope);
        assert_eq!(errs.len(), 1, "expected exactly one error, got {:?}", errs);
        assert!(
            errs[0].contains("placeholder"),
            "error must mention placeholder: {}",
            errs[0]
        );
        assert!(
            errs[0].contains("Scope"),
            "error must mention section name: {}",
            errs[0]
        );
    }

    /// R4(b): `(fill)` in Reference Context is rejected.
    #[test]
    fn placeholder_in_reference_context_rejected() {
        let body = body_with(
            "### In Scope\n- real item\n\n### Out of Scope\n- nothing",
            "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| (fill) | (fill) |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| (fill) | (fill) | (fill) |",
        );
        let errs = validate_section_format(&body, IssueSection::ReferenceContext);
        assert_eq!(errs.len(), 1, "expected exactly one error, got {:?}", errs);
        assert!(
            errs[0].contains("placeholder"),
            "error must mention placeholder: {}",
            errs[0]
        );
        assert!(
            errs[0].contains("Reference Context"),
            "error must mention section name: {}",
            errs[0]
        );
    }

    /// R4(c): a body with real content in both sections passes.
    #[test]
    fn clean_body_passes() {
        let body = body_with(
            "### In Scope\n- real scope item\n\n### Out of Scope\n- explicit exclusion",
            "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| foo.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| foo | create | foo.md |",
        );
        let scope_errs = validate_section_format(&body, IssueSection::Scope);
        assert!(
            scope_errs.is_empty(),
            "Scope should pass, got {:?}",
            scope_errs
        );
        let rc_errs = validate_section_format(&body, IssueSection::ReferenceContext);
        assert!(
            rc_errs.is_empty(),
            "Reference Context should pass, got {:?}",
            rc_errs
        );
    }

    /// R2: missing '### In Scope' is now detected even when '### Out of Scope' is present.
    #[test]
    fn scope_missing_in_scope_detected() {
        let body = body_with(
            "### Out of Scope\n- something",
            "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| foo.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| foo | create | foo.md |",
        );
        let errs = validate_section_format(&body, IssueSection::Scope);
        assert!(
            errs.iter().any(|e| e.contains("In Scope")),
            "expected '### In Scope' missing error, got {:?}",
            errs
        );
    }

    /// @spec .aw/tech-design/projects/jet/specs/3941.md#unit-test
    #[test]
    fn normalize_scope_preserves_loose_out_of_scope_label() {
        let normalized = normalize_scope_section_content(
            "In scope:\n- Build a parity harness\n\nOut of scope:\n- Full MUI corpus rollout\n- Pixel-perfect baselines",
        );
        assert!(
            normalized.contains("### In Scope"),
            "must create canonical In Scope heading: {normalized}"
        );
        assert!(
            normalized.contains("### Out of Scope"),
            "must create canonical Out of Scope heading: {normalized}"
        );
        let out_heading = normalized
            .find("### Out of Scope")
            .expect("out heading present");
        let in_item = normalized
            .find("- Build a parity harness")
            .expect("in item present");
        let out_item = normalized
            .find("- Full MUI corpus rollout")
            .expect("out item present");
        assert!(
            in_item < out_heading && out_item > out_heading,
            "loose out-of-scope bullets must not be merged into In Scope: {normalized}"
        );
    }

    #[test]
    fn validate_planning_alignment_rejects_huge_non_epic() {
        let mut issue = planning_issue(
            IssueType::Enhancement,
            "Build Google Maps in Rust",
            Some("p1"),
            10,
        );
        issue.body = body_with(
            "### In Scope\n- real scope item\n\n### Out of Scope\n- explicit exclusion",
            "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| foo.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| foo | create | foo.md |",
        );
        let errors = validate_planning_alignment(&issue);
        assert!(
            errors.iter().any(|e| e.contains("too-large")),
            "expected too-large validation error, got {:?}",
            errors
        );
        assert!(
            errors.iter().any(|e| e.contains("Capability Alignment")),
            "expected capability alignment error, got {:?}",
            errors
        );
    }

    /// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
    #[test]
    fn too_large_gate_regression_false_positives_pass_issue_1294() {
        // #1294: hyphenated technical terms must not trip the raw substring
        // that used to fire on "whole"/"everything" anywhere in the text.
        let mut whole_doc = planning_issue(
            IssueType::Enhancement,
            "Adopt whole-doc LWW conflict resolution",
            Some("p1"),
            20,
        );
        whole_doc.body = format!(
            "{}\n\nAlways-send-everything contract stays unchanged; the client should own the whole row on write.",
            whole_doc.body
        );
        assert!(
            !looks_too_large_for_atomic_wi(&whole_doc),
            "hyphenated whole-doc/always-send-everything technical prose must not flag too-large"
        );

        let own_whole_row = planning_issue(
            IssueType::Bug,
            "Client should own the whole row on write",
            Some("p1"),
            21,
        );
        assert!(
            !looks_too_large_for_atomic_wi(&own_whole_row),
            "'own the whole row' is bounded technical prose, not roadmap scope"
        );

        // A genuinely bounded two-clause title joined by a semicolon must
        // still pass -- semicolon punctuation alone is not a size signal.
        let semicolon_title = planning_issue(
            IssueType::Enhancement,
            "Fix config parser edge case; add regression test for empty section",
            Some("p1"),
            22,
        );
        assert!(
            !looks_too_large_for_atomic_wi(&semicolon_title),
            "a bounded semicolon-joined two-clause title must not flag too-large"
        );
    }

    /// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
    #[test]
    fn too_large_gate_still_flags_context_and_hard_phrases() {
        // Bare scale word next to a real scope noun still flags.
        let whole_platform = planning_issue(
            IssueType::Enhancement,
            "Migrate the whole platform to Rust",
            Some("p0"),
            23,
        );
        assert!(
            looks_too_large_for_atomic_wi(&whole_platform),
            "'the whole platform' co-occurrence must still flag too-large"
        );

        let entire_codebase = planning_issue(
            IssueType::Enhancement,
            "Rewrite the entire codebase in Zig",
            Some("p0"),
            24,
        );
        assert!(
            looks_too_large_for_atomic_wi(&entire_codebase),
            "'the entire codebase' co-occurrence must still flag too-large"
        );

        // Adversarial hard-phrase true positives beyond the pre-existing
        // Google Maps case.
        let all_projects = planning_issue(
            IssueType::Enhancement,
            "Roll out the new lint config across all projects",
            Some("p0"),
            25,
        );
        assert!(
            looks_too_large_for_atomic_wi(&all_projects),
            "'all projects' is an unambiguous roadmap-scale phrase"
        );

        let rewrite_everything = planning_issue(
            IssueType::Enhancement,
            "Rewrite everything across the fleet",
            Some("p0"),
            26,
        );
        assert!(
            looks_too_large_for_atomic_wi(&rewrite_everything),
            "'rewrite everything'/'across the fleet' must still flag too-large"
        );
    }

    #[test]
    fn wi_remove_agent_estimate_bounded_non_epic_passes_without_estimate() {
        let mut issue = planning_issue(IssueType::Bug, "Fix config parsing", Some("p1"), 11);
        issue.body = format!(
            "{}\n\n## Capability Alignment\n\nCapability: Config correctness\nCapability Gap: malformed config errors are unclear\nProgress Evidence: parser fixture reports line number\n\n## Acceptance Criteria\n\n- AC1: config parse fixture reports the line number\n",
            body_with(
                "### In Scope\n- real scope item\n\n### Out of Scope\n- explicit exclusion",
                "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| foo.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| foo | create | foo.md |",
            )
        );
        let errors = validate_planning_alignment(&issue);
        assert!(errors.is_empty(), "expected pass, got {:?}", errors);
    }

    #[test]
    fn wi_remove_agent_estimate_legacy_section_is_inert() {
        let mut issue = planning_issue(IssueType::Bug, "Fix config parsing", Some("p1"), 12);
        issue.body = format!(
            "{}\n\n## Capability Alignment\n\nCapability: Config correctness\nCapability Gap: malformed config errors are unclear\nProgress Evidence: parser fixture reports line number\n\n## Acceptance Criteria\n\n- AC1: config parse fixture reports the line number\n\n## Agent Estimate\n\nagent_minutes: 45\nconfidence: medium\nrisk: medium\nhuman_attention: confirm\n",
            body_with(
                "### In Scope\n- real scope item\n\n### Out of Scope\n- explicit exclusion",
                "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| foo.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| foo | create | foo.md |",
            )
        );
        let errors = validate_planning_alignment(&issue);
        assert!(
            errors.is_empty(),
            "legacy estimate section should be inert: {:?}",
            errors
        );
    }

    // -- project_label_warnings ---------------------------------------------

    #[test]
    fn project_label_warnings_non_epic_with_one_label_passes() {
        let labels = vec!["type:bug".into(), "app:cclab-agent".into()];
        assert!(project_label_warnings(&labels, IssueType::Bug, "demo", &[]).is_empty());
    }

    #[test]
    fn project_label_warnings_non_epic_with_zero_labels_warns() {
        let labels = vec!["type:bug".into()];
        let warnings = project_label_warnings(&labels, IssueType::Enhancement, "demo", &[]);
        assert_eq!(warnings.len(), 1);
        let msg = &warnings[0];
        assert!(msg.contains("no app/lib"), "msg was: {}", msg);
        assert!(msg.contains("demo"), "msg should name the slug: {}", msg);
    }

    #[test]
    fn project_label_warnings_non_epic_with_multiple_labels_warns() {
        let labels = vec![
            "type:refactor".into(),
            "app:cclab-agent".into(),
            "app:agentic-workflow".into(),
        ];
        let warnings = project_label_warnings(&labels, IssueType::Refactor, "demo", &[]);
        assert_eq!(warnings.len(), 1);
        let msg = &warnings[0];
        assert!(msg.contains("2 app/lib"), "msg should count: {}", msg);
        assert!(
            msg.contains("only epics may span"),
            "msg should explain epic exception: {}",
            msg
        );
    }

    #[test]
    fn project_label_warnings_epic_with_zero_labels_passes() {
        let labels = vec!["type:epic".into()];
        assert!(project_label_warnings(&labels, IssueType::Epic, "demo", &[]).is_empty());
    }

    #[test]
    fn project_label_warnings_epic_with_multiple_labels_passes() {
        let labels = vec![
            "type:epic".into(),
            "app:cclab-agent".into(),
            "app:agentic-workflow".into(),
            "app:conductor".into(),
        ];
        assert!(project_label_warnings(&labels, IssueType::Epic, "demo", &[]).is_empty());
    }

    #[test]
    fn project_label_warnings_known_label_passes() {
        let labels = vec!["type:bug".into(), "app:agentic-workflow".into()];
        let known = vec!["app:agentic-workflow".into(), "app:agentic-workflow".into()];
        assert!(project_label_warnings(&labels, IssueType::Bug, "demo", &known).is_empty());
    }

    #[test]
    fn project_label_warnings_unknown_label_warns_against_known_set() {
        let labels = vec!["type:bug".into(), "app:typo".into()];
        let known = vec!["app:agentic-workflow".into(), "app:agentic-workflow".into()];
        let warnings = project_label_warnings(&labels, IssueType::Bug, "demo", &known);
        assert_eq!(
            warnings.len(),
            1,
            "expected one warning, got {:?}",
            warnings
        );
        let msg = &warnings[0];
        assert!(
            msg.contains("app:typo"),
            "msg should name the bad label: {}",
            msg
        );
        assert!(msg.contains("not declared"), "msg should explain: {}", msg);
        assert!(
            msg.contains("app:agentic-workflow"),
            "msg should list known labels: {}",
            msg
        );
    }

    #[test]
    fn project_label_warnings_unknown_label_with_empty_known_skips_value_check() {
        let labels = vec!["type:bug".into(), "app:typo".into()];
        // Empty known => degrade gracefully, only the count rule fires (and
        // here count=1 is canonical, so no warnings at all).
        assert!(project_label_warnings(&labels, IssueType::Bug, "demo", &[]).is_empty());
    }

    #[test]
    fn project_label_warnings_unknown_label_on_epic_still_warns() {
        let labels = vec!["type:epic".into(), "app:typo".into()];
        let known = vec!["app:agentic-workflow".into()];
        let warnings = project_label_warnings(&labels, IssueType::Epic, "demo", &known);
        assert_eq!(warnings.len(), 1, "epic with bad label should still warn");
        assert!(warnings[0].contains("app:typo"));
    }

    #[test]
    fn project_label_warnings_count_and_value_warnings_combine() {
        // Two unknown labels => one count warning + two value warnings.
        let labels = vec![
            "type:refactor".into(),
            "app:typo-a".into(),
            "app:typo-b".into(),
        ];
        let known = vec!["app:agentic-workflow".into()];
        let warnings = project_label_warnings(&labels, IssueType::Refactor, "demo", &known);
        assert_eq!(
            warnings.len(),
            3,
            "expected 1 count + 2 value warnings, got {:?}",
            warnings
        );
    }

    // -- read_known_project_labels ------------------------------------------

    #[test]
    fn read_known_project_labels_missing_config_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        // No aw.toml at all.
        assert!(read_known_project_labels(tmp.path()).is_empty());
    }

    #[test]
    fn read_known_project_labels_no_projects_table_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        std::fs::write(tmp.path().join("aw.toml"), "version = \"0.3.13\"\n").unwrap();
        assert!(read_known_project_labels(tmp.path()).is_empty());
    }

    #[test]
    fn read_known_project_labels_collects_labels() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        std::fs::write(
            tmp.path().join("aw.toml"),
            r#"
[[projects]]
name = "agentic-workflow"
path = "apps/agentic-workflow"
label = "app:agentic-workflow"

[[projects]]
name = "agentic-workflow"
path = "apps/agentic-workflow"
label = "app:agentic-workflow"

[[projects]]
name = "no-label"
path = "crates/no-label"
"#,
        )
        .unwrap();
        let labels = read_known_project_labels(tmp.path());
        assert_eq!(labels, vec!["app:agentic-workflow", "app:no-label"]);
    }

    // -- score-wi-cli-redesign: typed-flag tests ----------------------------
    // @spec apps/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#test-plan

    fn write_config(tmp: &std::path::Path, body: &str) {
        std::fs::create_dir_all(tmp.join(".aw")).unwrap();
        std::fs::write(tmp.join("aw.toml"), body).unwrap();
    }

    const CONFIG_WITH_PROJECTS_AND_AGENTS: &str = r#"
[[projects]]
name = "mamba"
label = "app:mamba"

[[projects]]
name = "agentic-workflow"
label = "app:agentic-workflow"

[[agents]]
name = "claude-code"
label = "agent::claude-code"

[[agents]]
name = "codex"
label = "agent::codex"
"#;

    #[test]
    fn resolve_project_label_known_returns_label() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let label = resolve_project_label(tmp.path(), "agentic-workflow").unwrap();
        assert_eq!(label, "app:agentic-workflow");
    }

    #[test]
    fn resolve_project_label_alias_returns_canonical_label() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(
            tmp.path(),
            r#"
[[projects]]
name = "agentic-workflow"
aliases = ["aw"]
label = "app:agentic-workflow"
"#,
        );
        let label = resolve_project_label(tmp.path(), "aw").unwrap();
        assert_eq!(label, "app:agentic-workflow");
    }

    #[test]
    fn resolve_agent_label_alias_returns_canonical_label() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(
            tmp.path(),
            r#"
[[agents]]
name = "claude-code"
aliases = ["cc"]
label = "agent::claude-code"
"#,
        );
        let label = resolve_agent_label(tmp.path(), "cc").unwrap();
        assert_eq!(label, "agent::claude-code");
    }

    #[test]
    fn resolve_project_label_unknown_returns_envelope_error() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let err = resolve_project_label(tmp.path(), "ghost").unwrap_err();
        let msg = err.to_envelope_message();
        assert!(msg.contains("unknown --project 'ghost'"), "msg: {}", msg);
        assert!(
            msg.contains("agentic-workflow"),
            "msg should list valid names: {}",
            msg
        );
        assert!(
            msg.contains("mamba"),
            "msg should list valid names: {}",
            msg
        );
    }

    #[test]
    fn list_project_filter_resolves_configured_label() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let label = resolve_list_label_filter(tmp.path(), None, Some("agentic-workflow")).unwrap();
        assert_eq!(label.as_deref(), Some("app:agentic-workflow"));
    }

    #[test]
    fn list_project_filter_rejects_raw_label_combination() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let err = resolve_list_label_filter(
            tmp.path(),
            Some("app:agentic-workflow"),
            Some("agentic-workflow"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("use either --label or --project"));
    }

    #[test]
    fn infer_project_from_project_branch() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let project =
            infer_project_name_from_branch(tmp.path(), "project-agentic-workflow").unwrap();
        assert_eq!(project, "agentic-workflow");
    }

    #[test]
    fn infer_project_from_alias_project_branch() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(
            tmp.path(),
            r#"
[[projects]]
name = "agentic-workflow"
aliases = ["aw"]
label = "app:agentic-workflow"
"#,
        );
        let project = infer_project_name_from_branch(tmp.path(), "project-aw").unwrap();
        assert_eq!(project, "agentic-workflow");
    }

    #[test]
    fn infer_project_from_alias_wi_branch_prefix() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(
            tmp.path(),
            r#"
[[projects]]
name = "agentic-workflow"
aliases = ["aw"]
label = "app:agentic-workflow"
"#,
        );
        let project = infer_project_name_from_branch(tmp.path(), "project-aw-wi-foo").unwrap();
        assert_eq!(project, "agentic-workflow");
    }

    #[test]
    fn infer_project_from_wi_branch_prefix() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let project =
            infer_project_name_from_branch(tmp.path(), "agentic-workflow-wi-20260513-mermaid")
                .unwrap();
        assert_eq!(project, "agentic-workflow");
    }

    #[test]
    fn infer_project_from_project_wi_branch_prefix() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let project =
            infer_project_name_from_branch(tmp.path(), "project-agentic-workflow-wi-draft-flow")
                .unwrap();
        assert_eq!(project, "agentic-workflow");
    }

    #[test]
    fn infer_project_from_main_requires_project_flag() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let err = infer_project_name_from_branch(tmp.path(), "main").unwrap_err();
        let msg = err.to_envelope_message();
        assert!(msg.contains("--project is required"), "msg: {}", msg);
        assert!(msg.contains("agentic-workflow"), "msg: {}", msg);
        assert!(msg.contains("mamba"), "msg: {}", msg);
    }

    #[test]
    fn render_draft_issue_markdown_writes_tmp_metadata() {
        let issue = Issue {
            issue_type: IssueType::Enhancement,
            title: "enhancement(agentic-workflow): demo".to_string(),
            state: IssueState::Draft,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec![
                "type:enhancement".to_string(),
                "app:agentic-workflow".to_string(),
            ],
            created_at: Some("2026-05-13T00:00:00Z".to_string()),
            updated_at: Some("2026-05-13T00:00:00Z".to_string()),
            slug: "wi-demo".to_string(),
            body: "## Problem\n\nDemo\n".to_string(),
            related: vec![],
            implements: vec![],
            phase: Some("created".to_string()),
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: vec![],
            review_count: Some(0),
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        };
        let rendered = render_draft_issue_markdown(&issue, "agentic-workflow", "wi-demo");
        assert!(rendered.contains("draft: true"));
        assert!(rendered.contains("tmp_id: 'wi-demo'"));
        assert!(rendered.contains("project: 'agentic-workflow'"));
        assert!(rendered.contains("- 'app:agentic-workflow'"));
        assert!(rendered.contains("## Problem"));
    }

    #[test]
    fn read_draft_issue_parses_metadata_and_body() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("wi-demo.md");
        std::fs::write(
            &path,
            "---\n\
draft: true\n\
tmp_id: 'wi-demo'\n\
project: 'agentic-workflow'\n\
type: enhancement\n\
title: 'demo draft'\n\
state: draft\n\
labels:\n\
- 'type:enhancement'\n\
- 'app:agentic-workflow'\n\
---\n\n\
## Problem\n\nDemo\n",
        )
        .unwrap();

        let (issue, meta) = read_draft_issue(&path).unwrap();
        assert!(meta.draft);
        assert_eq!(meta.project, "agentic-workflow");
        assert_eq!(meta.tmp_id.as_deref(), Some("wi-demo"));
        assert_eq!(issue.title, "demo draft");
        assert_eq!(issue.issue_type, IssueType::Enhancement);
        assert!(issue.body.contains("## Problem"));
    }

    #[test]
    fn validate_draft_fill_checks_only_target_sections() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let path = tmp.path().join("wi-demo.md");
        let meta = DraftIssueFrontmatter {
            draft: true,
            tmp_id: Some("wi-demo".to_string()),
            project: "agentic-workflow".to_string(),
        };
        let issue = Issue {
            issue_type: IssueType::Enhancement,
            title: "demo draft".to_string(),
            state: IssueState::Draft,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec!["type:enhancement".to_string(), "app:agentic-workflow".to_string()],
            created_at: None,
            updated_at: None,
            slug: "wi-demo".to_string(),
            body: "## Problem\n\n(fill)\n\n## Requirements\n\n- R1: Real draft requirement.\n\n## Scope\n\n- (fill)\n\n## Reference Context\n\n(fill)\n".to_string(),
            related: vec![],
            implements: vec![],
            phase: Some("created".to_string()),
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: vec![],
            review_count: Some(0),
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        };

        let errors = validate_draft_fill(
            tmp.path(),
            &path,
            &issue,
            &meta,
            &[IssueSection::Requirements],
        );
        assert!(
            errors.is_empty(),
            "target-only fill should pass: {:?}",
            errors
        );
    }

    #[test]
    fn default_draft_body_passes_draft_validation() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let path = tmp.path().join("wi-demo.md");
        let meta = DraftIssueFrontmatter {
            draft: true,
            tmp_id: Some("wi-demo".to_string()),
            project: "agentic-workflow".to_string(),
        };
        let mut issue =
            planning_issue(IssueType::Enhancement, "Fix config parsing", Some("p2"), 14);
        issue.state = IssueState::Draft;
        issue.slug = "wi-demo".to_string();
        issue.body = default_structured_issue_body("Fix config parsing");

        let errors = validate_draft_issue(tmp.path(), &path, &issue, &meta);
        assert!(
            errors.is_empty(),
            "default draft body should validate: {:?}",
            errors
        );
    }

    #[test]
    fn issue_show_json_includes_slug_and_body_inline() {
        let issue = test_issue_with_phase(Some("created"));
        let value = issue_show_json(&issue).unwrap();

        assert_eq!(value["slug"], "1234");
        assert!(value["body"]
            .as_str()
            .unwrap()
            .contains("## Reference Context"));
    }

    // @spec workitem-loop-state-model-additive-foundation.md R3
    #[test]
    fn issue_show_json_surfaces_loop_state() {
        use crate::cli::loop_state::{upsert_loop_state, LoopState, LoopStatus};
        // Absent block -> loop_state is null, not an error.
        let mut issue = test_issue_with_phase(Some("created"));
        let value = issue_show_json(&issue).unwrap();
        assert!(value["loop_state"].is_null());

        // Present block -> surfaced under `loop_state`.
        let state = LoopState {
            version: 1,
            issue_id: "1234".into(),
            goal: Some("some-gap".into()),
            status: LoopStatus::Iterating,
            ..Default::default()
        };
        issue.body = upsert_loop_state(&issue.body, &state).unwrap();
        let value = issue_show_json(&issue).unwrap();
        assert_eq!(value["loop_state"]["goal"], "some-gap");
        assert_eq!(value["loop_state"]["status"], "iterating");
    }

    #[test]
    fn initial_draft_body_normalizes_unnumbered_requirements_and_flat_scope() {
        let body = normalize_initial_draft_body(
            "Fix config parsing",
            "## Problem\n\nParser errors hide the line number.\n\n## Requirements\n\n- report the failing line number\n\n## Scope\n\n- parser diagnostics only\n",
        );
        assert!(body.contains("- R1: report the failing line number"));
        assert!(body.contains("### In Scope"));
        assert!(body.contains("- parser diagnostics only"));
        assert!(body.contains("### Out of Scope"));

        let mut issue = planning_issue(IssueType::Bug, "Fix config parsing", Some("p1"), 15);
        issue.body = body;
        let errors = validate_publishable_issue_body(&issue);
        assert!(
            errors.is_empty(),
            "normalized draft body should publish cleanly: {:?}",
            errors
        );
    }

    #[test]
    fn publish_validation_rejects_invalid_reference_context() {
        let mut issue = planning_issue(IssueType::Bug, "Fix config parsing", Some("p1"), 16);
        issue.body = "## Problem\n\nParser errors hide the line number.\n\n## Capability Alignment\n\nCapability: Config correctness\nCapability Gap: parser diagnostics are incomplete\nProgress Evidence: validation error includes line number\n\n## Requirements\n\n- R1: Report the failing line number.\n\n## Scope\n\n### In Scope\n- Parser diagnostics.\n\n### Out of Scope\n- New config schema.\n\n## Acceptance Criteria\n\n- AC1: parser diagnostic includes line number\n\n## Reference Context\n\nnone\n".to_string();

        let errors = validate_publishable_issue_body(&issue);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("Reference Context missing '### Related Specs'")),
            "expected reference context error, got {:?}",
            errors
        );
    }

    #[test]
    fn merge_all_sections_replaces_alignment_without_estimate_section() {
        let base = default_structured_issue_body("Fix config parsing");
        let payload = "## Problem\n\nFix config parsing.\n\n## Capability Alignment\n\nCapability: Config correctness\nCapability Gap: malformed config errors are unclear\nProgress Evidence: parser fixture reports line number\n\n## Requirements\n\n- R1: Report the line number for malformed config.\n\n## Scope\n\n### In Scope\n- Config parser error fixture.\n\n### Out of Scope\n- New config schema format.\n\n## Acceptance Criteria\n\n- AC1: malformed config fixture reports the line number\n\n## Reference Context\n\n### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| foo.md | high |\n\n### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| foo | create | foo.md |\n";
        let merged = merge_all_sections(&base, payload);
        assert!(merged.contains("Capability: Config correctness"));
        assert!(!merged.contains("Agent Estimate"));
        assert!(!merged.contains("agent_minutes"));
        let mut issue = planning_issue(IssueType::Bug, "Fix config parsing", Some("p1"), 13);
        issue.body = merged;
        let errors = validate_planning_alignment(&issue);
        assert!(errors.is_empty(), "expected pass, got {:?}", errors);
    }

    #[test]
    fn fill_section_payload_template_all_scaffolds_required_sections() {
        let template = fill_section_payload_template("all").unwrap();
        for heading in [
            "## Problem",
            "## Capability Alignment",
            "## Requirements",
            "## Scope",
            "## Acceptance Criteria",
            "## Reference Context",
        ] {
            assert!(
                template.contains(heading),
                "template missing heading {heading}"
            );
        }
        assert!(template.contains("(fill)"));
        assert!(template.contains("### Related Specs"));
        assert!(template.contains("### Spec Plan"));
    }

    #[test]
    fn fill_section_payload_template_specific_sections_are_bounded() {
        let template = fill_section_payload_template("requirements,scope").unwrap();
        assert!(template.contains("## Requirements"));
        assert!(template.contains("- R1: (fill)"));
        assert!(template.contains("## Scope"));
        assert!(template.contains("### In Scope"));
        assert!(template.contains("### Out of Scope"));
        assert!(!template.contains("## Reference Context"));
    }

    #[test]
    fn fill_section_payload_scope_accepts_only_declared_slots() {
        let all = fill_section_payload_template("all").unwrap();
        validate_wi_fill_payload_scope("all", &all, &[]).unwrap();

        let extra = format!("{all}\n## Surprise\n\noutside the producer contract\n");
        let error = validate_wi_fill_payload_scope("all", &extra, &[]).unwrap_err();
        assert!(error.to_string().contains("outside requested slot `all`"));

        let targets = parse_section_arg("requirements,scope").unwrap();
        let specific = fill_section_payload_template("requirements,scope").unwrap();
        validate_wi_fill_payload_scope("requirements,scope", &specific, &targets).unwrap();

        let wrong = fill_section_payload_template("reference_context").unwrap();
        let error =
            validate_wi_fill_payload_scope("requirements,scope", &wrong, &targets).unwrap_err();
        assert!(error
            .to_string()
            .contains("outside requested slot `requirements,scope`"));
    }

    #[test]
    fn initialize_payload_file_creates_parent_and_preserves_existing_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = crate::shared::workspace::payloads_path(tmp.path())
            .join("wi")
            .join("123")
            .join("body.md");

        assert!(initialize_payload_file(&path, "first\n").unwrap());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "first\n");

        assert!(!initialize_payload_file(&path, "second\n").unwrap());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "first\n");
    }

    #[test]
    fn resolve_agent_label_known_returns_label() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let label = resolve_agent_label(tmp.path(), "claude-code").unwrap();
        assert_eq!(label, "agent::claude-code");
    }

    #[test]
    fn resolve_agent_label_unknown_returns_envelope_error() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let err = resolve_agent_label(tmp.path(), "openai").unwrap_err();
        let msg = err.to_envelope_message();
        assert!(msg.contains("unknown --agent 'openai'"), "msg: {}", msg);
        assert!(
            msg.contains("claude-code"),
            "msg should list valid: {}",
            msg
        );
    }

    #[test]
    fn check_project_cardinality_non_epic_exact_one_passes() {
        assert!(check_project_cardinality(IssueType::Bug, 1).is_ok());
        assert!(check_project_cardinality(IssueType::Enhancement, 1).is_ok());
        assert!(check_project_cardinality(IssueType::Refactor, 1).is_ok());
        assert!(check_project_cardinality(IssueType::Test, 1).is_ok());
    }

    #[test]
    fn check_project_cardinality_non_epic_zero_fails() {
        let err = check_project_cardinality(IssueType::Bug, 0).unwrap_err();
        let msg = err.to_envelope_message();
        assert!(msg.contains("requires exactly 1 --project"), "msg: {}", msg);
        assert!(msg.contains("observed 0"), "msg: {}", msg);
    }

    #[test]
    fn check_project_cardinality_non_epic_multiple_fails() {
        let err = check_project_cardinality(IssueType::Enhancement, 3).unwrap_err();
        let msg = err.to_envelope_message();
        assert!(msg.contains("requires exactly 1 --project"), "msg: {}", msg);
        assert!(msg.contains("observed 3"), "msg: {}", msg);
    }

    #[test]
    fn check_project_cardinality_epic_zero_or_one_passes() {
        assert!(check_project_cardinality(IssueType::Epic, 0).is_ok());
        assert!(check_project_cardinality(IssueType::Epic, 1).is_ok());
    }

    #[test]
    fn check_project_cardinality_epic_multiple_fails() {
        let err = check_project_cardinality(IssueType::Epic, 2).unwrap_err();
        let msg = err.to_envelope_message();
        assert!(msg.contains("epic accepts 0 or 1"), "msg: {}", msg);
        assert!(msg.contains("observed 2"), "msg: {}", msg);
    }

    #[test]
    fn build_create_label_vec_orders_type_project_priority_agent() {
        let labels = build_create_label_vec(
            "type:bug",
            &["app:agentic-workflow".into()],
            Some("priority:p1"),
            Some("agent::claude-code"),
        );
        assert_eq!(
            labels,
            vec![
                "type:bug",
                "app:agentic-workflow",
                "priority:p1",
                "agent::claude-code"
            ]
        );
    }

    #[test]
    fn build_create_label_vec_skips_optional_when_absent() {
        let labels = build_create_label_vec(
            "type:enhancement",
            &["app:agentic-workflow".into()],
            None,
            None,
        );
        assert_eq!(labels, vec!["type:enhancement", "app:agentic-workflow"]);
    }

    #[test]
    fn build_create_label_vec_dedupes_preserving_first_seen_order() {
        let labels = build_create_label_vec(
            "type:epic",
            &["app:agentic-workflow".into(), "app:agentic-workflow".into()],
            None,
            None,
        );
        assert_eq!(labels, vec!["type:epic", "app:agentic-workflow"]);
    }

    #[test]
    fn build_create_label_vec_epic_multi_project_ordered() {
        let labels = build_create_label_vec(
            "type:epic",
            &["app:agentic-workflow".into(), "app:mamba".into()],
            None,
            None,
        );
        assert_eq!(
            labels,
            vec!["type:epic", "app:agentic-workflow", "app:mamba"]
        );
    }

    fn create_args_test_command() -> clap::Command {
        <CreateArgs as clap::Args>::augment_args(clap::Command::new("create"))
    }

    #[test]
    fn wi_create_remote_help_hides_deprecated_remote_flag() {
        let mut command = create_args_test_command();
        let help = command.render_long_help().to_string();

        assert!(
            !help.contains("--remote"),
            "create help should not expose deprecated --remote flag:\n{}",
            help
        );
    }

    #[test]
    fn wi_create_remote_compat_flag_still_parses_hidden_noop() {
        let matches = create_args_test_command()
            .try_get_matches_from([
                "create",
                "--title",
                "Demo",
                "--type",
                "bug",
                "--project",
                "agentic-workflow",
                "--remote",
            ])
            .unwrap();
        let args = <CreateArgs as clap::FromArgMatches>::from_arg_matches(&matches).unwrap();

        assert!(args.remote);
    }

    #[test]
    fn wi_create_remote_backend_selection_is_config_driven() {
        assert!(!create_uses_remote_backend("local"));
        assert!(create_uses_remote_backend("github"));
        assert!(create_uses_remote_backend("gitlab"));
    }

    #[test]
    fn read_known_agent_name_label_pairs_collects_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let pairs = read_known_agent_name_label_pairs(tmp.path());
        assert_eq!(
            pairs,
            vec![
                ("claude-code".to_string(), "agent::claude-code".to_string()),
                ("codex".to_string(), "agent::codex".to_string()),
            ]
        );
    }

    #[test]
    fn read_known_agent_name_label_pairs_empty_when_no_agents_table() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(
            tmp.path(),
            r#"
[[projects]]
name = "agentic-workflow"
label = "app:agentic-workflow"
"#,
        );
        let pairs = read_known_agent_name_label_pairs(tmp.path());
        assert!(
            pairs.is_empty(),
            "expected empty agents pairs, got {:?}",
            pairs
        );
    }

    #[test]
    fn priority_filter_label_suffixes_are_lowercase() {
        assert_eq!(PriorityFilter::P0.as_label_suffix(), "p0");
        assert_eq!(PriorityFilter::P1.as_label_suffix(), "p1");
        assert_eq!(PriorityFilter::P2.as_label_suffix(), "p2");
        assert_eq!(PriorityFilter::P3.as_label_suffix(), "p3");
    }
}

// CODEGEN-END
