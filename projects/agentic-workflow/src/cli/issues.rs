// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
// CODEGEN-BEGIN
//! `aw wi` CLI -- list/show/sync/create/update/close/find across
//! local + GitHub + GitLab backends.
//!
//! Backend selection is resolved from `.aw/config.toml`
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
use std::path::{Path, PathBuf};

// Top-level args for `aw wi`.
// @spec projects/agentic-workflow/tech-design/surface/issues_top.md#schema
#[derive(Debug, Args)]
pub struct IssuesArgs {
    /// The selected subcommand.
    #[command(subcommand)]
    pub command: IssuesCommand,
}

// Available subcommands for `aw wi`.
// @spec projects/agentic-workflow/tech-design/surface/issues_top.md#schema
#[derive(Debug, Subcommand)]
pub enum IssuesCommand {
    /// Work with local draft work-items before creating a tracker issue.
    Draft(DraftArgs),
    /// List work-items from a backend.
    List(ListArgs),
    /// Show a single work-item by slug or numeric id.
    Show(ShowArgs),
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
    /// Plan a project phase from the current work-item inventory.
    Epicize(EpicizeArgs),
    /// Split epic/roadmap-sized work into atomic work-item candidates.
    Atomize(AtomizeArgs),
    /// Re-rank issue backlog by priority, dependency, and readiness.
    Prioritize(PrioritizeArgs),
    /// Fill the Reference Context section via agent exploration.
    Enrich(EnrichArgs),
    /// Validate work-item quality (CRR gate).
    Validate(ValidateArgs),
    /// Fill work-item sections via structured round-trip.
    FillSection(FillSectionArgs),
    /// Review the filled work-item via reviewer round-trip.
    Review(ReviewArgs),
    /// Arbitrate a stalled CRRR loop after second needs-revision.
    Arbitrate(ArbitrateArgs),
}
#[derive(Debug, Args)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct DraftArgs {
    #[command(subcommand)]
    pub command: DraftCommand,
}

#[derive(Debug, Subcommand)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub enum DraftCommand {
    /// Initialize a local draft work-item under /tmp/aw/{project}/workitems/.
    Init(DraftInitArgs),
    /// Fill sections in a local draft work-item.
    Fill(DraftFillArgs),
    /// Append a review bullet to a local draft work-item.
    Review(DraftReviewArgs),
    /// Validate a local draft work-item.
    Validate(DraftValidateArgs),
}

#[derive(Debug, Args)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct DraftValidateArgs {
    /// Draft markdown file created by `aw wi draft init`.
    pub draft_path: PathBuf,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct DraftReviewArgs {
    /// Draft markdown file created by `aw wi draft init`.
    pub draft_path: PathBuf,

    /// Inline review bullet. Mutually exclusive with --body-file.
    #[arg(long, conflicts_with = "body_file")]
    pub body: Option<String>,

    /// Read review bullet from a file path, or `-` for stdin.
    #[arg(long)]
    pub body_file: Option<String>,

    /// Output machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R1
// @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
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
    /// `.aw/config.toml`; emits each entry's `label` field. Cardinality
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

    /// Agent name. Resolved against `[[agents]]` in `.aw/config.toml`;
    /// emits the entry's `label` field (e.g. `agent::claude-code`).
    /// Unknown name → error envelope.
    #[arg(long = "agent")]
    pub agent: Option<String>,

    /// Deprecated compatibility no-op. Backend selection is configured in
    /// `.aw/config.toml`; local-only authoring lives under `aw wi draft`.
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
// @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum PriorityFilter {
    P0,
    P1,
    P2,
    P3,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R3
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R4
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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

    /// Write plan to this path instead of /tmp/aw/{project}/capability-plan/.
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct EpicizeArgs {
    /// Project name. Defaults to the current project branch when omitted.
    #[arg(long)]
    pub project: Option<String>,

    /// Optional phase title.
    #[arg(long)]
    pub title: Option<String>,

    /// Write plan to this path instead of /tmp/aw/{project}/epics/.
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct AtomizeArgs {
    /// Project name. Defaults to the current project branch when omitted.
    #[arg(long)]
    pub project: Option<String>,

    /// Optional atomization title.
    #[arg(long)]
    pub title: Option<String>,

    /// Write plan to this path instead of /tmp/aw/{project}/atomize/.
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct PrioritizeArgs {
    /// Project name. Defaults to the current project branch when omitted.
    #[arg(long)]
    pub project: Option<String>,

    /// Optional planning title.
    #[arg(long)]
    pub title: Option<String>,

    /// Write plan to this path instead of /tmp/aw/{project}/priorities/.
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub struct EnrichArgs {
    /// Work-item slug.
    pub slug: String,
}

// REQ: R3, R4 — Work-item CRR loop
#[derive(Debug, Args)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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

// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R4 #R5
#[derive(Debug, Args)]
pub struct FillSectionArgs {
    /// Work-item slug.
    #[arg(long)]
    pub slug: String,

    /// Which section to fill. `all` (default) means the subagent writes the
    /// complete body in one pass.
    #[arg(long, default_value = "all")]
    pub section: String,

    /// Apply mode: merge .aw/payloads/<slug>/body.md into the checkout issue
    /// and emit the next validate envelope. Without this flag the CLI prints a
    /// plain-text brief.
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

// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R12
#[derive(Debug, Args)]
pub struct ArbitrateArgs {
    /// Work-item slug.
    #[arg(long)]
    pub slug: String,

    /// Send the work-item back for one more author pass — commits
    /// `Lifecycle-Stage: Reset`, resets `phase=created` and `review_count=0`,
    /// dispatches author to refill Requirements. Bounded once per slug; a
    /// second `--send-back` is rejected.
    #[arg(long)]
    pub send_back: bool,
}

// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R9 #R10
#[derive(Debug, Args)]
pub struct ReviewArgs {
    /// Work-item slug.
    #[arg(long)]
    pub slug: String,

    /// Apply mode: append .aw/payloads/<slug>/review.md under `# Reviews`
    /// and emit the next validate envelope.
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub enum BackendKind {
    Local,
    Github,
    Gitlab,
    Jira,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub enum StateFilter {
    Open,
    Closed,
    Draft,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub enum TypeFilter {
    Epic,
    Change,
    Bug,
    Enhancement,
    Refactor,
    Test,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R7
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
// Wire format per `projects/agentic-workflow/logic/structured-issue.md` R6:
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

// @spec projects/agentic-workflow/tech-design/surface/specs/aw-wi-draft-valid-by-construction.md#draft_authoring_contract
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

// @spec projects/agentic-workflow/tech-design/surface/specs/aw-wi-draft-valid-by-construction.md#draft_authoring_contract
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

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub async fn run(args: IssuesArgs) -> Result<()> {
    match args.command {
        IssuesCommand::Draft(a) => run_draft(a).await,
        IssuesCommand::List(a) => run_list(a).await,
        IssuesCommand::Show(a) => run_show(a).await,
        IssuesCommand::Create(a) => run_create(a).await,
        IssuesCommand::Update(a) => run_update(a).await,
        IssuesCommand::Close(a) => run_close(a).await,
        IssuesCommand::Find(a) => run_find(a).await,
        IssuesCommand::Plan(a) => run_plan(a).await,
        IssuesCommand::Epicize(a) => run_epicize(a).await,
        IssuesCommand::Atomize(a) => run_atomize(a).await,
        IssuesCommand::Prioritize(a) => run_prioritize(a).await,
        IssuesCommand::Enrich(a) => run_enrich(a).await,
        IssuesCommand::Validate(a) => run_validate(a).await,
        IssuesCommand::FillSection(a) => run_fill_section(a).await,
        IssuesCommand::Review(a) => run_review(a).await,
        IssuesCommand::Arbitrate(a) => run_arbitrate(a).await,
    }
}

async fn run_draft(args: DraftArgs) -> Result<()> {
    match args.command {
        DraftCommand::Init(a) => run_draft_init(a).await,
        DraftCommand::Fill(a) => run_draft_fill(a).await,
        DraftCommand::Review(a) => run_draft_review(a).await,
        DraftCommand::Validate(a) => run_draft_validate(a).await,
    }
}

// ---------------------------------------------------------------------------
// Backend resolution helper (Phase A)
// ---------------------------------------------------------------------------

// Resolve the backend triple `(kind, repo, host)` from `.aw/config.toml`.
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

    // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R5 — warn on broken cross-references
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R5
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

// Read declared `[[projects]].label` values from `.aw/config.toml`.
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
    let path = project_root.join(".aw/config.toml");
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

// Read `[[projects]]` entries as a `(name, label)` list from `.aw/config.toml`.
///
// Order is preserved from the config file so error messages and the
// emitted label vector are deterministic. Empty when the config is
// missing / unparseable / has no `[[projects]]` table.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
pub(crate) fn read_known_project_name_label_pairs(project_root: &Path) -> Vec<(String, String)> {
    read_name_label_pairs(project_root, "projects")
}

// Read `[[agents]]` entries as a `(name, label)` list from `.aw/config.toml`.
///
// Same shape and contract as `read_known_project_name_label_pairs`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
pub(crate) fn read_known_agent_name_label_pairs(project_root: &Path) -> Vec<(String, String)> {
    read_name_label_pairs(project_root, "agents")
}

// Shared loader for `[[projects]]` / `[[agents]]` tables. Reads the
// `.aw/config.toml`, returns the entries as `(name, label)` pairs.
fn read_name_label_pairs(project_root: &Path, table: &str) -> Vec<(String, String)> {
    read_name_aliases_label(project_root, table)
        .into_iter()
        .map(|(name, _, label)| (name, label))
        .collect()
}

// Shared loader for `[[projects]]` / `[[agents]]` tables. Reads the
// `.aw/config.toml`, returns the entries as `(name, aliases, label)`
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
    let path = project_root.join(".aw/config.toml");
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
// listing all valid names from `.aw/config.toml`. Accepts either the
// canonical `name` or any value listed under that entry's `aliases`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
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
// @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#cli
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
// @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#state-machine
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

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
impl CreateValidationError {
    pub(crate) fn to_envelope_message(&self) -> String {
        match self {
            Self::UnknownProject { name, known } => format!(
                "unknown --project '{}' (valid: {:?}); update .aw/config.toml [[projects]] or pick from the list",
                name, known
            ),
            Self::UnknownAgent { name, known } => format!(
                "unknown --agent '{}' (valid: {:?}); update .aw/config.toml [[agents]] or pick from the list",
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
// @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#state-machine
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
// @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#logic
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

// Pure: returns warning messages for `labels` that violate either
///
// 1. the one-issue-one-project count rule (epics excepted), or
// 2. the project-label-must-match-`[[projects]]` value rule.
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
    let project_labels: Vec<&String> = labels
        .iter()
        .filter(|l| l.starts_with("project:"))
        .collect();

    let mut warnings = Vec::new();

    // Rule 1: count.
    match (issue_type, project_labels.len()) {
        (IssueType::Epic, _) => {} // epics may have any count, including 0
        (_, 1) => {}                // canonical case
        (_, 0) => warnings.push(format!(
            "issue '{}' has no project:* label (non-epic issues should have exactly 1)",
            slug
        )),
        (_, n) => warnings.push(format!(
            "issue '{}' has {} project:* labels {:?} (non-epic issues should have exactly 1; only epics may span multiple)",
            slug, n, project_labels
        )),
    }

    // Rule 2: vocabulary. Each project:* label must appear in
    // `[[projects]].label`. Applies to epics too — a typo'd project name
    // is still a typo regardless of issue type.
    if !known_labels.is_empty() {
        for label in &project_labels {
            if !known_labels.iter().any(|k| k == *label) {
                warnings.push(format!(
                    "issue '{}' has project label '{}' not declared in [[projects]] in .aw/config.toml (known: {:?})",
                    slug, label, known_labels
                ));
            }
        }
    }

    warnings
}

// Side-effecting wrapper: loads the known-projects vocabulary from
// `.aw/config.toml` and prints any warnings to stderr.
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
            // @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R7
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

// ---------------------------------------------------------------------------
// CLI envelope (mainthread ↔ subagent ↔ hook loop)
// ---------------------------------------------------------------------------

// Envelope emitted by `aw wi` verbs that drive the author loop.
// See `projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md`.
///
// `Dispatch.agent` is optional — when `None`, mainthread runs `invoke.command`
// directly (used for approved → `aw wi merge`); when `Some`, mainthread
// spawns `Agent(subagent_type=agent)` with the envelope embedded in the prompt.
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R1 #R2 #R12
#[derive(serde::Serialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum IssueEnvelope<'a> {
    Dispatch {
        #[serde(skip_serializing_if = "Option::is_none")]
        agent: Option<&'a str>,
        slug: &'a str,
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
    out.push_str("review_count: 0\n");
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

// @spec projects/agentic-workflow/tech-design/surface/specs/aw-wi-draft-valid-by-construction.md#draft_authoring_contract
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
            "aw wi create <draft> requires a tracker issue backend; .aw/config.toml resolved to local"
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

async fn run_draft_review(args: DraftReviewArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let (mut issue, meta) = read_draft_issue(&args.draft_path)?;
    let tmp_id = draft_tmp_id(&args.draft_path, &meta)?;
    resolve_project_label(&project_root, &meta.project)
        .map_err(|e| anyhow::anyhow!("{}", e.to_envelope_message()))?;

    let review_body = match (&args.body, &args.body_file) {
        (Some(body), None) => Some(body.clone()),
        (None, Some(path)) => Some(read_body_file(path)?),
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!("clap enforces body/body-file conflict"),
    };

    let Some(review_body) = review_body else {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M").to_string();
        println!("# score-wi-draft-review brief");
        println!();
        println!("Draft:   {}", args.draft_path.display());
        println!("Project: {}", meta.project);
        println!("Title:   {}", issue.title);
        println!();
        println!("## Task");
        println!("Review the draft work-item and decide whether it is ready to publish.");
        println!();
        println!("## Output contract");
        println!("Write ONE top-level list item, then run:");
        println!(
            "  aw wi draft review {} --body-file <file>",
            args.draft_path.display()
        );
        println!();
        println!("Format:");
        println!(
            "- **{} · score-issue-reviewer** — <approved|needs-revision>",
            now
        );
        println!("  - [Requirements] <finding>");
        return Ok(());
    };

    let review_body = review_body.trim_end_matches('\n').to_string();
    let verdict = parse_review_bullet(&review_body)
        .map_err(|e| anyhow::anyhow!("invalid draft review bullet: {}", e))?;
    issue.body = append_review_bullet(&issue.body, &review_body);
    issue.review_count = Some(issue.review_count.unwrap_or(0) + 1);
    if matches!(verdict, ReviewVerdict::NeedsRevision) {
        let flagged = extract_section_tags(&issue.body);
        if !flagged.is_empty() {
            issue.flagged_sections = Some(flagged);
        }
    }

    let content = render_draft_issue_markdown(&issue, &meta.project, &tmp_id);
    write_file_atomically(&args.draft_path, &content)?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "draft_reviewed",
                "project": meta.project,
                "tmp_id": tmp_id,
                "path": args.draft_path,
                "verdict": match verdict {
                    ReviewVerdict::Approved => "approved",
                    ReviewVerdict::NeedsRevision => "needs-revision",
                },
                "review_count": issue.review_count,
            }))?
        );
    } else {
        println!("Draft reviewed: {}", args.draft_path.display());
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
        review_count: Some(0),
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

    let draft_dir = PathBuf::from("/tmp")
        .join("aw")
        .join(&project)
        .join("workitems");
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R1
async fn run_create(args: CreateArgs) -> Result<()> {
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

    // Resolve --project names against [[projects]] in .aw/config.toml.
    let mut project_labels: Vec<String> = Vec::new();
    for name in &args.projects {
        match resolve_project_label(&project_root, name) {
            Ok(label) => project_labels.push(label),
            Err(e) => emit_create_envelope_error(title, &e.to_envelope_message()),
        }
    }

    // Resolve --agent name against [[agents]] in .aw/config.toml.
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
        // CRRR phase starts at `created` so validate can route the first
        // author dispatch to fill Requirements next.
        // @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R2
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

        // @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R1 #R2 #R3
        // Always emit the canonical JSON envelope on stdout — the
        // `--json` flag is retained above only as a deprecated no-op for
        // callers that still pass it. Mainthread reads this envelope and
        // dispatches the named subagent per CLAUDE.md protocol.
        // CRRR loop-fill dispatch: the create envelope kicks off ONE
        // author invocation that fills the full structured body, including
        // capability alignment and agent estimate gates. The
        // mainthread runs `--apply --section all`, then runs `validate` once
        // after the full-body merge.
        // @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R3
        let envelope = IssueEnvelope::Dispatch {
            agent: None,
            slug: &created.slug,
            invoke: Invoke {
                command: "aw wi fill-section",
                args: serde_json::json!({
                    "slug": created.slug,
                    "sections": ["all"],
                }),
            },
        };
        print_envelope(&envelope)?;
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
// Fill-section (envelope loop: subagent round-trip via .aw/payloads/<slug>/body.md)
// ---------------------------------------------------------------------------

// Derive the issue workspace path for `<slug>` under the active workspace mode.
///
// Payload path where the subagent writes filled body for CLI to merge.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R5
fn fill_section_payload_path(project_root: &std::path::Path, slug: &str) -> std::path::PathBuf {
    project_root
        .join(".aw")
        .join("payloads")
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

// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R4 #R5 #R8
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
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R4
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
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R6
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
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R7
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
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R7
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

// Apply mode: read the subagent's payload, merge ONLY the requested sections
// into the issue file, delete the payload, and dispatch mainthread to run
// `aw wi validate` next.
///
// Apply does not commit; WI state is projected through the configured issue
// backend. Format checks run before the merge reaches the issue body.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R5 #R8 #R9
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R7
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
    // @spec projects/agentic-workflow/tech-design/core/validate/section-format-rule.md#requirements
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
        invoke: Invoke {
            command: "aw wi validate",
            args: serde_json::json!({ "slug": slug }),
        },
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Review (envelope loop: reviewer round-trip via .aw/payloads/<slug>/review.md)
// ---------------------------------------------------------------------------

// Payload path where the reviewer writes its single bullet for CLI to merge.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R10
fn review_payload_path(project_root: &std::path::Path, slug: &str) -> std::path::PathBuf {
    project_root
        .join(".aw")
        .join("payloads")
        .join(slug)
        .join("review.md")
}

fn review_payload_template(now: &str) -> String {
    format!(
        "- **{} · score-issue-reviewer** — <verdict>\n  - [<section>] (fill)\n",
        now
    )
}

// Entry point for `aw wi review`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R9 #R10 #R11
async fn run_review(args: ReviewArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = args.slug.clone();
    let worktree_abs = project_root.clone();

    if args.apply {
        run_review_apply(&slug, &worktree_abs).await
    } else {
        run_review_brief(&slug, &worktree_abs).await
    }
}

// Brief mode: print a plain-text review brief for mainthread to consume
// directly (post-Phase-2 mainthread-only model — no reviewer subagent).
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R9
async fn run_review_brief(slug: &str, worktree_abs: &std::path::Path) -> Result<()> {
    let backend = LocalBackend::from_project_root(worktree_abs);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;

    let payload = review_payload_path(worktree_abs, slug);
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M").to_string();
    let payload_created = initialize_payload_file(&payload, &review_payload_template(&now))?;

    println!("# score-issue-reviewer brief");
    println!();
    println!("Issue:      {}  ({})", issue.slug, issue.title);
    println!("Checkout:   {}", worktree_abs.display());
    println!("Issue file: {}", backend.issue_path(&issue).display());
    println!("Output:     {}", payload.display());
    println!(
        "Payload:    {}",
        if payload_created {
            "initialized"
        } else {
            "existing"
        }
    );
    println!();
    println!("## Task");
    println!();
    println!("Review the filled work-item and judge whether:");
    println!(
        "1. Each section (Problem / Requirements / Scope / Reference Context) is clear and concrete."
    );
    println!(
        "2. Every spec listed in `### Related Specs` is **highly relevant** to the Problem — low-relevance entries should be flagged for replacement or removal."
    );
    println!();
    println!(
        "Do NOT hunt for specs that *should* have been linked but weren't — that is out of scope for this review. Only evaluate what the author already listed."
    );
    println!();
    println!("## Output contract");
    println!();
    println!("Write ONE top-level list item to:");
    println!("  {}", payload.display());
    println!();
    println!("Format (exact):");
    println!();
    println!("```markdown");
    println!("- **{} · score-issue-reviewer** — <verdict>", now);
    println!("  - [<section>] <finding — observation + concrete suggestion>");
    println!("  - [<section>] <finding>");
    println!("```");
    println!();
    println!("Where `<verdict>` is one of: `approved` or `needs-revision`.");
    println!();
    println!("Rules:");
    println!("- `approved` → zero findings OR only stylistic nits; sub-bullets are optional.");
    println!("- `needs-revision` → MUST include at least one finding sub-bullet.");
    println!("- `<section>` is one of: `Problem`, `Requirements`, `Scope`, `Reference Context`.");
    println!(
        "- Write ONE bullet only — the CLI appends it to the `# Reviews` list in the issue body."
    );
    println!();
    println!("Do NOT run `aw wi review --apply` yourself — the workflow hook/mainthread");
    println!("invokes it after you return.");

    Ok(())
}

// Apply mode: read the reviewer's payload, validate its shape, append under
// `# Reviews` in the checkout issue body, delete the payload, and dispatch
// mainthread to run `aw wi validate` next.
///
// Apply does not commit; WI state is projected through the configured issue
// backend.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R10 #R11 #R12
async fn run_review_apply(slug: &str, worktree_abs: &std::path::Path) -> Result<()> {
    let payload = review_payload_path(worktree_abs, slug);
    if !payload.exists() {
        print_envelope(&IssueEnvelope::Error {
            slug,
            message: &format!("review payload not found: {}", payload.display()),
        })?;
        return Ok(());
    }

    let review_md = std::fs::read_to_string(&payload)
        .with_context(|| format!("failed to read review payload: {}", payload.display()))?;
    let review_md = review_md.trim_end_matches('\n').to_string();

    // Validate structure: first line must match
    //   `- **<ts> · score-issue-reviewer** — <verdict>`
    if let Err(e) = parse_review_bullet(&review_md) {
        print_envelope(&IssueEnvelope::Error { slug, message: &e })?;
        return Ok(());
    }

    let backend = LocalBackend::from_project_root(worktree_abs);
    let existing = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;

    let new_body = append_review_bullet(&existing.body, &review_md);

    let patch = IssuePatch {
        body: Some(new_body),
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
        invoke: Invoke {
            command: "aw wi validate",
            args: serde_json::json!({ "slug": slug }),
        },
    })?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewVerdict {
    Approved,
    NeedsRevision,
}

// Validate the reviewer's single-bullet payload.
///
// Expected shape:
// ```text
// - **<iso-timestamp> · score-issue-reviewer** — <verdict>
//   - [<section>] <finding>        (optional when verdict=approved)
//   - [<section>] <finding>
// ```
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R11
fn parse_review_bullet(payload: &str) -> std::result::Result<ReviewVerdict, String> {
    let first = payload.lines().next().unwrap_or("").trim_end();
    if first.is_empty() {
        return Err("review payload is empty".into());
    }
    if !first.starts_with("- **") {
        return Err(format!(
            "review payload must start with `- **<ts> · score-issue-reviewer** — <verdict>`, got: {}",
            first.chars().take(120).collect::<String>()
        ));
    }
    let verdict_part = match first.rsplit_once(" — ") {
        Some((_, v)) => v.trim(),
        None => {
            return Err(format!(
                "review payload header missing ` — <verdict>` separator: {}",
                first
            ));
        }
    };
    let verdict = match verdict_part {
        "approved" => ReviewVerdict::Approved,
        "needs-revision" => ReviewVerdict::NeedsRevision,
        other => {
            return Err(format!(
                "invalid verdict '{}'; expected 'approved' or 'needs-revision'",
                other
            ));
        }
    };

    if verdict == ReviewVerdict::NeedsRevision {
        let has_finding = payload
            .lines()
            .skip(1)
            .any(|l| l.trim_start().starts_with("- ["));
        if !has_finding {
            return Err(
                "needs-revision verdict requires at least one finding sub-bullet of the form `  - [<section>] ...`".into(),
            );
        }
    }

    Ok(verdict)
}

// Append a single review bullet under a `# Reviews` H1 at the tail of the
// issue body. Creates the H1 if it doesn't exist, else appends the bullet
// to the existing list in a canonical shape.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md#R11
fn append_review_bullet(body: &str, review_bullet: &str) -> String {
    let trimmed_body = body.trim_end_matches('\n');
    let bullet = review_bullet.trim_end_matches('\n');
    if trimmed_body.contains("\n# Reviews\n") || trimmed_body.starts_with("# Reviews\n") {
        format!("{}\n\n{}\n", trimmed_body, bullet)
    } else {
        format!("{}\n\n---\n\n# Reviews\n\n{}\n", trimmed_body, bullet)
    }
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R3
async fn run_close(args: CloseArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;

    // Close locally first
    let local = make_backend("local", &project_root, None, None)?;
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

    // Fetch the updated issue for output
    let closed_issue = local.get(&args.id).await?;

    // Optionally push to remote
    if args.push {
        if let Some(ref issue) = closed_issue {
            if let Some(remote_id) = issue.github_id.or(issue.gitlab_id) {
                let remote = make_backend("github", &project_root, args.repo.clone(), None)
                    .context("Failed to create remote backend")?;
                if let Err(e) = remote
                    .close(&remote_id.to_string(), args.reason.as_deref())
                    .await
                {
                    if args.json {
                        emit_json_error(&e.to_string(), IssueErrorCode::Backend);
                    }
                    return Err(e);
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

// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R4
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
    capability: String,
    capability_type: String,
    surfaces: String,
    ec_dimensions: String,
    current_state: String,
    gaps: String,
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
    pub warnings: Vec<String>,
    pub agent_review_required: bool,
    pub review_status: &'static str,
    pub plan_command: String,
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
    Ok((backend.name().to_string(), project.to_string(), issues))
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
    let capability_rows =
        crate::cli::capability::capability_rows_for_wi_plan(&capability_document, &td_refs)?;
    let capability_map = CapabilityMap {
        capability_count: capability_document.capabilities.len(),
        rows: capability_rows
            .into_iter()
            .map(|row| CapabilityRow {
                capability: row.capability,
                capability_type: row.capability_type,
                surfaces: row.surfaces,
                ec_dimensions: row.ec_dimensions,
                current_state: row.current_state,
                gaps: row.gaps,
                active_wi: row.active_wi,
                evidence: row.evidence,
                claim_id: row.claim_id,
                claim_user_story: row.claim_user_story,
            })
            .collect(),
        health_note: extract_project_health_note(&cap_body),
    };
    let (backend_name, project, issues, warnings) =
        match load_project_open_issues(&project_root, &project, args.repo.clone()).await {
            Ok((backend_name, project, issues)) => (backend_name, project, issues, Vec::new()),
            Err(err) => (
                "unavailable".to_string(),
                project.clone(),
                Vec::new(),
                vec![format!("issue inventory unavailable: {err:#}")],
            ),
        };
    let title = args
        .title
        .clone()
        .unwrap_or_else(|| format!("{} capability WI plan", project));
    let candidates = capability_wi_candidates(&capability_map.rows, &issues);
    let body = render_capability_wi_plan(
        &project,
        &title,
        &backend_name,
        &cap_path,
        &capability_map,
        &issues,
        &candidates,
        &warnings,
    );
    let path = write_planning_artifact(
        &project,
        "capability-plan",
        &title,
        args.output.as_deref(),
        &body,
    )?;

    let plan_command = capability_wi_plan_command(&project, args.cap_path.as_deref());
    Ok(CapabilityWiPlanReport {
        action: "planned",
        kind: "capability_plan",
        project: project.clone(),
        backend: backend_name,
        path,
        cap_path,
        capability_count: capability_map.capability_count,
        planning_row_count: capability_map.rows.len(),
        issue_count: issues.len(),
        candidate_count: candidates.len(),
        warnings,
        agent_review_required: true,
        review_status: "pending",
        plan_command,
    })
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
    let path = write_planning_artifact(&project, "epics", &title, args.output.as_deref(), &body)?;

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
                "agent_review_required": true,
                "review_status": "pending",
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
    let path = write_planning_artifact(&project, "atomize", &title, args.output.as_deref(), &body)?;

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
                "agent_review_required": true,
                "review_status": "pending",
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
        .unwrap_or_else(|| format!("{} priority review", project));
    let body = render_prioritize_plan(&project, &title, &backend_name, &lanes, &issues);
    let path = write_planning_artifact(
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
                "agent_review_required": true,
                "review_status": "pending",
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

fn priority_rank(issue: &Issue) -> u8 {
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

fn looks_too_large_for_atomic_wi(issue: &Issue) -> bool {
    let text = format!("{}\n{}", issue.title, issue.body).to_ascii_lowercase();
    let large_phrases = [
        "google map",
        "google maps",
        "entire",
        "whole",
        "full platform",
        "complete platform",
        "from scratch",
        "end-to-end product",
        "rewrite all",
        "everything",
    ];
    large_phrases.iter().any(|phrase| text.contains(phrase))
}

#[derive(Debug, Clone, Default)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
pub(crate) struct PrioritizeLanes {
    pub(crate) ready_now: Vec<Issue>,
    pub(crate) blocked_by_dependency: Vec<Issue>,
    pub(crate) needs_atomize: Vec<Issue>,
    pub(crate) needs_triage: Vec<Issue>,
    pub(crate) deferred: Vec<Issue>,
}

/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md#source
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapabilityCandidate {
    title: String,
    issue_type: &'static str,
    source_capability: String,
    capability_gap: String,
    first_gate: String,
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

    let config_file = project_root.join(".aw").join("config.toml");
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

fn capability_wi_candidates(rows: &[CapabilityRow], issues: &[Issue]) -> Vec<CapabilityCandidate> {
    let mut candidates = Vec::new();
    for row in rows {
        if !has_actionable_gap(row) || !matching_issues_for_capability(row, issues).is_empty() {
            continue;
        }
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
            issue_type: infer_candidate_issue_type(&row.gaps),
            source_capability: row.capability.clone(),
            capability_gap: row.gaps.clone(),
            first_gate: if row.claim_id.is_some() {
                row.evidence.clone()
            } else {
                "Create one bounded WI with acceptance criteria and a concrete verification command."
                    .to_string()
            },
        });
    }
    candidates
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
        for reference in active_wi_summary_refs(row) {
            if !summary.existing_wi_refs.contains(&reference) {
                summary.existing_wi_refs.push(reference);
            }
        }
        for issue in matches {
            let reference = issue_ref(issue);
            if !summary.existing_wi_refs.contains(&reference) {
                summary.existing_wi_refs.push(reference);
            }
        }
        let operator =
            suggested_capability_operator(row, &matching_issues_for_capability(row, issues));
        summary.next_operator = merge_capability_plan_operator(&summary.next_operator, operator);
        if summary.first_action == "monitor" {
            if let Some(candidate) = row_candidates.first() {
                summary.first_action = candidate.title.clone();
            } else if has_actionable_gap(row) {
                summary.first_action = row.gaps.clone();
            } else if !summary.existing_wi_refs.is_empty() {
                summary.first_action = "review existing WI linkage".to_string();
            }
        }
    }
    summaries
}

fn active_wi_summary_refs(row: &CapabilityRow) -> Vec<String> {
    if is_empty_active_wi(&row.active_wi.to_ascii_lowercase()) {
        return Vec::new();
    }
    let mut numbers = extract_hash_numbers(&row.active_wi)
        .into_iter()
        .collect::<Vec<_>>();
    if numbers.is_empty() {
        numbers = extract_active_wi_numbers(&row.active_wi);
    }
    numbers.sort_unstable();
    if numbers.is_empty() {
        vec![review_summary_cell(&row.active_wi)]
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
        "monitor" => 1,
        _ => 0,
    };
    if priority(next) > priority(current) {
        next.to_string()
    } else {
        current.to_string()
    }
}

fn review_summary_cell(text: &str) -> String {
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
    let explicit_numbers = extract_hash_numbers(&row.active_wi);
    let active_wi_lower = row.active_wi.to_ascii_lowercase();
    let keywords = capability_keywords(row);

    issues
        .iter()
        .filter(|issue| {
            if issue
                .github_id
                .or(issue.gitlab_id)
                .is_some_and(|id| explicit_numbers.contains(&id))
            {
                return true;
            }

            if !is_empty_active_wi(&active_wi_lower) {
                let issue_ref = issue_ref(issue).to_ascii_lowercase();
                let title = issue.title.to_ascii_lowercase();
                if active_wi_lower.contains(&issue_ref)
                    || active_wi_lower.contains(&issue.slug.to_ascii_lowercase())
                    || (!title.is_empty() && active_wi_lower.contains(&title))
                {
                    return true;
                }
            }

            if let Some(claim_id) = row.claim_id.as_deref() {
                let search = format!("{}\n{}", issue.title, issue.body).to_ascii_lowercase();
                return search.contains(&claim_id.to_ascii_lowercase());
            }

            if keywords.is_empty() {
                return false;
            }
            let search = format!("{}\n{}", issue.title, issue.body).to_ascii_lowercase();
            let hits = keywords
                .iter()
                .filter(|keyword| search.contains(keyword.as_str()))
                .count();
            hits >= 2
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

fn suggested_capability_operator(row: &CapabilityRow, matches: &[&Issue]) -> &'static str {
    if !has_actionable_gap(row) {
        "monitor"
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
    warnings: &[String],
) -> String {
    let mut out = String::new();
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
    if !warnings.is_empty() {
        out.push_str("warnings:\n");
        for warning in warnings {
            out.push_str(&format!("  - {}\n", yaml_quote(warning)));
        }
    }
    out.push_str("agent_review_required: true\n");
    out.push_str("review_status: pending\n");
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n\n", title));
    out.push_str("## Purpose\n\n");
    out.push_str("- Translate the confirmed project capability map into WI planning inputs.\n");
    out.push_str("- Cross-check capability gaps against the current open work-item inventory.\n");
    out.push_str("- Keep this artifact local until a human confirms which WI drafts or tracker updates should publish.\n\n");

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

    out.push_str("## Review Summary\n\n");
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
                markdown_table_cell(&review_summary_cell(&row.first_action))
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
            markdown_table_cell(&row.active_wi),
            markdown_table_cell(&issue_refs(&matches)),
            suggested_capability_operator(row, &matches),
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
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                markdown_table_cell(&candidate.title),
                candidate.issue_type,
                markdown_table_cell(&candidate.source_capability),
                markdown_table_cell(&candidate.capability_gap),
                markdown_table_cell(&candidate.first_gate)
            ));
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
    out.push_str(&format!(
        "1. `aw wi epicize --project {} --title \"{} epics\"`\n",
        project, project
    ));
    out.push_str(&format!(
        "2. `aw wi atomize --project {} --title \"{} bounded WI candidates\"`\n",
        project, project
    ));
    out.push_str(&format!(
        "3. `aw wi prioritize --project {} --title \"{} priority review\"`\n",
        project, project
    ));
    out.push_str(&format!(
        "4. `aw run --project {} --max-ticks 1`\n",
        project
    ));

    out.push_str("\n## Review Guardrails\n\n");
    out.push_str("- Treat README capability rows as the confirmed anchor; if the direction changed, rerun `/aw:capability` before publishing WIs.\n");
    out.push_str("- Convert each accepted candidate through `aw wi draft init` / `aw wi create`; this artifact does not mutate the tracker.\n");
    out.push_str("- Non-epic WIs still need Capability Alignment, Scope, Acceptance Criteria, and Reference Context before `aw td`.\n");
    out
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
    out.push_str("agent_review_required: true\n");
    out.push_str("review_status: pending\n");
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n\n", title));
    out.push_str("## Purpose\n\n");
    out.push_str("- Classify the project work-item inventory into epic candidates.\n");
    out.push_str("- Convert README capability roots into reviewed epic/subepic candidates when a Markdown capability map is available.\n");
    out.push_str(
        "- Identify duplicate, underspecified, or deferred requirements before prioritize.\n",
    );
    out.push_str("- Keep this artifact local until the candidate epics are reviewed.\n\n");
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
    out.push_str("## Required Agent Review Brief\n\n");
    out.push_str("This epic draft requires agent review before publishing tracker changes.\n\n");
    out.push_str(
        "- Merge groups that are clearly one outcome; split groups that mix unrelated goals.\n",
    );
    out.push_str("- Mark duplicate work-items and choose one canonical issue per duplicate set.\n");
    out.push_str("- For each accepted epic candidate, produce title, problem statement, acceptance criteria, included issues, deferred issues, and execution order.\n");
    out.push_str("- Do not publish tracker changes from this artifact without human review.\n");
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
        "- Atomic change WIs are created by `aw wi atomize` after these roots are reviewed.\n\n",
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
    out.push_str("agent_review_required: true\n");
    out.push_str("review_status: pending\n");
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

    out.push_str("\n## Required Human Review\n\n");
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
    out.push_str("agent_review_required: true\n");
    out.push_str("review_status: pending\n");
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n\n", title));
    out.push_str("## Purpose\n\n");
    out.push_str("- Re-rank issue backlog by priority, dependency, and readiness.\n");
    out.push_str("- Identify ready work, blocked dependencies, atomization needs, triage blockers, and deferred work before tracker updates.\n");
    out.push_str(
        "- Keep this artifact local until agent review approves the proposed ordering.\n\n",
    );

    push_prioritize_lane(&mut out, "Ready Now", &lanes.ready_now);
    push_prioritize_lane(
        &mut out,
        "Blocked By Dependency",
        &lanes.blocked_by_dependency,
    );
    push_prioritize_lane(&mut out, "Needs Atomize", &lanes.needs_atomize);
    push_prioritize_lane(&mut out, "Needs Triage", &lanes.needs_triage);
    push_prioritize_lane(&mut out, "Deferred", &lanes.deferred);

    out.push_str("\n## Priority Review Matrix\n\n");
    out.push_str("| Work item | Current priority | Proposed priority | Reason |\n");
    out.push_str("|-----------|------------------|-------------------|--------|\n");
    if issues.is_empty() {
        out.push_str("| none | - | - | - |\n");
    } else {
        for issue in issues {
            out.push_str(&format!(
                "| {} | {} | TBD | Agent review required |\n",
                issue_ref(issue),
                priority_label(issue)
            ));
        }
    }

    out.push_str("\n## Required Agent Review Brief\n\n");
    out.push_str(
        "This priority draft requires agent review before publishing tracker changes.\n\n",
    );
    out.push_str("- Reorder ready work only when dependency or urgency overrides deterministic priority ordering.\n");
    out.push_str(
        "- Keep dependency-blocked work out of the ready lane until the blocker closes.\n",
    );
    out.push_str(
        "- Recommend concrete priority label changes in the matrix with one short reason each.\n",
    );
    out.push_str("- Do not publish tracker changes from this artifact without human review.\n");
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
    project: &str,
    bucket: &str,
    title: &str,
    output: Option<&Path>,
    body: &str,
) -> Result<PathBuf> {
    let path = if let Some(path) = output {
        path.to_path_buf()
    } else {
        let stamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let dir = PathBuf::from("/tmp").join("aw").join(project).join(bucket);
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
// Validate — CRR quality gate (R3, R4)
// ---------------------------------------------------------------------------

// Validate work-item quality and commit the pending current-checkout change
// with the right lifecycle stage and
// emit the next-step dispatch envelope.
///
// Validate is the **sole commit point** in the CRR loop. `--apply` (fill /
// review) merges the subagent payload into the checkout issue but does not
// commit; mainthread runs validate immediately afterwards. On pass, validate
// commits + emits a dispatch envelope (next agent or mainthread step). On
// fail, validate rolls back the checkout-staged issue file and emits an
// Error envelope so mainthread can re-dispatch the same agent.
///
// When invoked outside a lifecycle branch (legacy CLI use, no pending changes), it
// behaves as before — quality check + auto-promote draft→open + text/json
// output, no commit, no envelope.
// REQ: R3 — Issue CRR loop
// REQ: R4 — Quality checks (R-id, scope, spec_plan, no ambiguity)
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

    // Soft check on every validate: warn if project:* label count doesn't
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

// Legacy CRR path: global quality check + draft→open promotion. Used when
// validate is invoked against the main repo (no worktree exists for this
// slug). Preserved so older `/aw:issue update <slug>` flows still work.
// REQ: R3, R4 — Issue CRR loop
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

// Extract the last reviewer bullet (with its sub-bullets) from a `# Reviews`
// block. Returns None if there's no `# Reviews` H1 or no bullets under it.
fn last_review_bullet(body: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let reviews_idx = lines.iter().position(|l| l.trim() == "# Reviews")?;

    let mut last_start: Option<usize> = None;
    for (i, l) in lines.iter().enumerate().skip(reviews_idx + 1) {
        if l.starts_with("- **") {
            last_start = Some(i);
        }
    }
    let start = last_start?;

    let mut end = lines.len();
    for (i, l) in lines.iter().enumerate().skip(start + 1) {
        if l.starts_with("- **") {
            end = i;
            break;
        }
        // Stop if we hit a non-list, non-blank line (likely another section).
        if !l.is_empty() && !l.starts_with(' ') && !l.starts_with("- ") && !l.starts_with("\t") {
            end = i;
            break;
        }
    }
    Some(lines[start..end].join("\n"))
}

// Placeholder substrings that author/reviser stubs use to mark sections
// they haven't filled yet. `validate_section_format` rejects any section
// whose content contains one of these markers — this prevents the
// CRRR loop from advancing past a section that was structurally well-formed
// but semantically empty.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-validator-placeholder-rejection.md#R5
const PLACEHOLDER_MARKERS: &[&str] = &["(fill)", "(replace-this)"];

// Per-section format check used by `validate` after each Fill-* milestone.
// Returns an empty vec on pass, or one error per problem.
///
// Mirrors the per-section rules in
// `crate::services::issue_parser::validate_issue_quality` but scoped to a
// single section so intermediate Fill stages don't fail just because later
// sections aren't filled yet.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R5
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-validator-placeholder-rejection.md
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

// Extract `[Section]` tags from the last reviewer bullet's sub-bullets.
// Returns the unique, sorted list of sections the reviewer flagged. Returns
// empty if no reviewer bullet exists, no sub-bullets are present, or no
// recognized tags are found.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R9
fn extract_section_tags(body: &str) -> Vec<crate::issues::IssueSection> {
    use crate::issues::IssueSection;
    let bullet = match last_review_bullet(body) {
        Some(b) => b,
        None => return Vec::new(),
    };
    let mut found: std::collections::BTreeSet<IssueSection> = std::collections::BTreeSet::new();
    for line in bullet.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("- [") {
            continue;
        }
        let after_bracket = match trimmed.strip_prefix("- [") {
            Some(s) => s,
            None => continue,
        };
        let close = match after_bracket.find(']') {
            Some(i) => i,
            None => continue,
        };
        let tag = &after_bracket[..close];
        if let Some(sec) = IssueSection::parse(tag) {
            found.insert(sec);
        }
    }
    found.into_iter().collect()
}

// ---------------------------------------------------------------------------
// Arbitrate — manual decision after the second `needs-revision` (R12)
// ---------------------------------------------------------------------------

// Arbitrate a stalled CRRR loop. For now only `--send-back` is automated;
// the bare command emits an error envelope with manual-decision instructions
// so a human picks one of force-merge / reject-close / send-back.
///
// `--send-back` is bounded once per slug — a second invocation is rejected
// to avoid infinite loops (R12). On accept it commits
// `Lifecycle-Stage: Reset`, resets `phase=created`, `review_count=0`, clears
// `flagged_sections`, and dispatches the author to refill Requirements.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R12
async fn run_arbitrate(args: ArbitrateArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = args.slug.clone();
    let worktree_path = project_root.clone();

    if !args.send_back {
        print_envelope(&IssueEnvelope::Error {
            slug: &slug,
            message: &format!(
                "manual arbitration required for '{}'. Review the temp issue working copy's Reviews section and run one of: \
                 `aw wi merge --slug {}` (force-merge), \
                 `aw wi close <id>` (reject-close), or \
                 `aw wi arbitrate --slug {} --send-back` (one more author pass).",
                slug, slug, slug
            ),
        })?;
        return Ok(());
    }

    let backend = LocalBackend::from_project_root(&worktree_path);
    if backend.get(&slug).await?.is_none() {
        print_envelope(&IssueEnvelope::Error {
            slug: &slug,
            message: &format!("issue '{}' not found in current checkout", slug),
        })?;
        return Ok(());
    }

    let patch = IssuePatch {
        phase: Some(crate::issues::IssuePhase::Created.as_str().to_string()),
        review_count: Some(0),
        flagged_sections: Some(vec![]),
        validation_errors: Some(vec![]),
        ..Default::default()
    };
    backend.update(&slug, &patch).await?;

    print_envelope(&IssueEnvelope::Dispatch {
        agent: None,
        slug: &slug,
        invoke: Invoke {
            command: "aw wi fill-section",
            args: serde_json::json!({ "slug": slug, "section": "requirements" }),
        },
    })?;
    Ok(())
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
            labels: vec!["type:bug".to_string(), "project:agentic-workflow".to_string()],
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
            "project:agentic-workflow".to_string(),
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
    fn epicize_artifact_requires_agent_review() {
        let issues = vec![planning_issue(
            IssueType::Enhancement,
            "new capability",
            Some("p1"),
            1,
        )];
        let body = render_epicize_plan("score", "Score phase", "github", &issues, None);
        assert!(body.contains("agent_review_required: true"));
        assert!(body.contains("review_status: pending"));
        assert!(body.contains("## Required Agent Review Brief"));
    }

    #[test]
    fn epicize_artifact_includes_markdown_capability_roots() {
        let cap_body = r#"# demo

## Package Manager

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| package-manager | #3779 | auditing | Replace package manager flows. | smoke | projects/jet/validation/pkg-manager.toml |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Lockfile parity | epic | #3779 | partial | planned | smoke | projects/jet/validation/pkg-manager.toml |
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
    fn prioritize_artifact_requires_agent_review_and_orders_all_layers() {
        let issues = vec![
            planning_issue(IssueType::Epic, "phase", Some("p1"), 3),
            planning_issue(IssueType::Bug, "urgent", Some("p0"), 1),
            planning_issue(IssueType::Enhancement, "capability", Some("p2"), 2),
        ];
        let lanes = prioritize_lanes(&issues);
        let body = render_prioritize_plan("score", "Score priorities", "github", &lanes, &issues);
        assert!(body.contains("kind: prioritize"));
        assert!(body.contains("agent_review_required: true"));
        assert!(body.contains("## Ready Now"));
        assert!(body.contains("## Blocked By Dependency"));
        assert!(body.contains("## Needs Atomize"));
        assert!(body.contains("## Needs Triage"));
        assert!(body.contains("## Priority Review Matrix"));
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
                    capability: "Package manager".to_string(),
                    capability_type: "DeveloperTool".to_string(),
                    surfaces: "CLI: `jet install` - install dependencies".to_string(),
                    ec_dimensions: "behavior: `jet test` - package manager conformance<br>efficiency: `meter` - install profile".to_string(),
                    current_state: "Works for lockfile installs".to_string(),
                    gaps: "peer dependency roadmap missing".to_string(),
                    active_wi: "none".to_string(),
                    evidence: "README".to_string(),
                    claim_id: None,
                    claim_user_story: None,
                },
                CapabilityRow {
                    capability: "Dev server".to_string(),
                    capability_type: "-".to_string(),
                    surfaces: "-".to_string(),
                    ec_dimensions: "-".to_string(),
                    current_state: "HMR works".to_string(),
                    gaps: "none".to_string(),
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
            Path::new("/repo/projects/jet/README.md"),
            &map,
            &[],
            &candidates,
            &[],
        );
        assert!(body.contains("kind: capability_plan"));
        assert!(body.contains("capability_count: 2"));
        assert!(body.contains("planning_row_count: 2"));
        assert!(body.contains("## Review Summary"));
        assert!(body.contains("| Package manager | 1 | none | epicize -> atomize | Close capability gap: Package manager |"));
        assert!(body.contains("| Capability | Type | Surfaces | EC Dimensions | Claim |"));
        assert!(body.contains("DeveloperTool"));
        assert!(body.contains("CLI: `jet install` - install dependencies"));
        assert!(body.contains("efficiency: `meter` - install profile"));
        assert!(body.contains("Close capability gap: Package manager"));
        assert!(body.contains("## Recommended CLI Sequence"));
        assert!(body.contains("does not mutate the tracker"));

        let warned = render_capability_wi_plan(
            "jet",
            "Jet capability plan",
            "unavailable",
            Path::new("/repo/projects/jet/README.md"),
            &map,
            &[],
            &candidates,
            &["issue inventory unavailable: gh auth missing".to_string()],
        );
        assert!(warned.contains("warnings:"));
        assert!(warned.contains("## Source"));
        assert!(warned.contains("### Planning Warnings"));
        assert!(warned.contains("issue inventory unavailable: gh auth missing"));
    }

    #[test]
    fn capability_plan_summary_groups_candidates_by_capability() {
        let rows = vec![
            CapabilityRow {
                capability: "Package Manager".to_string(),
                capability_type: "DeveloperTool".to_string(),
                surfaces: "CLI: `jet install`".to_string(),
                ec_dimensions: "behavior: `jet test`".to_string(),
                current_state: "Install surface exists".to_string(),
                gaps: "claim package-manager-readiness: package readiness needs proof".to_string(),
                active_wi: "#3779".to_string(),
                evidence: "claim gate: cargo test -p jet --lib pkg_manager".to_string(),
                claim_id: Some("package-manager-readiness".to_string()),
                claim_user_story: None,
            },
            CapabilityRow {
                capability: "Package Manager".to_string(),
                capability_type: "DeveloperTool".to_string(),
                surfaces: "CLI: `jet install`".to_string(),
                ec_dimensions: "behavior: `jet test`".to_string(),
                current_state: "Workspace support exists".to_string(),
                gaps: "claim package-manager-workspace-parity: workspace parity needs proof"
                    .to_string(),
                active_wi: "3780".to_string(),
                evidence: "claim gate: cargo test -p jet --lib pkg_manager::workspace".to_string(),
                claim_id: Some("package-manager-workspace-parity".to_string()),
                claim_user_story: None,
            },
        ];

        let candidates = capability_wi_candidates(&rows, &[]);
        let summary = capability_plan_summary_rows(&rows, &[], &candidates);

        assert_eq!(candidates.len(), 2);
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].capability, "Package Manager");
        assert_eq!(summary[0].candidate_count, 2);
        assert_eq!(summary[0].existing_wi_refs, vec!["#3779", "#3780"]);
        assert_eq!(summary[0].next_operator, "epicize -> atomize");
        assert_eq!(
            summary[0].first_action,
            "Close capability claim: Package Manager / package-manager-readiness"
        );
    }

    #[test]
    fn capability_wi_plan_command_preserves_cap_path_override() {
        let command =
            capability_wi_plan_command("lumen", Some(Path::new("/tmp/aw plan/lumen README.md")));

        assert_eq!(
            command,
            "aw wi plan --project lumen --cap-path '/tmp/aw plan/lumen README.md'"
        );
    }

    #[test]
    fn capability_matching_uses_active_wi_reference_before_creating_candidate() {
        let row = CapabilityRow {
            capability: "Package manager".to_string(),
            capability_type: "DeveloperTool".to_string(),
            surfaces: "CLI: `jet install`".to_string(),
            ec_dimensions: "behavior: `jet test`".to_string(),
            current_state: "Works for lockfile installs".to_string(),
            gaps: "peer dependency roadmap missing".to_string(),
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
    fn capability_claim_rows_do_not_match_broad_epic_by_keywords_only() {
        let row = CapabilityRow {
            capability: "Package Manager".to_string(),
            capability_type: "DeveloperTool".to_string(),
            surfaces: "CLI: `jet install`".to_string(),
            ec_dimensions: "behavior: `jet test`".to_string(),
            current_state: "Install surface exists".to_string(),
            gaps: "claim package-manager-workspace-parity: workspace package discovery needs a bounded verification WI".to_string(),
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
        let candidates = capability_wi_candidates(&[row], &issues);

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].title,
            "Close capability claim: Package Manager / package-manager-workspace-parity"
        );
    }

    #[test]
    fn resolve_capability_path_uses_cap_path_or_project_readme() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        std::fs::write(
            tmp.path().join(".aw/config.toml"),
            r#"
[[projects]]
name = "jet"
aliases = ["j"]
path = "projects/jet"
label = "project:jet"

[[projects]]
name = "score"
path = "projects/score"
cap_path = "docs/score-cap.md"
label = "project:score"
"#,
        )
        .unwrap();
        assert_eq!(
            resolve_capability_path(tmp.path(), "j", None).unwrap(),
            tmp.path().join("projects/jet/README.md")
        );
        assert_eq!(
            resolve_capability_path(tmp.path(), "score", None).unwrap(),
            tmp.path().join("docs/score-cap.md")
        );
    }

    #[test]
    fn planning_slug_is_filesystem_safe() {
        assert_eq!(planning_slug("Score: Next Run!"), "score-next-run");
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
        let labels = vec!["type:bug".into(), "project:cclab-agent".into()];
        assert!(project_label_warnings(&labels, IssueType::Bug, "demo", &[]).is_empty());
    }

    #[test]
    fn project_label_warnings_non_epic_with_zero_labels_warns() {
        let labels = vec!["type:bug".into()];
        let warnings = project_label_warnings(&labels, IssueType::Enhancement, "demo", &[]);
        assert_eq!(warnings.len(), 1);
        let msg = &warnings[0];
        assert!(msg.contains("no project:*"), "msg was: {}", msg);
        assert!(msg.contains("demo"), "msg should name the slug: {}", msg);
    }

    #[test]
    fn project_label_warnings_non_epic_with_multiple_labels_warns() {
        let labels = vec![
            "type:refactor".into(),
            "project:cclab-agent".into(),
            "project:agentic-workflow".into(),
        ];
        let warnings = project_label_warnings(&labels, IssueType::Refactor, "demo", &[]);
        assert_eq!(warnings.len(), 1);
        let msg = &warnings[0];
        assert!(msg.contains("2 project:*"), "msg should count: {}", msg);
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
            "project:cclab-agent".into(),
            "project:agentic-workflow".into(),
            "project:conductor".into(),
        ];
        assert!(project_label_warnings(&labels, IssueType::Epic, "demo", &[]).is_empty());
    }

    #[test]
    fn project_label_warnings_known_label_passes() {
        let labels = vec!["type:bug".into(), "project:agentic-workflow".into()];
        let known = vec![
            "project:agentic-workflow".into(),
            "project:agentic-workflow".into(),
        ];
        assert!(project_label_warnings(&labels, IssueType::Bug, "demo", &known).is_empty());
    }

    #[test]
    fn project_label_warnings_unknown_label_warns_against_known_set() {
        let labels = vec!["type:bug".into(), "project:typo".into()];
        let known = vec![
            "project:agentic-workflow".into(),
            "project:agentic-workflow".into(),
        ];
        let warnings = project_label_warnings(&labels, IssueType::Bug, "demo", &known);
        assert_eq!(
            warnings.len(),
            1,
            "expected one warning, got {:?}",
            warnings
        );
        let msg = &warnings[0];
        assert!(
            msg.contains("project:typo"),
            "msg should name the bad label: {}",
            msg
        );
        assert!(msg.contains("not declared"), "msg should explain: {}", msg);
        assert!(
            msg.contains("project:agentic-workflow"),
            "msg should list known labels: {}",
            msg
        );
    }

    #[test]
    fn project_label_warnings_unknown_label_with_empty_known_skips_value_check() {
        let labels = vec!["type:bug".into(), "project:typo".into()];
        // Empty known => degrade gracefully, only the count rule fires (and
        // here count=1 is canonical, so no warnings at all).
        assert!(project_label_warnings(&labels, IssueType::Bug, "demo", &[]).is_empty());
    }

    #[test]
    fn project_label_warnings_unknown_label_on_epic_still_warns() {
        let labels = vec!["type:epic".into(), "project:typo".into()];
        let known = vec!["project:agentic-workflow".into()];
        let warnings = project_label_warnings(&labels, IssueType::Epic, "demo", &known);
        assert_eq!(warnings.len(), 1, "epic with bad label should still warn");
        assert!(warnings[0].contains("project:typo"));
    }

    #[test]
    fn project_label_warnings_count_and_value_warnings_combine() {
        // Two unknown labels => one count warning + two value warnings.
        let labels = vec![
            "type:refactor".into(),
            "project:typo-a".into(),
            "project:typo-b".into(),
        ];
        let known = vec!["project:agentic-workflow".into()];
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
        // No .aw/config.toml at all.
        assert!(read_known_project_labels(tmp.path()).is_empty());
    }

    #[test]
    fn read_known_project_labels_no_projects_table_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        std::fs::write(tmp.path().join(".aw/config.toml"), "version = \"0.3.13\"\n").unwrap();
        assert!(read_known_project_labels(tmp.path()).is_empty());
    }

    #[test]
    fn read_known_project_labels_collects_labels() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        std::fs::write(
            tmp.path().join(".aw/config.toml"),
            r#"
[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
label = "project:agentic-workflow"

[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
label = "project:agentic-workflow"

[[projects]]
name = "no-label"
path = "crates/no-label"
"#,
        )
        .unwrap();
        let labels = read_known_project_labels(tmp.path());
        assert_eq!(labels, vec!["project:agentic-workflow", "project:no-label"]);
    }

    // -- score-wi-cli-redesign: typed-flag tests ----------------------------
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-wi-cli-redesign.md#test-plan

    fn write_config(tmp: &std::path::Path, body: &str) {
        std::fs::create_dir_all(tmp.join(".aw")).unwrap();
        std::fs::write(tmp.join(".aw/config.toml"), body).unwrap();
    }

    const CONFIG_WITH_PROJECTS_AND_AGENTS: &str = r#"
[[projects]]
name = "mamba"
label = "project:mamba"

[[projects]]
name = "agentic-workflow"
label = "project:agentic-workflow"

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
        assert_eq!(label, "project:agentic-workflow");
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
label = "project:agentic-workflow"
"#,
        );
        let label = resolve_project_label(tmp.path(), "aw").unwrap();
        assert_eq!(label, "project:agentic-workflow");
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
        assert_eq!(label.as_deref(), Some("project:agentic-workflow"));
    }

    #[test]
    fn list_project_filter_rejects_raw_label_combination() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), CONFIG_WITH_PROJECTS_AND_AGENTS);
        let err = resolve_list_label_filter(
            tmp.path(),
            Some("project:agentic-workflow"),
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
label = "project:agentic-workflow"
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
label = "project:agentic-workflow"
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
                "project:agentic-workflow".to_string(),
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
        assert!(rendered.contains("- 'project:agentic-workflow'"));
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
- 'project:agentic-workflow'\n\
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
            labels: vec!["type:enhancement".to_string(), "project:agentic-workflow".to_string()],
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
    fn initialize_payload_file_creates_parent_and_preserves_existing_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".aw/payloads/123/body.md");

        assert!(initialize_payload_file(&path, "first\n").unwrap());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "first\n");

        assert!(!initialize_payload_file(&path, "second\n").unwrap());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "first\n");
    }

    #[test]
    fn review_payload_template_requires_agent_edit() {
        let template = review_payload_template("2026-06-01T12:00");
        assert!(template.contains("score-issue-reviewer"));
        assert!(template.contains("<verdict>"));
        assert!(template.contains("(fill)"));
        assert!(parse_review_bullet(&template).is_err());
    }

    #[test]
    fn append_review_bullet_marks_draft_review_metadata() {
        let mut issue = Issue {
            issue_type: IssueType::Enhancement,
            title: "demo draft".to_string(),
            state: IssueState::Draft,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec![
                "type:enhancement".to_string(),
                "project:agentic-workflow".to_string(),
            ],
            created_at: None,
            updated_at: None,
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
        let review = "- **2026-05-13T04:28 · score-issue-reviewer** — needs-revision\n  - [Requirements] Tighten R1.";
        let verdict = parse_review_bullet(review).unwrap();
        issue.body = append_review_bullet(&issue.body, review);
        issue.review_count = Some(issue.review_count.unwrap_or(0) + 1);
        if matches!(verdict, ReviewVerdict::NeedsRevision) {
            let flagged = extract_section_tags(&issue.body);
            if !flagged.is_empty() {
                issue.flagged_sections = Some(flagged);
            }
        }

        assert_eq!(issue.review_count, Some(1));
        assert_eq!(
            issue.flagged_sections,
            Some(vec![IssueSection::Requirements])
        );
        assert!(issue.body.contains("# Reviews"));
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
            &["project:agentic-workflow".into()],
            Some("priority:p1"),
            Some("agent::claude-code"),
        );
        assert_eq!(
            labels,
            vec![
                "type:bug",
                "project:agentic-workflow",
                "priority:p1",
                "agent::claude-code"
            ]
        );
    }

    #[test]
    fn build_create_label_vec_skips_optional_when_absent() {
        let labels = build_create_label_vec(
            "type:enhancement",
            &["project:agentic-workflow".into()],
            None,
            None,
        );
        assert_eq!(labels, vec!["type:enhancement", "project:agentic-workflow"]);
    }

    #[test]
    fn build_create_label_vec_dedupes_preserving_first_seen_order() {
        let labels = build_create_label_vec(
            "type:epic",
            &[
                "project:agentic-workflow".into(),
                "project:agentic-workflow".into(),
            ],
            None,
            None,
        );
        assert_eq!(labels, vec!["type:epic", "project:agentic-workflow"]);
    }

    #[test]
    fn build_create_label_vec_epic_multi_project_ordered() {
        let labels = build_create_label_vec(
            "type:epic",
            &["project:agentic-workflow".into(), "project:mamba".into()],
            None,
            None,
        );
        assert_eq!(
            labels,
            vec!["type:epic", "project:agentic-workflow", "project:mamba"]
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
label = "project:agentic-workflow"
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
