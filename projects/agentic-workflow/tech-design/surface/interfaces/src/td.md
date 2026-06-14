---
id: projects-score-src-td-rs
fill_sections: [overview, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "TD CLI surface manifests cover tech-design artifact authoring, validation, review, revision, merge, and arbitration behavior."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "TD/CB CLI surface manifests cover lifecycle dispatch, review, fill, and merge command behavior."
command_refs:
  - command: aw td
  - command: aw td arbitrate
  - command: aw td ast
  - command: aw td check
  - command: aw td claim
  - command: aw td create
  - command: aw td merge
  - command: aw td review
  - command: aw td revise
  - command: aw td validate
---

# Standardized projects/agentic-workflow/src/cli/td.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/td.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ArbitrateArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 258 |  |
| `AstArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 82 |  |
| `AuditArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 277 |  |
| `AuditGroupBy` | projects/agentic-workflow/src/cli/td.rs | enum | pub | 265 |  |
| `CheckArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 220 |  |
| `CreateArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 92 |  |
| `GenCodeArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 202 |  |
| `MergeArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 237 |  |
| `ReviewArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 146 |  |
| `ReviseArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 171 |  |
| `TdArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 23 |  |
| `TdClaimArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 62 |  |
| `TdCommand` | projects/agentic-workflow/src/cli/td.rs | enum | pub | 30 |  |
| `ValidateArgs` | projects/agentic-workflow/src/cli/td.rs | struct | pub | 125 |  |
| `discover_worktree_spec` | projects/agentic-workflow/src/cli/td.rs | function | pub | 740 | discover_worktree_spec(worktree_abs: &std::path::Path) -> Option<String> |
| `run` | projects/agentic-workflow/src/cli/td.rs | function | pub | 2031 | run(args: TdArgs) -> Result<()> |
| `run_audit` | projects/agentic-workflow/src/cli/td.rs | function | pub | 4757 | run_audit(args: AuditArgs) -> Result<()> |
| `run_check` | projects/agentic-workflow/src/cli/td.rs | function | pub | 2085 | run_check(args: CheckArgs) -> Result<()> |
| `run_claim` | projects/agentic-workflow/src/cli/td.rs | function | pub | 6189 | run_claim(args: TdClaimArgs) -> Result<()> |
| `run_gen_code` | projects/agentic-workflow/src/cli/td.rs | function | pub | 4259 | run_gen_code(args: GenCodeArgs) -> Result<()> |
| `td_activate_inplace_allowing_dirty_spec_path` | projects/agentic-workflow/src/cli/td.rs | function | pub | 447 | td_activate_inplace_allowing_dirty_spec_path(     project_root: &std::path::Path,     slug: &str,     spec_path: &str, ) -> Result<()> |
| `td_activate_inplace_if_present` | projects/agentic-workflow/src/cli/td.rs | function | pub | 423 | td_activate_inplace_if_present(     project_root: &std::path::Path,     slug: &str, ) -> Result<()> |
| `td_workspace_path` | projects/agentic-workflow/src/cli/td.rs | function | pub | 376 | td_workspace_path(project_root: &std::path::Path, _slug: &str) -> std::path::PathBuf |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/td.rs -->
````rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
// CODEGEN-BEGIN
//! `aw td` CLI — tech-design lifecycle.
//!
//! Envelope protocol mirrors `aw wi` (dispatch/done/error).

use crate::generate::apply::{is_all_hand_written, run_apply_worktree};
use crate::generate::diagrams::content::{InteractionContent, LogicContent, StateMachineContent};
use crate::generate::frontmatter::extract_mermaid_plus_blocks;
use crate::issues::{
    make_backend, resolve_default_backend, Issue, IssueBackend, IssuePatch, IssueState,
    LocalBackend,
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use super::remote_push::maybe_push_remote;

// ── CLI args ─────────────────────────────────────────────────────────

#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct TdArgs {
    /// Project name for project-scoped TD utility commands such as `td lock`.
    #[arg(long, global = true)]
    pub project: Option<String>,
    #[command(subcommand)]
    pub command: TdCommand,
}

#[derive(Debug, Subcommand)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub enum TdCommand {
    /// Author a tech-design spec (brief or apply mode).
    Create(CreateArgs),
    /// Validate legacy slug lifecycle state or run the read-only TD checker.
    Validate(ValidateArgs),
    /// Review a tech-design spec (brief or apply mode).
    Review(ReviewArgs),
    /// Revise flagged sections of a tech-design spec.
    Revise(ReviseArgs),
    /// Merge an approved tech-design spec back to main.
    Merge(MergeArgs),
    /// Escalate to human after 2nd needs-revision.
    Arbitrate(ArbitrateArgs),
    /// Read-only rule-registry check against `.aw/tech-design/` files.
    /// Accepts a slug (resolved in the current checkout), a single file path, or
    /// a directory. Runs the unified rule registry; no commit, no phase
    /// advance, no envelope. Exits non-zero on any violation.
    /// @spec .aw/tech-design/projects/score/specs/score-namespaces.md#changes
    Check(CheckArgs),
    /// Parse a TD spec file into the unified TDAst and dump it as JSON.
    /// Debug/inspection verb — accepts a file path (no slug context).
    Ast(AstArgs),
    /// Convert legacy mermaid blocks to Mermaid Plus format.
    /// @spec .aw/tech-design/projects/agentic-workflow/generate/diagrams/mermaid_plus/migrate.md#cli
    MigrateMermaid(super::td_migrate::MigrateMermaidArgs),
    /// Write or verify the configured project TD snapshot lock.
    Lock(super::td_lock::TdLockArgs),
    /// Adopt an on-disk TD spec into the score lifecycle.
    Claim(TdClaimArgs),
}

/// Args for `aw td claim <slug>`.
#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct TdClaimArgs {
    /// Issue slug identifying the tech-design to adopt.
    pub slug: String,
    /// Optional path to an existing spec file on disk. When provided, an
    /// issue stub is created if no open issue matches `slug`.
    #[arg(long)]
    pub from_path: Option<String>,
    /// Re-run the claim even when the active TD branch already carries a
    /// Td-Claim trailer. Without this flag the verb is an idempotent no-op.
    #[arg(long)]
    pub force_rebase: bool,
    /// Emit the dispatch envelope as pretty-printed JSON.
    #[arg(long)]
    pub json: bool,
}

/// Args for `aw td ast <path>`.
///
/// @spec .aw/tech-design/projects/agentic-workflow/td_ast/types.md#changes
#[derive(Debug, Args)]
pub struct AstArgs {
    /// Path to the TD spec markdown file (relative or absolute).
    pub path: String,
    /// Pretty-print the JSON output (default: compact).
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct CreateArgs {
    /// Issue slug.
    pub slug: String,
    /// Apply mode: validate the spec in-place and emit dispatch envelope.
    #[arg(long)]
    pub apply: bool,
    /// Path to the spec file (relative to the current checkout root). Required with --apply.
    #[arg(long)]
    pub spec_path: Option<String>,
    /// TD authoring pass. `applicability` fills section applicability/N/A
    /// evidence; `contract` fills the approved contract content.
    #[arg(long)]
    pub phase: Option<String>,
    /// Per-section apply: merge ONLY the section of the given `type:`
    /// annotation from the payload file at
    /// `.aw/payloads/<slug>/<section>.md` into the spec. Other
    /// sections in the spec are untouched. Required for loop-fill flow
    /// where the subagent applies one section at a time.
    #[arg(long)]
    pub section: Option<String>,
    /// DEPRECATED compatibility no-op. TD lifecycle envelopes are JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human authoring brief.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON envelope.
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct ValidateArgs {
    /// Target: issue slug (slug mode, CRRR commit-gate) OR a spec path
    /// (read-only rule check). A value containing `/` or ending in `.md`
    /// is treated as a path; everything else is a slug.
    pub slug: String,
    /// Path to the spec file (relative to the current checkout root). Slug-mode only.
    #[arg(long)]
    pub spec_path: Option<String>,
    /// Emit JSON instead of human-readable output. Path-mode only for now.
    #[arg(long)]
    pub json: bool,
    /// DEPRECATED — replaced by `aw td check <slug>`. When present,
    /// prints a deprecation line on stderr and routes to `td check`. The
    /// flag remains a hidden compat shim for one release.
    /// @spec .aw/tech-design/projects/score/specs/score-namespaces.md#changes
    #[arg(long, hide = true)]
    pub check: bool,
}

#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct ReviewArgs {
    /// Issue slug.
    pub slug: String,
    /// Apply mode: validate the review and emit dispatch envelope.
    #[arg(long)]
    pub apply: bool,
    /// Path to the spec file (relative to the current checkout root). Required.
    #[arg(long)]
    pub spec_path: Option<String>,
    /// Review pass: applicability or contract.
    #[arg(long)]
    pub phase: Option<String>,
    /// DEPRECATED compatibility no-op. TD lifecycle envelopes are JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human review brief.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON envelope.
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct ReviseArgs {
    /// Issue slug.
    pub slug: String,
    /// Apply mode: validate the revision and emit dispatch envelope.
    #[arg(long)]
    pub apply: bool,
    /// Path to the spec file (relative to the current checkout root). Required.
    #[arg(long)]
    pub spec_path: Option<String>,
    /// Per-section apply: merge ONLY the section of the given `type:`
    /// annotation from the payload file at
    /// `.aw/payloads/<slug>/<section>.md` into the spec BEFORE
    /// validation. Mirrors section-queue `aw td create --apply`. Required
    /// for the reviser loop where the subagent rewrites one flagged
    /// section at a time and the validator must see the post-merge
    /// result, not the pre-merge spec at HEAD.
    #[arg(long)]
    pub section: Option<String>,
    /// DEPRECATED compatibility no-op. TD lifecycle envelopes are JSON by default.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Emit the legacy human revise brief.
    #[arg(long)]
    pub human: bool,
    /// Pretty-print the JSON envelope.
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct GenCodeArgs {
    /// Issue slug.
    pub slug: String,
    /// Path to the spec file (relative to the current checkout root).
    #[arg(long)]
    pub spec_path: Option<String>,
}

/// Args for `aw td check <target>` — Phase 1 read-only rule check.
///
/// `--section-type-conformance` switches to the registry-conformance pass
/// (R2 of #1212): walks `.aw/tech-design/**/*.md` under the resolved
/// scope and reports per-spec headings whose `<!-- type: ... -->`
/// annotation is unknown / deprecated / missing.
///
/// @spec .aw/tech-design/projects/score/specs/score-namespaces.md#changes
/// @spec .aw/tech-design/projects/score/specs/aw-td-check-section-type-conformance.md#schema
#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Issue slug, spec file path, or directory to check. Optional when
    /// `--section-type-conformance` is set (defaults to project root).
    #[arg(default_value = "")]
    pub target: String,
    /// Emit findings as a JSON array.
    #[arg(long)]
    pub json: bool,
    /// Run the section-type registry conformance pass instead of the
    /// rule-registry check. Reads
    /// `.aw/tech-design/projects/score/specs/score-section-type-registry.md`.
    #[arg(long)]
    pub section_type_conformance: bool,
}

#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct MergeArgs {
    /// Issue slug.
    pub slug: String,
    /// Path to the spec file (relative to the current checkout root).
    #[arg(long)]
    pub spec_path: Option<String>,
    /// Target branch to merge into. When omitted, the current branch is detected
    /// via `git rev-parse --abbrev-ref HEAD`; detached HEAD falls back to
    /// `[sdd.repo_platform].default_branch` in `.score/config.toml`.
    #[arg(long)]
    pub target_branch: Option<String>,
    /// Allow merging a TD whose Changes section lists files that don't exist
    /// on disk yet (Bug 2 escape hatch). The default refuses, since a 0-of-N
    /// implementation rate signals codegen skipped emission. Pass this flag
    /// only for legitimate spec-only / docs-only merges.
    #[arg(long)]
    pub allow_empty_impl: bool,
}

#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct ArbitrateArgs {
    /// Issue slug.
    pub slug: String,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub enum AuditGroupBy {
    /// Group findings by codegen-gap label (HANDWRITE marker `gap` attribute,
    /// plus the synthetic `missing-spec-marker` for MarkerGap reports).
    Gap,
    /// Group findings by source file path.
    File,
    /// Group findings by status (clean / drift / marker_gap / ...).
    Status,
}

#[derive(Debug, Args)]
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub struct AuditArgs {
    /// Target: code-space prefix (e.g. `projects/mamba/mambalibs/httpkit/`) or single source
    /// file. Emits one report entry per top-level item, classified as
    /// `Clean` / `Drift` / `MarkerGap` / `Uncovered` / `Handwrite`. When
    /// omitted, falls back to the legacy spec-side audit (deprecated — see
    /// --help).
    pub path: Option<String>,
    /// Output as JSON.
    #[arg(long)]
    pub json: bool,
    /// Group findings by gap / file / status. Especially useful with `gap`
    /// to surface remaining codegen-generator gaps.
    #[arg(long, value_enum)]
    pub group_by: Option<AuditGroupBy>,
    /// REMOVED — use path-based audit. Invocation errors with migration hint.
    #[arg(long, hide = true)]
    pub ready_only: bool,
    /// REMOVED — rolled into the default unified walk. Invocation errors.
    #[arg(long, hide = true)]
    pub drift: bool,
}

// ── Envelope (same schema as aw wi) ───────────────────────────────

#[derive(serde::Serialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum TdEnvelope<'a> {
    Dispatch {
        #[serde(skip_serializing_if = "Option::is_none")]
        agent: Option<&'a str>,
        slug: &'a str,
        invoke: Invoke<'a>,
    },
    Done {
        slug: &'a str,
        message: &'a str,
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

fn print_envelope(env: &TdEnvelope<'_>) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(env)?);
    Ok(())
}

fn print_json_value(value: &serde_json::Value, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}

fn next_dispatch(command: String, reason: &str, payload_path: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": command,
        "reason": reason,
        "requires_hitl": false,
        "payload_path": payload_path,
    })
}

fn next_none(reason: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "none",
        "command": null,
        "reason": reason,
        "requires_hitl": false,
        "payload_path": null,
    })
}

fn td_error(slug: &str, message: impl Into<String>) -> Result<()> {
    let message = message.into();
    print_envelope(&TdEnvelope::Error {
        slug,
        message: &message,
    })?;
    Err(anyhow::anyhow!(message))
}

// ── TD workspace path helpers ────────────────────────────────────────

/// Path of the active TD workspace — post-Phase-C this is always the
/// host repo root. Mutating verbs stay on the current project branch
/// unless they are launched from `main`, in which case they activate
/// `td-<slug>` first.
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub(crate) fn td_workspace_path(project_root: &std::path::Path, _slug: &str) -> std::path::PathBuf {
    project_root.to_path_buf()
}

fn workflow_slug_for_issue(issue: &Issue, fallback: &str) -> String {
    issue
        .github_id
        .or(issue.gitlab_id)
        .map(|id| id.to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn should_use_td_branch(current_branch: &str) -> bool {
    current_branch == "main"
}

fn td_branch_name(slug: &str) -> String {
    format!("td-{}", slug)
}

fn activate_td_workspace_for_lifecycle(
    project_root: &std::path::Path,
    workflow_slug: &str,
) -> Result<String> {
    let current_branch = crate::branch_switch::current_branch(project_root)?;
    if should_use_td_branch(&current_branch) {
        let aw = super::slug_workspace::enter_workspace_for_verb(
            project_root,
            crate::issues::slug::BranchKind::Td,
            workflow_slug,
            /*provision_if_missing=*/ true,
        )
        .context("failed to provision tech-design workspace")?;
        Ok(aw.branch)
    } else {
        crate::branch_switch::ensure_branch_clean(project_root)
            .map_err(|e| anyhow::anyhow!("in-place td verb requires clean tree: {}", e))?;
        Ok(current_branch)
    }
}

/// Activate the `td-<slug>` branch only when invoked from `main`.
///
/// Project branches are already isolated working contexts, so mutating TD
/// verbs stay on the current branch and let issue/spec state carry lifecycle
/// progress.
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub(crate) fn td_activate_inplace_if_present(
    project_root: &std::path::Path,
    slug: &str,
) -> Result<()> {
    crate::branch_switch::ensure_branch_clean(project_root)
        .map_err(|e| anyhow::anyhow!("in-place td verb requires clean tree: {}", e))?;
    let current = crate::branch_switch::current_branch(project_root)?;
    if !should_use_td_branch(&current) {
        return Ok(());
    }

    let branch = td_branch_name(slug);
    if !crate::branch_switch::branch_exists_local(project_root, &branch).unwrap_or(false) {
        anyhow::bail!(
            "workspace not found: branch '{}' does not exist (run `aw td create {}` first to provision)",
            branch,
            slug,
        );
    }
    crate::branch_switch::switch_or_create_branch(project_root, &branch, &current)?;
    Ok(())
}

/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub(crate) fn td_activate_inplace_allowing_dirty_lifecycle_paths(
    project_root: &std::path::Path,
    slug: &str,
    allowed_rel: &[&str],
) -> Result<()> {
    ensure_clean_or_only_dirty_paths(project_root, allowed_rel).map_err(|e| {
        anyhow::anyhow!(
            "in-place td verb requires clean tree or only matching lifecycle-state files \
             (allowed: {:?}): {}",
            allowed_rel
                .iter()
                .map(|p| normalize_checkout_rel_path(p))
                .collect::<Vec<_>>(),
            e
        )
    })?;
    let current = crate::branch_switch::current_branch(project_root)?;
    if !should_use_td_branch(&current) {
        return Ok(());
    }

    let branch = td_branch_name(slug);
    if !crate::branch_switch::branch_exists_local(project_root, &branch).unwrap_or(false) {
        anyhow::bail!(
            "workspace not found: branch '{}' does not exist (run `aw td create {}` first to provision)",
            branch,
            slug,
        );
    }
    crate::branch_switch::switch_or_create_branch(project_root, &branch, &current)?;
    Ok(())
}

/// @spec .aw/tech-design/projects/score/specs/aw-td-extend-dirty-allow-issue-file.md#logic
pub(crate) fn td_activate_inplace_allowing_dirty_spec_path(
    project_root: &std::path::Path,
    slug: &str,
    spec_path: &str,
) -> Result<()> {
    let issue_path = canonical_issue_path_for_slug(project_root, slug);
    let payload_prefix = format!(".aw/payloads/{slug}/");
    let allowed: Vec<&str> = match issue_path.as_deref() {
        Some(p) => vec![spec_path, p, payload_prefix.as_str()],
        None => vec![spec_path, payload_prefix.as_str()],
    };
    ensure_clean_or_only_dirty_paths(project_root, &allowed).map_err(|e| {
        anyhow::anyhow!(
            "in-place td verb requires clean tree or only the dirty spec path \
             plus matching lifecycle-state files (allowed: {:?}): {}",
            allowed
                .iter()
                .map(|p| normalize_checkout_rel_path(p))
                .collect::<Vec<_>>(),
            e
        )
    })?;
    let current = crate::branch_switch::current_branch(project_root)?;
    if !should_use_td_branch(&current) {
        return Ok(());
    }

    let branch = td_branch_name(slug);
    if !crate::branch_switch::branch_exists_local(project_root, &branch).unwrap_or(false) {
        anyhow::bail!(
            "workspace not found: branch '{}' does not exist (run `aw td create {}` first to provision)",
            branch,
            slug,
        );
    }
    crate::branch_switch::switch_or_create_branch(project_root, &branch, &current)?;
    Ok(())
}

/// The retired checkout `.aw/issues` tree is no longer a lifecycle dirty-path
/// exception. Issue working copies now live under the temp-backed
/// [`LocalBackend`] store, outside git status.
fn canonical_issue_path_for_slug(_project_root: &std::path::Path, _slug: &str) -> Option<String> {
    None
}

fn issue_path_arg(backend: &LocalBackend, issue: &crate::issues::Issue) -> String {
    if !issue.slug.is_empty() {
        return backend.issue_path(issue).to_string_lossy().into_owned();
    }
    let mut issue = issue.clone();
    issue.slug = issue
        .github_id
        .or(issue.gitlab_id)
        .map(|id| id.to_string())
        .or_else(|| issue.id.clone())
        .unwrap_or_else(|| "unknown".to_string());
    backend.issue_path(&issue).to_string_lossy().into_owned()
}

/// Permit at most the given set of checkout-relative lifecycle-state paths
/// to be dirty. Any dirty path outside the allowed set is a hard error.
/// Pass an empty slice to require a fully clean tree.
fn ensure_clean_or_only_dirty_paths(
    project_root: &std::path::Path,
    allowed_rel: &[&str],
) -> Result<()> {
    let git = crate::git::find_git_bin().context("git binary not found on PATH")?;
    let output = std::process::Command::new(&git)
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output()
        .with_context(|| format!("running git status in {}", project_root.display()))?;
    if !output.status.success() {
        anyhow::bail!(
            "git status failed in {}: {}",
            project_root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let porcelain = String::from_utf8_lossy(&output.stdout).into_owned();
    let dirty_paths: Vec<String> = porcelain
        .lines()
        .filter_map(porcelain_path)
        .map(normalize_checkout_rel_path)
        .collect();
    if dirty_paths.is_empty() {
        return Ok(());
    }

    let allowed_norm: Vec<String> = allowed_rel
        .iter()
        .map(|p| normalize_checkout_rel_path(p))
        .collect();
    if dirty_paths.iter().all(|d| {
        allowed_norm.contains(d)
            || allowed_norm
                .iter()
                .any(|allowed| allowed.ends_with('/') && d.starts_with(allowed))
            || (d.ends_with('/') && allowed_norm.iter().any(|allowed| allowed.starts_with(d)))
    }) {
        return Ok(());
    }

    anyhow::bail!(
        "working tree at {} is dirty outside allowed paths {:?}; porcelain output:\n{}",
        project_root.display(),
        allowed_norm,
        porcelain
    )
}

fn porcelain_path(line: &str) -> Option<&str> {
    let path = line.get(3..)?.trim();
    if path.is_empty() {
        return None;
    }
    Some(path.rsplit_once(" -> ").map_or(path, |(_, dst)| dst.trim()))
}

fn normalize_checkout_rel_path(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .replace(std::path::MAIN_SEPARATOR, "/")
}

/// Activate TD lifecycle state for `slug`, updating the issue's frontmatter
/// to `phase: td_inited` + active branch and committing a `Td-Init` lifecycle
/// trailer. A fresh `aw td create <slug>` does this itself.
///
/// Auto-heals stale frontmatter: if the issue says it has an active `td_*`
/// phase but the branch does not exist, the frontmatter is a leftover from a
/// prior aborted run — scrub it (commit a `Td-Reset` trailer on the current
/// checkout) and continue with fresh branch activation.
async fn provision_td_workspace(
    project_root: &std::path::Path,
    issue_ref: &str,
    workflow_slug: &str,
    branch: &str,
) -> Result<()> {
    use crate::issues::IssueBackend;

    let backend = LocalBackend::from_project_root(project_root);
    let mut issue = backend.get(issue_ref).await?.ok_or_else(|| {
        anyhow::anyhow!(
            "work-item '{}' not found in the temp issue store (file it first via `aw wi create`)",
            issue_ref
        )
    })?;
    // Guard: state must be open.
    if issue.state != IssueState::Open {
        let state_str = match issue.state {
            IssueState::Open => "open",
            IssueState::Closed => "closed",
            IssueState::Draft => "draft",
        };
        anyhow::bail!(
            "issue '{}' is state:{}, must be state:open before starting tech-design",
            issue_ref,
            state_str
        );
    }

    // Guard: no active td_ phase. Auto-heal stale frontmatter when the
    // issue claims an active phase but neither workspace nor branch exist.
    if let Some(phase) = issue.phase.clone() {
        if phase.starts_with("td_") {
            let worktree_abs = td_workspace_path(project_root, workflow_slug);
            let branch_present =
                crate::branch_switch::branch_exists_local(project_root, branch).unwrap_or(false);
            let stale = !worktree_abs.exists() && !branch_present;
            if stale {
                issue.phase = None;
                issue.branch = None;
                backend.write(&issue).await?;
                let issue_path = backend.issue_path(&issue);
                let issue_path_s = issue_path.to_string_lossy().into_owned();
                maybe_push_remote(project_root, &issue_path, workflow_slug).await?;
                commit_lifecycle_with_extra(
                    project_root,
                    workflow_slug,
                    &issue.title,
                    "Td-Reset",
                    &[issue_path_s.as_str()],
                    &[
                        ("Reset-Reason", "stale-frontmatter"),
                        ("Reset-From-Phase", phase.as_str()),
                    ],
                )?;
            } else {
                anyhow::bail!(
                    "issue '{}' already has active tech-design (phase: {})",
                    issue_ref,
                    phase
                );
            }
        }
    }

    // Provision: only split to `td-<slug>` when starting from `main`.
    // Project branches stay as the active TD workspace.
    let active_branch = activate_td_workspace_for_lifecycle(project_root, workflow_slug)?;

    // Set phase + branch on the issue, on the workspace.
    let wt_backend = LocalBackend::from_project_root(project_root);
    let patch = IssuePatch {
        phase: Some("td_inited".to_string()),
        branch: Some(active_branch),
        clear_transient: true,
        ..Default::default()
    };
    wt_backend.update(issue_ref, &patch).await?;

    // Commit Td-Init.
    let issue_path = wt_backend.issue_path(&issue);
    let issue_path_s = issue_path.to_string_lossy().into_owned();
    maybe_push_remote(project_root, &issue_path, workflow_slug).await?;
    commit_lifecycle(
        project_root,
        workflow_slug,
        &issue.title,
        "Td-Init",
        &[issue_path_s.as_str()],
    )?;

    Ok(())
}

async fn bootstrap_td_issue(project_root: &std::path::Path, issue_ref: &str) -> Result<Issue> {
    let local = LocalBackend::from_project_root(project_root);
    if let Some(issue) = local.get(issue_ref).await? {
        return Ok(issue);
    }

    let (kind, repo, host) = resolve_default_backend(project_root).with_context(|| {
        format!("issue '{issue_ref}' not found in workspace and no issue backend is configured")
    })?;
    if kind == "local" {
        anyhow::bail!("issue '{}' not found in workspace", issue_ref);
    }
    let remote = make_backend(&kind, project_root, repo, host)
        .context("failed to create configured issue backend")?;
    let mut issue = remote.get(issue_ref).await?.ok_or_else(|| {
        anyhow::anyhow!("issue '{}' not found in workspace or {}", issue_ref, kind)
    })?;
    let slug = workflow_slug_for_issue(&issue, issue_ref);
    if issue.state != IssueState::Open {
        let state = match issue.state {
            IssueState::Open => "open",
            IssueState::Closed => "closed",
            IssueState::Draft => "draft",
        };
        anyhow::bail!(
            "issue '{}' is state:{}, must be state:open before starting tech-design",
            issue_ref,
            state
        );
    }

    // Pass the lifecycle clean-tree / branch-activation gate before creating
    // the local working copy, otherwise hydration itself would make the tree
    // dirty and block in-place TD mode.
    let active_branch = activate_td_workspace_for_lifecycle(project_root, &slug)?;
    issue.slug = slug.clone();
    local.write(&issue).await?;

    let issue_path = local.issue_path(&issue);
    let issue_path_s = issue_path.to_string_lossy().into_owned();
    commit_lifecycle_with_extra(
        project_root,
        &slug,
        &format!("hydrate remote issue {issue_ref}"),
        "Td-Hydrate",
        &[issue_path_s.as_str()],
        &[
            ("Issue-Backend", kind.as_str()),
            ("Issue-Ref", issue_ref),
            ("Active-Branch", active_branch.as_str()),
        ],
    )?;

    local
        .get(&slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("hydrated issue '{}' was not readable", slug))
}

/// Find the TD spec file in the current checkout. Used as fallback when
/// callers didn't pass `--spec-path`. Returns the checkout-relative path
/// of the unique spec under `.aw/tech-design/` that the current branch added or modified
/// relative to its parent. Returns None if zero or multiple candidates.
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub(crate) fn discover_worktree_spec(worktree_abs: &std::path::Path) -> Option<String> {
    let git_bin = crate::git::find_git_bin()?;
    let out = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree_abs)
        .args([
            "diff",
            "--name-only",
            "--diff-filter=AM",
            "main...HEAD",
            "--",
            ".aw/tech-design",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let candidates: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| {
            let bn = std::path::Path::new(l).file_name().and_then(|s| s.to_str());
            l.ends_with(".md")
                && !matches!(
                    bn,
                    Some("AUTHORING.md") | Some("README.md") | Some("CHANGELOG.md")
                )
        })
        .map(|l| l.to_string())
        .collect();
    if candidates.len() == 1 {
        Some(candidates.into_iter().next().unwrap())
    } else {
        None
    }
}

// ── Lifecycle commit ─────────────────────────────────────────────────

fn commit_lifecycle(
    worktree_path: &std::path::Path,
    slug: &str,
    detail: &str,
    stage: &str,
    paths: &[&str],
) -> Result<()> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;

    stage_lifecycle_paths(worktree_path, &git_bin, paths)?;

    let msg = format!(
        "td({slug}) \u{2014} {detail}\n\n\
         Lifecycle-Slug: {slug}\n\
         Work-Item: {slug}\n\
         Lifecycle-Stage: {stage}",
    );

    let mut command = std::process::Command::new(&git_bin);
    command.arg("-C").arg(worktree_path).arg("commit");
    if !has_staged_changes(worktree_path, &git_bin)? {
        command.arg("--allow-empty");
    }
    let commit = command
        .args(["-m", &msg])
        .output()
        .context("git commit failed")?;
    if !commit.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit.stderr).trim()
        );
    }
    Ok(())
}

fn stage_lifecycle_paths(
    worktree_path: &std::path::Path,
    git_bin: &std::path::Path,
    paths: &[&str],
) -> Result<()> {
    for p in paths {
        if !should_stage_lifecycle_path(worktree_path, p) {
            continue;
        }
        let add = std::process::Command::new(git_bin)
            .arg("-C")
            .arg(worktree_path)
            .args(["add", p])
            .output()
            .context("git add failed")?;
        if !add.status.success() {
            anyhow::bail!(
                "git add '{}' failed: {}",
                p,
                String::from_utf8_lossy(&add.stderr).trim()
            );
        }
    }
    Ok(())
}

fn should_stage_lifecycle_path(worktree_path: &std::path::Path, path: &str) -> bool {
    let path_obj = std::path::Path::new(path);
    let normalized = if path_obj.is_absolute() {
        let Ok(rel) = path_obj.strip_prefix(worktree_path) else {
            return false;
        };
        normalize_checkout_rel_path(&rel.to_string_lossy())
    } else {
        normalize_checkout_rel_path(path)
    };
    normalized != ".aw/issues" && !normalized.starts_with(".aw/issues/")
}

fn has_staged_changes(worktree_path: &std::path::Path, git_bin: &std::path::Path) -> Result<bool> {
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(worktree_path)
        .args(["diff", "--cached", "--quiet", "--exit-code"])
        .output()
        .context("git diff --cached failed")?;
    Ok(!output.status.success())
}

// ── Spec validation helpers ──────────────────────────────────────────

/// Parsed section annotation from `<!-- type: X lang: Y -->`.
struct SectionAnnotation {
    section_type: String,
    lang: String,
}

/// Parse YAML frontmatter from a markdown file. Returns (frontmatter_str, body).
fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_open = &trimmed[3..];
    let close = after_open.find("\n---")?;
    let fm = &after_open[..close];
    let body = &after_open[close + 4..];
    Some((fm.trim(), body))
}

/// Extract section annotations from a markdown body.
fn extract_sections(body: &str) -> Vec<(String, SectionAnnotation, String)> {
    let mut sections = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if let Some(open) = markdown_fence_open_marker(lines[i]) {
            i += 1;
            while i < lines.len() {
                if markdown_fence_closes(lines[i], &open) {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        if lines[i].starts_with("## ") {
            let heading = lines[i][3..].trim().to_string();
            // Check next line for annotation
            if i + 1 < lines.len() {
                if let Some(ann) = parse_annotation(lines[i + 1]) {
                    // Collect content until next ## or EOF
                    let content_start = i + 2;
                    let mut content_end = content_start;
                    let mut fence_open: Option<String> = None;
                    while content_end < lines.len() {
                        if let Some(open) = &fence_open {
                            if markdown_fence_closes(lines[content_end], open) {
                                fence_open = None;
                            }
                            content_end += 1;
                            continue;
                        }
                        if let Some(open) = markdown_fence_open_marker(lines[content_end]) {
                            fence_open = Some(open);
                            content_end += 1;
                            continue;
                        }
                        if lines[content_end].starts_with("## ") {
                            break;
                        }
                        content_end += 1;
                    }
                    let content: String = lines[content_start..content_end].join("\n");
                    sections.push((heading, ann, content.trim().to_string()));
                    i = content_end;
                    continue;
                }
            }
        }
        i += 1;
    }
    sections
}

fn markdown_fence_open_marker(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let first = trimmed.as_bytes().first().copied()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let count = trimmed
        .as_bytes()
        .iter()
        .take_while(|byte| **byte == first)
        .count();
    if count < 3 {
        return None;
    }
    Some(trimmed[..count].to_string())
}

fn markdown_fence_closes(line: &str, opener: &str) -> bool {
    let Some(marker) = markdown_fence_open_marker(line) else {
        return false;
    };
    marker.as_bytes().first() == opener.as_bytes().first()
        && marker.len() >= opener.len()
        && line.trim_start()[marker.len()..].trim().is_empty()
}

/// Replace the section matching `section_type` in `base_body` with
/// `payload_body`. Section match = the `<!-- type: X -->` annotation
/// line immediately under an `## H2`. If no such section exists in the
/// base, append the payload at the end (the brand-new-section case
/// triggered on the first `--apply --section X` for a given type).
///
/// `payload_body` is expected to be a standalone section fragment —
/// starting with `## <Heading>` followed by its annotation — but the
/// helper doesn't parse beyond the type match: it replaces the entire
/// `## Heading … (next H2 or EOF)` byte range verbatim.
///
/// Used by `aw td create --apply --section X` to drive the loop-fill
/// workflow (subagent writes one payload per section; mainthread/subagent
/// merges them sequentially without overwriting earlier sections).
fn merge_spec_section(base_body: &str, section_type: &str, payload_body: &str) -> Result<String> {
    // Normalize payload so it always ends with exactly one newline; the
    // surrounding base body generally does the same, and merged output
    // should not accidentally run sections together.
    let payload_norm = {
        let trimmed = payload_body.trim_end_matches('\n');
        format!("{}\n", trimmed)
    };

    // Walk base lines to find every `## H\n<!-- type: <section_type> -->`
    // range. Preserve original line terminators via split_inclusive.
    let lines: Vec<&str> = base_body.split_inclusive('\n').collect();
    let mut matches: Vec<(usize, usize)> = Vec::new();
    for i in 0..lines.len() {
        if !lines[i].starts_with("## ") {
            continue;
        }
        let Some(next) = lines.get(i + 1) else {
            continue;
        };
        let Some(ann) = parse_annotation(next.trim_end()) else {
            continue;
        };
        if ann.section_type != section_type {
            continue;
        }
        let mut end = lines.len();
        for j in (i + 1)..lines.len() {
            if lines[j].starts_with("## ") {
                end = j;
                break;
            }
        }
        matches.push((i, end));
    }

    let merged = if let Some((first_start, first_end)) = matches.first().copied() {
        // Replace the first matching section with payload_norm and drop any
        // duplicate matching sections already present from an interrupted
        // retry. This keeps section replay idempotent after a failed apply.
        let mut out: String = lines[..first_start].concat();
        out.push_str(&payload_norm);
        let mut cursor = first_end;
        for (dup_start, dup_end) in matches.iter().skip(1).copied() {
            out.push_str(&lines[cursor..dup_start].concat());
            cursor = dup_end;
        }
        out.push_str(&lines[cursor..].concat());
        out
    } else {
        // No existing section — append. Ensure a blank line between
        // the previous block and the new section for readability.
        let mut out = ensure_fill_sections_has_section(base_body, section_type);
        if !out.ends_with("\n\n") {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }
        out.push_str(&payload_norm);
        out
    };
    Ok(ensure_fill_sections_has_section(&merged, section_type))
}

fn ensure_fill_sections_has_section(content: &str, section_type: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    if lines.first().map(|line| line.trim()) != Some("---") {
        return content.to_string();
    }
    let Some(frontmatter_end) = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(idx, line)| (line.trim() == "---").then_some(idx))
    else {
        return content.to_string();
    };

    for idx in 1..frontmatter_end {
        let trimmed = lines[idx].trim_start();
        let Some(rest) = trimmed.strip_prefix("fill_sections:") else {
            continue;
        };
        let indent_len = lines[idx].len() - trimmed.len();
        let indent = &lines[idx][..indent_len];
        let rest = rest.trim();
        if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            let mut sections: Vec<String> = inner
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            if !sections.iter().any(|section| section == section_type) {
                sections.push(section_type.to_string());
                lines[idx] = format!("{indent}fill_sections: [{}]", sections.join(", "));
            }
            return finish_lines(lines, content.ends_with('\n'));
        }
        return content.to_string();
    }

    lines.insert(frontmatter_end, format!("fill_sections: [{section_type}]"));
    finish_lines(lines, content.ends_with('\n'))
}

fn finish_lines(lines: Vec<String>, trailing_newline: bool) -> String {
    let mut out = lines.join("\n");
    if trailing_newline {
        out.push('\n');
    }
    out
}

/// Parse `<!-- type: X lang: Y -->` annotation line.
fn parse_annotation(line: &str) -> Option<SectionAnnotation> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    let mut section_type = None;
    let mut lang = None;
    for part in inner.split_whitespace() {
        if part == "type:" {
            continue;
        }
        if part == "lang:" {
            continue;
        }
        if section_type.is_none() && inner.contains(&format!("type: {}", part)) {
            section_type = Some(part.to_string());
        } else if lang.is_none() && inner.contains(&format!("lang: {}", part)) {
            lang = Some(part.to_string());
        }
    }
    Some(SectionAnnotation {
        section_type: section_type?,
        lang: lang?,
    })
}

enum MissingSectionPolicy<'a> {
    RequireAll,
    RequireThrough(&'a str),
}

#[derive(Debug, Clone, Copy)]
enum TdContentValidationScope<'a> {
    Complete,
    RequireThrough(&'a str),
}

/// Validate a spec file and return a list of errors (empty = valid).
fn validate_spec(spec_content: &str) -> Vec<String> {
    validate_spec_inner(spec_content, MissingSectionPolicy::RequireAll)
}

/// Validate a section-apply intermediate. Sections up to and including
/// `current_section` must exist and be non-empty; later fill_sections may still
/// be absent because the CLI applies them one at a time.
fn validate_spec_for_section_apply(spec_content: &str, current_section: &str) -> Vec<String> {
    validate_spec_inner(
        spec_content,
        MissingSectionPolicy::RequireThrough(current_section),
    )
}

fn validate_td_content_file(
    spec_path: &std::path::Path,
    scope: TdContentValidationScope<'_>,
) -> Result<crate::validate::RuleReport> {
    let content = std::fs::read_to_string(spec_path)
        .with_context(|| format!("failed to read spec file: {}", spec_path.display()))?;
    validate_td_content(spec_path, &content, scope)
}

fn validate_td_content(
    spec_path: &std::path::Path,
    content: &str,
    scope: TdContentValidationScope<'_>,
) -> Result<crate::validate::RuleReport> {
    let mut report = crate::validate::RuleReport::new();

    let legacy_errors = match scope {
        TdContentValidationScope::Complete => validate_spec(&content),
        TdContentValidationScope::RequireThrough(section) => {
            validate_spec_for_section_apply(content, section)
        }
    };
    for error in legacy_errors {
        report.push(crate::validate::Finding::error(
            crate::validate::RuleId::SectionFormat,
            spec_path,
            error,
        ));
    }
    for warning in legacy_test_section_warnings(&content) {
        report.push(crate::validate::Finding {
            rule: crate::validate::RuleId::SectionFormat,
            file: spec_path.to_path_buf(),
            line: None,
            path: None,
            message: warning,
            severity: crate::validate::Severity::Warning,
        });
    }

    let project_root = crate::find_project_root()?;
    for error in crate::cli::capability::validate_td_capability_refs_for_spec_path(
        &project_root,
        spec_path,
        &content,
    ) {
        report.push(crate::validate::Finding::error(
            crate::validate::RuleId::SectionFormat,
            spec_path,
            error,
        ));
    }

    report.extend(crate::validate::run_rules(&[spec_path.to_path_buf()]));
    Ok(report)
}

fn validate_new_td_authoring_content(
    spec_path: &std::path::Path,
    content: &str,
    scope: TdContentValidationScope<'_>,
) -> Result<crate::validate::RuleReport> {
    let mut report = validate_td_content(spec_path, content, scope)?;
    for error in new_td_forbidden_section_errors(content) {
        report.push(crate::validate::Finding::error(
            crate::validate::RuleId::SectionFormat,
            spec_path,
            error,
        ));
    }
    Ok(report)
}

fn validate_new_td_authoring_file(
    spec_path: &std::path::Path,
    scope: TdContentValidationScope<'_>,
) -> Result<crate::validate::RuleReport> {
    let content = std::fs::read_to_string(spec_path)
        .with_context(|| format!("failed to read spec file: {}", spec_path.display()))?;
    validate_new_td_authoring_content(spec_path, &content, scope)
}

fn new_td_forbidden_section_errors(spec_content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let Some((fm_str, body)) = split_frontmatter(spec_content) else {
        return errors;
    };
    if let Ok(fm) = serde_yaml::from_str::<serde_yaml::Value>(fm_str) {
        if let Some(seq) = fm.get("fill_sections").and_then(|v| v.as_sequence()) {
            for value in seq {
                if value.as_str() == Some("scenarios") {
                    errors.push(
                        "new TDs must not include legacy prose section type 'scenarios'; encode behavior as logic/unit-test/e2e-test artifacts"
                            .to_string(),
                    );
                }
            }
        }
    }
    for (_heading, ann, _content) in extract_sections(body) {
        if ann.section_type == "scenarios" {
            errors.push(
                "new TDs must not author section type 'scenarios'; legacy specs may retain it, but new specs must use artifact-driving sections"
                    .to_string(),
            );
        }
    }
    errors.sort();
    errors.dedup();
    errors
}

fn td_content_error_messages(report: &crate::validate::RuleReport) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|finding| finding.severity == crate::validate::Severity::Error)
        .map(|finding| finding.message.clone())
        .collect()
}

fn legacy_test_section_warnings(spec_content: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some((fm_str, body)) = split_frontmatter(spec_content) {
        if let Ok(fm) = serde_yaml::from_str::<serde_yaml::Value>(fm_str) {
            if let Some(seq) = fm.get("fill_sections").and_then(|v| v.as_sequence()) {
                for value in seq {
                    if let Some(section) = value.as_str() {
                        if section == "test-plan" || section == "tests" {
                            warnings.push(format!(
                                "fill_sections contains legacy section type `{section}`; use `unit-test` or `e2e-test` for new TDs"
                            ));
                        }
                    }
                }
            }
        }
        for (_heading, ann, _content) in extract_sections(body) {
            if ann.section_type == "test-plan" || ann.section_type == "tests" {
                warnings.push(format!(
                    "section annotation uses legacy type `{}`; use `unit-test` or `e2e-test` for new TDs",
                    ann.section_type
                ));
            }
        }
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

fn print_td_content_errors(label: &str, report: &crate::validate::RuleReport) {
    eprintln!("{label}:");
    for finding in report
        .findings
        .iter()
        .filter(|finding| finding.severity == crate::validate::Severity::Error)
    {
        eprintln!("  - {}", finding.format());
    }
}

fn validate_spec_inner(
    spec_content: &str,
    missing_policy: MissingSectionPolicy<'_>,
) -> Vec<String> {
    let mut errors = Vec::new();
    let require_all_sections = matches!(missing_policy, MissingSectionPolicy::RequireAll);

    // 1. Parse frontmatter
    let (fm_str, body) = match split_frontmatter(spec_content) {
        Some(pair) => pair,
        None => {
            errors.push("missing YAML frontmatter (---...---)".to_string());
            return errors;
        }
    };

    let fm: serde_yaml::Value = match serde_yaml::from_str(fm_str) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("invalid YAML frontmatter: {}", e));
            return errors;
        }
    };

    // 2. Check required frontmatter fields
    if fm.get("id").and_then(|v| v.as_str()).is_none() {
        errors.push("frontmatter missing 'id' field".to_string());
    }

    let fill_sections: Vec<String> = match fm.get("fill_sections") {
        Some(serde_yaml::Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => {
            errors.push("frontmatter missing 'fill_sections' array".to_string());
            return errors;
        }
    };

    // 3. Validate fill_sections entries are known section types
    for s in &fill_sections {
        if s.parse::<crate::models::spec_rules::SectionType>().is_err() {
            errors.push(format!("unknown section type in fill_sections: '{}'", s));
        }
    }

    let required_section_count = match missing_policy {
        MissingSectionPolicy::RequireAll => fill_sections.len(),
        MissingSectionPolicy::RequireThrough(current_section) => fill_sections
            .iter()
            .position(|s| s == current_section)
            .map(|idx| idx + 1)
            .unwrap_or_else(|| {
                errors.push(format!(
                    "current section '{}' is not listed in fill_sections",
                    current_section
                ));
                fill_sections.len()
            }),
    };

    // 4b. Reject deprecated section types per AUTHORING.md.
    // These existed in older TD; new specs MUST NOT include them.
    const DEPRECATED_SECTION_TYPES: &[&str] = &[
        "overview",
        "requirements",
        "doc",
        "rust/trait",
        "rust/enum",
        "rust/type-alias",
        "rust/impl",
        "rust/trait-impl",
        "rust/functions",
        "rust/reexports",
    ];
    for s in &fill_sections {
        if DEPRECATED_SECTION_TYPES.contains(&s.as_str()) {
            errors.push(format!(
                "fill_sections contains deprecated section type '{}' \
                 (see AUTHORING.md \"Deprecated types\" — must be migrated)",
                s
            ));
        }
    }

    // 5. Extract actual sections from body
    let sections = extract_sections(body);
    let found_types: Vec<&str> = sections
        .iter()
        .map(|(_, ann, _)| ann.section_type.as_str())
        .collect();

    // 6. Every fill_section must have content
    for (idx, fs) in fill_sections.iter().enumerate() {
        let required_now = idx < required_section_count;
        let matching = sections.iter().find(|(_, ann, _)| ann.section_type == *fs);
        match matching {
            None => {
                if required_now {
                    errors.push(format!(
                        "section type '{}' listed in fill_sections but not found in body",
                        fs
                    ));
                }
            }
            Some((_, _, content)) => {
                let trimmed = content.trim();
                if required_now
                    && (trimmed.is_empty()
                        || trimmed == "<!-- TODO -->"
                        || trimmed.starts_with("TODO"))
                {
                    errors.push(format!("section '{}' has empty/TODO content", fs));
                }
            }
        }
    }

    // 7. Validate annotation lang matches section type's default_lang
    for (heading, ann, _) in &sections {
        if let Ok(st) = ann
            .section_type
            .parse::<crate::models::spec_rules::SectionType>()
        {
            let expected = st.default_lang();
            if ann.lang != expected {
                errors.push(format!(
                    "section '{}' (type: {}) declares lang '{}' but expected '{}'",
                    heading, ann.section_type, ann.lang, expected
                ));
            }
        }
    }

    // 8. Check no duplicate section types
    let mut seen = std::collections::HashSet::new();
    for t in &found_types {
        if !seen.insert(*t) {
            errors.push(format!("duplicate section type: '{}'", t));
        }
    }

    for (heading, ann, content) in &sections {
        if ann.section_type == "e2e-test" {
            errors.extend(validate_e2e_test_section(heading, content));
        }
    }
    if require_all_sections {
        errors.extend(validate_section_implementation_edges(&sections));
    }

    errors
}

fn validate_section_implementation_edges(
    sections: &[(String, SectionAnnotation, String)],
) -> Vec<String> {
    let mut errors = Vec::new();
    let authored_sections: std::collections::BTreeSet<String> = sections
        .iter()
        .filter_map(|(_, ann, _)| normalize_td_section_type_name(&ann.section_type))
        .filter(|section| section != "changes")
        .collect();
    if authored_sections.is_empty() {
        return errors;
    }

    let Some((_, _, changes_content)) = sections.iter().find(|(_, ann, _)| {
        normalize_td_section_type_name(&ann.section_type).as_deref() == Some("changes")
    }) else {
        // `changes` is legacy author-supplied implementation metadata. New TDs
        // may omit it; TD-to-codebase tooling infers existing target edges from
        // spec references in the codebase.
        return errors;
    };
    let Some(yaml_text) = first_yaml_fence(changes_content) else {
        errors.push("changes section must contain a YAML fence with changes: [...]".to_string());
        return errors;
    };
    let value: serde_yaml::Value = match serde_yaml::from_str(&yaml_text) {
        Ok(value) => value,
        Err(err) => {
            errors.push(format!("changes section has invalid YAML: {err}"));
            return errors;
        }
    };
    let Some(seq) = value
        .get("changes")
        .and_then(|v| v.as_sequence())
        .or_else(|| value.get("files").and_then(|v| v.as_sequence()))
        .or_else(|| value.as_sequence())
    else {
        errors.push("changes section YAML must contain `changes: [...]`".to_string());
        return errors;
    };

    let mut implemented_sections = std::collections::BTreeSet::new();
    for (idx, item) in seq.iter().enumerate() {
        let Some(map) = item.as_mapping() else {
            continue;
        };
        let path_hint = map
            .get(serde_yaml::Value::String("path".into()))
            .and_then(|v| v.as_str())
            .or_else(|| {
                map.get(serde_yaml::Value::String("file".into()))
                    .and_then(|v| v.as_str())
            })
            .map(|path| format!("changes[{idx}] ({path})"))
            .unwrap_or_else(|| format!("changes[{idx}]"));

        let impl_mode_valid = match map
            .get(serde_yaml::Value::String("impl_mode".into()))
            .and_then(|v| v.as_str())
        {
            Some("codegen") | Some("hand-written") => true,
            Some(_) | None => false,
        };

        let Some(section) = map
            .get(serde_yaml::Value::String("section".into()))
            .and_then(|v| v.as_str())
        else {
            errors.push(format!(
                "{path_hint} missing section; every change must bind one TD section type"
            ));
            continue;
        };
        let Some(section) = normalize_td_section_type_name(section) else {
            errors.push(format!(
                "{path_hint} section `{section}` is not a known TD section type"
            ));
            continue;
        };
        if impl_mode_valid {
            implemented_sections.insert(section);
        }
    }

    for section in authored_sections {
        if !implemented_sections.contains(&section) {
            errors.push(format!(
                "section type '{section}' has no changes[] entry with matching section and impl_mode"
            ));
        }
    }
    errors
}

fn normalize_td_section_type_name(section: &str) -> Option<String> {
    section
        .parse::<crate::models::spec_rules::SectionType>()
        .ok()
        .map(|section_type| section_type.as_str().to_string())
}

fn validate_e2e_test_section(heading: &str, content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(yaml_text) = first_yaml_fence(content) else {
        errors.push(format!(
            "section '{heading}' (type: e2e-test) requires a YAML fence"
        ));
        return errors;
    };
    let value: serde_yaml::Value = match serde_yaml::from_str(&yaml_text) {
        Ok(value) => value,
        Err(err) => {
            errors.push(format!(
                "section '{heading}' has invalid e2e-test YAML: {err}"
            ));
            return errors;
        }
    };
    let Some(seq) = value
        .get("e2e_tests")
        .and_then(|v| v.as_sequence())
        .or_else(|| value.get("tests").and_then(|v| v.as_sequence()))
    else {
        errors.push(format!(
            "section '{heading}' e2e-test YAML must contain `e2e_tests: [...]`"
        ));
        return errors;
    };
    for (idx, item) in seq.iter().enumerate() {
        let label = format!("e2e_tests[{idx}]");
        let Some(map) = item.as_mapping() else {
            errors.push(format!("{label} must be a mapping"));
            continue;
        };
        if map.get("name").and_then(|v| v.as_str()).is_none() {
            errors.push(format!("{label}.name must be a non-empty string"));
        }
        let has_command = map.get("command").and_then(|v| v.as_str()).is_some()
            || map
                .get("cli")
                .and_then(|v| v.as_mapping())
                .and_then(|cli| cli.get("command"))
                .and_then(|v| v.as_str())
                .is_some();
        if !has_command {
            errors.push(format!(
                "{label} must define `command` or `cli.command` for the v1 CLI e2e runner"
            ));
        }
        if let Some(artifacts) = map
            .get("expect")
            .and_then(|v| v.as_mapping())
            .and_then(|expect| expect.get("artifacts"))
            .and_then(|v| v.as_sequence())
        {
            for (artifact_idx, artifact) in artifacts.iter().enumerate() {
                let path = format!("{label}.expect.artifacts[{artifact_idx}]");
                let Some(artifact_map) = artifact.as_mapping() else {
                    errors.push(format!("{path} must be a mapping"));
                    continue;
                };
                if artifact_map.get("path").and_then(|v| v.as_str()).is_none() {
                    errors.push(format!("{path}.path must be a string"));
                }
                if artifact_map
                    .get("exists")
                    .is_some_and(|v| !matches!(v, serde_yaml::Value::Bool(_)))
                {
                    errors.push(format!("{path}.exists must be a boolean when present"));
                }
                if artifact_map.get("contains").is_some_and(|v| {
                    v.as_sequence()
                        .map(|seq| seq.iter().any(|item| item.as_str().is_none()))
                        .unwrap_or(true)
                }) {
                    errors.push(format!(
                        "{path}.contains must be a string list when present"
                    ));
                }
            }
        }
    }
    errors
}

fn first_yaml_fence(content: &str) -> Option<String> {
    let mut in_yaml = false;
    let mut close_marker = String::new();
    let mut yaml = String::new();
    for line in content.lines() {
        if in_yaml {
            if markdown_fence_closes(line, &close_marker) {
                return Some(yaml);
            }
            yaml.push_str(line);
            yaml.push('\n');
            continue;
        }
        let trimmed = line.trim_start();
        for marker in ["```", "~~~"] {
            if let Some(rest) = trimmed.strip_prefix(marker) {
                let lang = rest.split_whitespace().next().unwrap_or("");
                if lang.eq_ignore_ascii_case("yaml") || lang.eq_ignore_ascii_case("json") {
                    in_yaml = true;
                    close_marker = marker.to_string();
                    yaml.clear();
                    break;
                }
            }
        }
    }
    None
}

/// Check that Mermaid Plus sections have codegen-ready frontmatter.
///
/// For each mermaid-lang section with a known Content type, verifies the
/// Mermaid Plus block's YAML frontmatter can deserialize into the expected
/// struct. Returns a list of errors (empty = all sections are codegen-ready).
fn check_codegen_ready(spec_content: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // Rule 2-2 (hand-written) specs skip codegen entirely — `aw cb gen`
    // emits zero files — so codegen-shape checks (LogicContent etc.) don't
    // apply. Short-circuit before we demand Mermaid Plus frontmatter that
    // would be meaningless for these specs.
    if is_all_hand_written(spec_content) {
        return errors;
    }

    let (_fm_str, body) = match split_frontmatter(spec_content) {
        Some(pair) => pair,
        None => return errors, // already caught by validate_spec
    };

    let blocks = extract_mermaid_plus_blocks(spec_content);
    let sections = extract_sections(body);

    for (heading, ann, _content) in &sections {
        // Only check section types that have a Mermaid Plus content model
        let expected_type = match ann.section_type.as_str() {
            "state-machine" => "StateMachineContent",
            "logic" => "LogicContent",
            "interaction" => "InteractionContent",
            _ => continue,
        };

        // Find the Mermaid Plus block for this section
        let block = blocks
            .iter()
            .find(|b| b.section_type.as_deref() == Some(ann.section_type.as_str()));

        match block {
            None => {
                errors.push(format!(
                    "section '{}' (type: {}) requires a Mermaid Plus block with YAML \
                     frontmatter for codegen (expected {})",
                    heading, ann.section_type, expected_type
                ));
            }
            Some(b) => {
                let deser_result = match ann.section_type.as_str() {
                    "state-machine" => {
                        serde_yaml::from_value::<StateMachineContent>(b.frontmatter.clone())
                            .map(|_| ())
                    }
                    "logic" => {
                        serde_yaml::from_value::<LogicContent>(b.frontmatter.clone()).map(|_| ())
                    }
                    "interaction" => {
                        serde_yaml::from_value::<InteractionContent>(b.frontmatter.clone())
                            .map(|_| ())
                    }
                    _ => Ok(()),
                };
                if let Err(e) = deser_result {
                    errors.push(format!(
                        "section '{}' (type: {}) frontmatter invalid for {}: {}",
                        heading, ann.section_type, expected_type, e
                    ));
                }
            }
        }
    }

    errors
}

/// Derive target spec directory from issue labels.
/// Short hint rendered alongside each section type in the `aw td create`
/// brief. Kept short — full vocabulary lives in `AUTHORING.md`.
fn section_type_brief_hint(st: crate::models::spec_rules::SectionType) -> &'static str {
    use crate::models::spec_rules::SectionType;
    match st {
        SectionType::Overview => "High-level description",
        SectionType::Requirements => "Formal requirements (requirementDiagram)",
        SectionType::Scenarios => "BDD acceptance (Given/When/Then)",
        SectionType::StateMachine => "FSM transitions (stateDiagram-v2)",
        SectionType::Interaction => "Actor sequences (sequenceDiagram)",
        SectionType::Logic => "Business logic (flowchart)",
        SectionType::Dependency => "Type hierarchy (classDiagram)",
        SectionType::DbModel => "Entity-relationship (erDiagram)",
        SectionType::Mindmap => "Hierarchical overview (mindmap)",
        SectionType::Schema => {
            "Data schema (JSON Schema — see x-mamba-* annotations in AUTHORING.md)"
        }
        SectionType::RestApi => "REST API (OpenAPI 3.1)",
        SectionType::RpcApi => "JSON-RPC (OpenRPC 1.3)",
        SectionType::AsyncApi => "WebSocket / pub-sub (AsyncAPI 2.6)",
        SectionType::Cli => "CLI command tree + args",
        SectionType::Config => "Config schema (JSON Schema)",
        SectionType::UnitTest => "Unit-test design (requirementDiagram)",
        SectionType::E2eTest => "E2E product journey + side-effect assertions",
        SectionType::Wireframe => "UI layout (Layout DSL)",
        SectionType::Component => "UI component contract (Custom Elements Manifest)",
        SectionType::DesignToken => "Design tokens (W3C DTCG)",
        SectionType::RuntimeImage => "Container image build contract (Dockerfile/build context)",
        SectionType::Deployment => "Deployment manifests (Kubernetes/Kustomize/runtime ops)",
        SectionType::Doc => "User-facing documentation (markdown)",
        SectionType::Manifest => "Package manifest deps (Cargo.toml / pyproject / package.json)",
        SectionType::RustSourceUnit => "Lossless Rust source unit (CST-backed regen)",
        SectionType::TextSourceUnit => "Lossless shell/text source unit (TD-owned regen)",
        SectionType::Changes => "File change list (path + action)",
    }
}

fn is_deprecated_td_section_type(st: crate::models::spec_rules::SectionType) -> bool {
    use crate::models::spec_rules::SectionType;
    matches!(
        st,
        SectionType::Overview | SectionType::Requirements | SectionType::Doc
    )
}

fn is_active_td_authoring_section_type(st: crate::models::spec_rules::SectionType) -> bool {
    use crate::models::spec_rules::SectionType;
    !is_deprecated_td_section_type(st)
        && !crate::generate::generators::primitive_registry::is_prose_section(st)
        && st != SectionType::Changes
}

fn is_supported_td_payload_section_type(st: crate::models::spec_rules::SectionType) -> bool {
    !is_deprecated_td_section_type(st)
        && !matches!(st, crate::models::spec_rules::SectionType::Scenarios)
}

fn active_td_section_types() -> Vec<crate::models::spec_rules::SectionType> {
    crate::models::spec_rules::SectionType::all_in_fill_order()
        .into_iter()
        .filter(|st| is_active_td_authoring_section_type(*st))
        .collect()
}

fn derive_spec_dir(labels: &[String]) -> String {
    derive_spec_dir_from_parts(labels, None)
}

fn derive_spec_dir_for_issue(issue: &Issue) -> String {
    derive_spec_dir_from_parts(&issue.labels, Some(&issue.title))
}

fn project_label_for_issue(issue: &Issue) -> Option<&str> {
    issue.labels.iter().find_map(|label| {
        let project = label.strip_prefix("project:")?.trim();
        (!project.is_empty()).then_some(project)
    })
}

fn derive_spec_dir_from_parts(labels: &[String], title: Option<&str>) -> String {
    let concern = derive_td_concern(labels, title);
    for label in labels {
        if let Some(crate_name) = label.strip_prefix("crate:") {
            return match crate_name {
                "sdd" => format!("projects/agentic-workflow/{concern}/"),
                "mamba" | "cclab-mamba" => format!("projects/mamba/{concern}/"),
                _ => format!("crates/{}/{concern}/", slugify_path_component(crate_name)),
            };
        }
        if let Some(project) = label.strip_prefix("project:") {
            let project = project.trim();
            if !project.is_empty() {
                return format!("projects/{}/{concern}/", slugify_path_component(project));
            }
        }
    }
    format!("projects/score/{concern}/")
}

fn derive_td_concern(labels: &[String], title: Option<&str>) -> &'static str {
    for label in labels {
        for prefix in ["td:concern:", "concern:", "area:"] {
            if let Some(raw) = label.strip_prefix(prefix) {
                return match raw.trim() {
                    "config" | "configuration" | "settings" => "config",
                    "interface" | "interfaces" | "api" | "cli" | "schema" | "protocol" => {
                        "interfaces"
                    }
                    "test" | "tests" | "validate" | "validation" | "verification" => "validate",
                    "semantic" | "traceability" | "capability" => "semantic",
                    _ => "logic",
                };
            }
        }
    }

    let haystack = title.unwrap_or("").to_ascii_lowercase();
    if contains_any(
        &haystack,
        &["config", "configuration", "settings", "profile"],
    ) {
        return "config";
    }
    if contains_any(
        &haystack,
        &[
            "api",
            "cli",
            "interface",
            "protocol",
            "schema",
            "contract",
            "wire",
            "rpc",
        ],
    ) {
        return "interfaces";
    }
    if contains_any(
        &haystack,
        &[
            "test",
            "tests",
            "validate",
            "validation",
            "verify",
            "verification",
            "conformance",
            "fixture",
        ],
    ) {
        return "validate";
    }
    if contains_any(
        &haystack,
        &["semantic", "traceability", "capability", "readiness"],
    ) {
        return "semantic";
    }
    "logic"
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn slugify_path_component(raw: &str) -> String {
    let slug = slugify_spec_filename(raw);
    if slug.is_empty() {
        "unknown".to_string()
    } else {
        slug
    }
}

fn td_authoring_pass(raw: Option<&str>) -> &str {
    match raw {
        Some("applicability") => "applicability",
        Some("contract") => "contract",
        Some(other) if !other.trim().is_empty() => other,
        _ => "applicability",
    }
}

fn td_section_queue(_pass: &str) -> Vec<String> {
    vec!["logic".to_string(), "unit-test".to_string()]
}

fn td_fill_sections_from_content(content: &str) -> Option<Vec<String>> {
    let (fm_str, _) = split_frontmatter(content)?;
    let fm = serde_yaml::from_str::<serde_yaml::Value>(fm_str).ok()?;
    let sections = fm.get("fill_sections")?.as_sequence()?;
    Some(
        sections
            .iter()
            .filter_map(|value| value.as_str())
            .filter_map(|section| {
                section
                    .parse::<crate::models::spec_rules::SectionType>()
                    .ok()
                    .filter(|st| is_supported_td_payload_section_type(*st))
                    .map(|st| st.as_str().to_string())
            })
            .collect(),
    )
}

fn td_section_queue_for_content(content: &str, pass: &str) -> Vec<String> {
    td_fill_sections_from_content(content)
        .filter(|sections| !sections.is_empty())
        .unwrap_or_else(|| td_section_queue(pass))
}

fn td_section_queue_for_spec(
    worktree_abs: &std::path::Path,
    spec_path: &str,
    pass: &str,
) -> Vec<String> {
    let spec_abs = worktree_abs.join(spec_path);
    std::fs::read_to_string(spec_abs)
        .ok()
        .map(|content| td_section_queue_for_content(&content, pass))
        .unwrap_or_else(|| td_section_queue(pass))
}

fn default_spec_path_for_issue(issue: &Issue, fallback_slug: &str, target_dir: &str) -> String {
    let filename =
        td_spec_filename_for_issue(issue).unwrap_or_else(|| format!("issue-{fallback_slug}"));
    format!(".aw/tech-design/{}{}.md", target_dir, filename)
}

fn default_spec_path_for_issue_in_project(
    project_root: &std::path::Path,
    issue: &Issue,
    fallback_slug: &str,
) -> String {
    let filename =
        td_spec_filename_for_issue(issue).unwrap_or_else(|| format!("issue-{fallback_slug}"));
    if let Some(project) = project_label_for_issue(issue) {
        if let Ok(resolved) =
            crate::services::project_registry::resolve_td_root_from_config(project_root, project)
        {
            let root = std::path::PathBuf::from(resolved.root);
            if let Ok(rel_root) = root.strip_prefix(project_root) {
                return slash_path(
                    rel_root
                        .join(derive_td_concern(&issue.labels, Some(&issue.title)))
                        .join(format!("{filename}.md")),
                );
            }
        }
    }

    let target_dir = derive_spec_dir_for_issue(issue);
    default_spec_path_for_issue(issue, fallback_slug, &target_dir)
}

fn slash_path(path: std::path::PathBuf) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn td_spec_filename_for_issue(issue: &Issue) -> Option<String> {
    let title = strip_issue_title_prefix(&issue.title);
    let slug = slugify_spec_filename(title);
    if slug.chars().any(|c| c.is_ascii_alphabetic()) {
        Some(slug)
    } else {
        None
    }
}

fn strip_issue_title_prefix(title: &str) -> &str {
    let mut s = title.trim();
    for _ in 0..3 {
        let Some(colon_pos) = s.find(':') else {
            break;
        };
        let before = &s[..colon_pos];
        let looks_like_tag = before.len() <= 30
            && before
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '(' | ')' | '.' | '-' | '_'));
        if !looks_like_tag {
            break;
        }
        s = s[colon_pos + 1..].trim_start();
    }
    s
}

fn slugify_spec_filename(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut last_dash = true;
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.len() <= 64 {
        return trimmed;
    }
    let mut cut = 64;
    while cut > 0 && !trimmed.is_char_boundary(cut) {
        cut -= 1;
    }
    trimmed[..cut].trim_end_matches('-').to_string()
}

fn section_payload_rel(slug: &str, pass: &str, section: &str) -> String {
    format!(".aw/payloads/{}/{}/{}.md", slug, pass, section)
}

fn review_payload_rel(slug: &str, pass: &str) -> String {
    format!(".aw/payloads/{}/{}/review.md", slug, pass)
}

fn initialize_td_payload_file(
    project_root: &std::path::Path,
    rel_path: &str,
    content: &str,
) -> Result<bool> {
    let abs = project_root.join(rel_path);
    if abs.exists() {
        return Ok(false);
    }
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create payload directory {}", parent.display()))?;
    }
    std::fs::write(&abs, content)
        .with_context(|| format!("failed to write payload {}", abs.display()))?;
    Ok(true)
}

fn td_section_payload_template(section: &str) -> Result<String> {
    let st = <crate::models::spec_rules::SectionType as std::str::FromStr>::from_str(section)
        .map_err(|e| anyhow::anyhow!(e))?;
    if !is_supported_td_payload_section_type(st) {
        anyhow::bail!("section '{}' is not supported for new TD payloads", section);
    }
    let lang = st.default_lang();
    let body = match lang {
        "markdown" => "(fill)\n".to_string(),
        other => format!("```{}\n(fill)\n```\n", other),
    };
    Ok(format!(
        "## {}\n<!-- type: {} lang: {} -->\n\n{}",
        td_section_title(st.as_str()),
        st.as_str(),
        lang,
        body
    ))
}

fn td_section_title(section: &str) -> String {
    section
        .split('-')
        .map(|part| match part {
            "api" => "API".to_string(),
            "async" => "Async".to_string(),
            "cli" => "CLI".to_string(),
            "db" => "DB".to_string(),
            "e2e" => "E2E".to_string(),
            "rpc" => "RPC".to_string(),
            "rest" => "REST".to_string(),
            "ui" => "UI".to_string(),
            other => {
                let mut chars = other.chars();
                match chars.next() {
                    Some(first) => {
                        format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
                    }
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn td_review_payload_template(round: u8) -> String {
    format!(
        "# Reviews\n\n### Review {}\n**Verdict:** <verdict>\n\n- [<section-type>] (fill)\n",
        round
    )
}

fn remaining_after_section(pass: &str, section: &str) -> Vec<String> {
    remaining_after_section_in_queue(td_section_queue(pass), section)
}

fn remaining_after_section_in_content(content: &str, pass: &str, section: &str) -> Vec<String> {
    remaining_after_section_in_queue(td_section_queue_for_content(content, pass), section)
}

fn remaining_after_section_in_spec(
    worktree_abs: &std::path::Path,
    spec_path: &str,
    pass: &str,
    section: &str,
) -> Vec<String> {
    let spec_abs = worktree_abs.join(spec_path);
    std::fs::read_to_string(spec_abs)
        .ok()
        .map(|content| remaining_after_section_in_content(&content, pass, section))
        .unwrap_or_else(|| remaining_after_section(pass, section))
}

fn remaining_after_section_in_queue(queue: Vec<String>, section: &str) -> Vec<String> {
    let mut seen = false;
    let mut out = Vec::new();
    for item in queue {
        if seen {
            out.push(item);
        } else if item == section {
            seen = true;
        }
    }
    out
}

fn lifecycle_pass_phase(pass: &str) -> String {
    format!("td_{}_in_progress", pass.replace('-', "_"))
}

fn td_review_pass(raw: Option<&str>, issue_phase: &str) -> String {
    match raw {
        Some("applicability") => "applicability".to_string(),
        Some("contract") => "contract".to_string(),
        Some(other) if !other.trim().is_empty() => other.to_string(),
        _ if issue_phase == "td_applicability_created" => "applicability".to_string(),
        _ => "contract".to_string(),
    }
}

/// Resolve the checkout-relative destination path for a `td claim`
/// `--from-path` source. If the source already lives under
/// `<project_root>/.aw/tech-design/`, mirror its exact relative path so
/// the claim does not duplicate an in-tree spec into a label-derived
/// directory. Otherwise fall back to `derive_spec_dir(labels) + file_name`.
fn preserve_or_derive_dest_rel(
    src: &std::path::Path,
    project_root: &std::path::Path,
    labels: &[String],
    slug: &str,
) -> String {
    let td_root_rel = std::path::Path::new(".aw/tech-design");
    let td_root_abs = project_root.join(td_root_rel);
    let src_abs = if src.is_absolute() {
        src.to_path_buf()
    } else {
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(src))
            .unwrap_or_else(|| src.to_path_buf())
    };
    let canon_src = src_abs.canonicalize().unwrap_or(src_abs);
    let canon_td = td_root_abs.canonicalize().unwrap_or(td_root_abs);
    if let Ok(rel) = canon_src.strip_prefix(&canon_td) {
        return format!(".aw/tech-design/{}", rel.display());
    }
    let group = derive_spec_dir(labels);
    let file_name = src
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("{}.md", slug));
    format!(".aw/tech-design/{}{}", group, file_name)
}

// ── Dispatch ─────────────────────────────────────────────────────────

/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub async fn run(args: TdArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    match &args.command {
        TdCommand::Check(_) | TdCommand::Ast(_) | TdCommand::Lock(_) => {}
        TdCommand::Validate(a) => {
            super::workflow_guard::guard_issue_mutation(&project_root, Some(("td", &a.slug)))
                .await?;
        }
        TdCommand::Create(a) => {
            super::workflow_guard::guard_issue_mutation(&project_root, Some(("td", &a.slug)))
                .await?;
        }
        TdCommand::Review(a) => {
            super::workflow_guard::guard_issue_mutation(&project_root, Some(("td", &a.slug)))
                .await?;
        }
        TdCommand::Revise(a) => {
            super::workflow_guard::guard_issue_mutation(&project_root, Some(("td", &a.slug)))
                .await?;
        }
        TdCommand::Merge(a) => {
            super::workflow_guard::guard_issue_mutation(&project_root, Some(("td", &a.slug)))
                .await?;
        }
        TdCommand::Arbitrate(a) => {
            super::workflow_guard::guard_issue_mutation(&project_root, Some(("td", &a.slug)))
                .await?;
        }
        TdCommand::MigrateMermaid(_) | TdCommand::Claim(_) => {
            super::workflow_guard::guard_issue_mutation(&project_root, None).await?;
        }
    }
    match args.command {
        TdCommand::Create(a) => run_create(a).await,
        TdCommand::Validate(a) => run_validate(a).await,
        TdCommand::Review(a) => run_review(a).await,
        TdCommand::Revise(a) => run_revise(a).await,
        TdCommand::Merge(a) => run_merge(a).await,
        TdCommand::Arbitrate(a) => run_arbitrate(a).await,
        TdCommand::Check(a) => run_check(a),
        TdCommand::Ast(a) => run_ast(a),
        TdCommand::MigrateMermaid(a) => super::td_migrate::run(a).await,
        TdCommand::Lock(a) => super::td_lock::run(args.project.as_deref(), a),
        TdCommand::Claim(a) => run_claim(a).await,
    }
}

/// `aw td check <target>` — Phase 1 read-only rule-registry check.
///
/// Resolves `target` as either a slug (current checkout spec dir), a single
/// file path (contains `/` or ends `.md` and points at a file), or a
/// directory. Runs the unified rule registry; exits 0 with no findings,
/// 1 with violations, 2 on invocation error.
///
/// @spec .aw/tech-design/projects/score/specs/score-namespaces.md#changes
pub fn run_check(args: CheckArgs) -> Result<()> {
    // R2 of #1212: section-type-conformance dispatch runs before any
    // target resolution — the verb defaults to scanning the project root
    // and treats `target` as an optional positional path.
    if args.section_type_conformance {
        let path = if args.target.trim().is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(args.target.trim()))
        };
        return super::td_check_section_type::run(super::td_check_section_type::CheckArgs {
            path,
            json: args.json,
        });
    }

    let project_root = crate::find_project_root()?;
    let target = args.target.trim();
    if target.is_empty() {
        anyhow::bail!(
            "aw td check requires a target (slug, spec path, or directory). Pass --section-type-conformance to scan the project root for registry conformance."
        );
    }

    // Disambiguate slug vs path. The same heuristic as `td validate`:
    // contains `/` or ends `.md` → path; otherwise slug.
    let looks_like_path = target.contains('/') || target.ends_with(".md");
    if looks_like_path {
        let shape = crate::validate::classify(target, &project_root);
        return run_validate_readonly(shape, args.json);
    }

    // Slug mode: resolve to the current checkout spec dir.
    run_slug_check(target, None, args.json)
}

/// Run `aw td ast <path>` — parse a TD spec into a `TDAst` and emit JSON.
///
/// @spec .aw/tech-design/projects/agentic-workflow/td_ast/types.md#changes
fn run_ast(args: AstArgs) -> Result<()> {
    let path = std::path::PathBuf::from(&args.path);
    let ast = crate::td_ast::parse_td(&path)
        .map_err(|e| anyhow::anyhow!("td ast parse failed: {}", e.message))?;
    let json = if args.pretty {
        serde_json::to_string_pretty(&ast)
    } else {
        serde_json::to_string(&ast)
    }
    .context("failed to serialise TDAst as JSON")?;
    println!("{}", json);
    Ok(())
}

// ── td create ────────────────────────────────────────────────────────

async fn run_create(args: CreateArgs) -> Result<()> {
    if args.apply {
        run_create_apply(&args).await
    } else {
        run_create_brief(&args).await
    }
}

/// Brief mode: print context for the aw-td-author agent.
///
/// Self-activates TD state on first call: load the issue from the temp issue store,
/// auto-heal any stale `td_*` frontmatter, switch to or create `td-<slug>` only
/// when launched from `main`, set `phase: td_inited` + active branch, and commit
/// a `Td-Init` trailer. After that, fall through to the standard brief output.
async fn run_create_brief(args: &CreateArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let issue_ref = &args.slug;

    let bootstrap_issue = bootstrap_td_issue(&project_root, issue_ref).await?;
    let slug = workflow_slug_for_issue(&bootstrap_issue, issue_ref);
    let branch = td_branch_name(&slug);
    if bootstrap_issue
        .phase
        .as_deref()
        .is_some_and(|phase| phase.starts_with("td_"))
    {
        td_activate_inplace_if_present(&project_root, &slug)?;
    } else {
        provision_td_workspace(&project_root, issue_ref, &slug, &branch).await?;
    }
    let active_branch = crate::branch_switch::current_branch(&project_root)?;

    let backend = LocalBackend::from_project_root(&project_root);
    let issue = backend
        .get(issue_ref)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in workspace", issue_ref))?;

    // Guard: phase must be td_inited
    let phase = issue.phase.as_deref().unwrap_or("");
    if phase != "td_inited" {
        anyhow::bail!(
            "issue '{}' has phase '{}', expected 'td_inited'",
            issue_ref,
            phase
        );
    }

    let pass = td_authoring_pass(args.phase.as_deref());
    let spec_path = args
        .spec_path
        .clone()
        .unwrap_or_else(|| default_spec_path_for_issue_in_project(&project_root, &issue, &slug));
    let queue = td_section_queue(pass);
    let mut first_payload_created = None;
    if let Some(first_section) = queue.first() {
        let expected_payload = section_payload_rel(&slug, pass, first_section);
        first_payload_created = Some(initialize_td_payload_file(
            &project_root,
            &expected_payload,
            &td_section_payload_template(first_section)?,
        )?);
        let already_locked = super::workflow_guard::parse_projection(&issue.body)
            .map(|projection| projection.locked)
            .unwrap_or(false);
        if !already_locked {
            let expected_command = format!(
                "aw td create {} --apply --phase {} --section {} --spec-path {}",
                slug, pass, first_section, spec_path
            );
            super::workflow_guard::create_issue_lock(
                &project_root,
                &super::workflow_guard::TransitionLock::new(&slug, "td", expected_command)
                    .with_expected_payload(expected_payload.clone())
                    .with_active_phase(lifecycle_pass_phase(pass))
                    .with_active_branch(active_branch.clone())
                    .with_current_section(first_section.clone())
                    .with_remaining_sections(queue.iter().skip(1).cloned())
                    .with_dirty_paths([spec_path.clone()]),
            )
            .await?;
            let issue_path_s = issue_path_arg(&backend, &issue);
            let phase_trailer = lifecycle_pass_phase(pass);
            commit_lifecycle_with_extra(
                &project_root,
                &slug,
                &format!("{pass} queue started"),
                "Td-Queue-Start",
                &[issue_path_s.as_str()],
                &[
                    ("Lifecycle-Phase", phase_trailer.as_str()),
                    ("Lifecycle-Pass", pass),
                    ("TD-Section", first_section.as_str()),
                    ("Next-Command", "see WI workflow projection"),
                ],
            )?;
        }
    }

    if !args.human {
        let first_section = queue.first().cloned();
        let payload_path = first_section
            .as_ref()
            .map(|section| section_payload_rel(&slug, pass, section));
        let next = if let (Some(section), Some(payload)) =
            (first_section.as_ref(), payload_path.as_ref())
        {
            next_dispatch(
                format!(
                    "aw td create {} --apply --phase {} --section {} --spec-path {}",
                    slug, pass, section, spec_path
                ),
                "fill the next TD section payload and apply it",
                Some(payload),
            )
        } else {
            next_none("TD create has no remaining section payload")
        };
        let env = serde_json::json!({
            "action": "dispatch",
            "agent": null,
            "slug": slug,
            "next": next,
            "payload_initialized": first_payload_created.unwrap_or(false),
            "target": {
                "spec_path": spec_path,
                "pass": pass,
                "branch": active_branch,
                "issue_file": backend.issue_path(&issue).to_string_lossy(),
            },
            "invoke": {
                "command": "aw td create",
                "args": {
                    "slug": slug,
                    "phase": pass,
                    "section": first_section,
                    "spec_path": spec_path,
                    "payload_path": payload_path,
                },
            },
        });
        print_json_value(&env, args.pretty)?;
        let _ = args.json;
        return Ok(());
    }

    println!("# aw-td-author brief");
    println!();
    println!("Issue:     {} ({})", slug, issue.title);
    println!(
        "Workspace: {} (branch {})",
        project_root.display(),
        active_branch
    );
    println!("Issue file: {}", backend.issue_path(&issue).display());
    println!();
    println!("## Target");
    println!();
    println!("Write the TD skeleton to `{}`.", spec_path);
    println!("Then write exactly one section payload at a time as directed below.");
    println!();
    println!("## Spec format");
    println!();
    println!("The file MUST have YAML frontmatter with `id` and `fill_sections`:");
    println!();
    println!("```yaml");
    println!("---");
    println!("id: <spec-id>");
    println!("summary: <short non-contract summary>");
    println!("fill_sections: [<chosen leaf section types>]");
    println!("---");
    println!("```");
    println!();
    println!("Example: `fill_sections: [logic, unit-test]`.");
    println!();
    println!("Each section uses an H2 heading with type annotation:");
    println!();
    println!("```markdown");
    println!("## Section Title");
    println!("<!-- type: <section-type> lang: <lang> -->");
    println!("```");
    println!();
    println!("## Available structural section types");
    println!();
    println!("| type | lang | use for |");
    println!("|------|------|---------|");
    // Dynamically enumerate from the single SectionType source of truth so new
    // variants (e.g. `manifest`, `tests`) show up here without a second edit.
    // Prose sections are intentionally omitted from new-TD authoring; legacy
    // specs may still parse them, but they are not artifact-driving queue items.
    for st in active_td_section_types() {
        println!(
            "| {} | {} | {} |",
            st.as_str(),
            st.default_lang(),
            section_type_brief_hint(st),
        );
    }
    println!();
    println!("## Suggested order for chosen sections");
    println!();
    println!(
        "{}",
        active_td_section_types()
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(" → "),
    );
    println!();
    println!("Only sections listed in `fill_sections` are required. Before the skeleton exists, the workflow seeds `logic` then `unit-test` as the minimal default.");
    println!();
    println!(
        "Use frontmatter `summary:` for overview text; requirements stay in the WI body. Do not add legacy prose sections such as `scenarios` unless migrating an older TD."
    );
    println!();
    println!("## Mermaid Plus (CODEGEN-READY — required for state-machine, logic, interaction)");
    println!();
    println!(
        "Sections with type `state-machine`, `logic`, or `interaction` MUST use Mermaid Plus:"
    );
    println!("YAML frontmatter between `---` markers INSIDE the mermaid code fence.");
    println!();
    println!(
        "state-machine: `id`, `initial`, `nodes` (map: id → {{kind, label?}}), `edges` (array: {{from, to, event?}})"
    );
    println!("  Node kinds: initial, normal, terminal, transient, choice");
    println!(
        "logic: `id`, `entry`, `nodes` (map: id → {{kind, label?}}), `edges` (array: {{from, to, label?}})"
    );
    println!("  Node kinds: start, process, decision, terminal");
    println!(
        "interaction: `id`, `actors` (array: {{id, kind?}}), `messages` (array: {{from, to, name, returns?}})"
    );
    println!("  Actor kinds: actor, participant, system");
    println!();
    println!("Read `.aw/tech-design/AUTHORING.md` § Mermaid Plus Content Model for full examples.");
    println!();
    println!("## Rules");
    println!();
    println!("- Section type MUST match content (state-machine → stateDiagram-v2, not flowchart)");
    println!("- Code fence lang MUST match declared lang");
    println!("- Natural language < 10% of file content");
    println!("- No real code (Python, TypeScript, Rust, SQL)");
    println!("- One code block per section");
    println!();
    println!("## When done");
    println!();
    println!("Run:");
    println!(
        "  aw td create {} --apply --phase {} --section <section-type> --spec-path {}",
        slug, pass, spec_path
    );
    println!();
    if let Some(first_section) = queue.first() {
        println!();
        println!("Next section payload:");
        println!("  {}", section_payload_rel(&slug, pass, first_section));
        println!(
            "Payload: {}",
            if first_payload_created.unwrap_or(false) {
                "initialized"
            } else {
                "existing"
            }
        );
    }
    println!();
    println!("## Issue context");
    println!();
    // Print the issue body (requirements, scope, etc.) for context
    if !issue.body.is_empty() {
        println!("{}", issue.body);
    }

    Ok(())
}

/// Apply mode: validate spec in-place, emit dispatch envelope for validate.
///
/// When `--section X` is supplied, reads
/// `.aw/payloads/<slug>/<section>.md` and merges ONLY that section
/// into the spec file before validating (loop-fill path). When omitted,
/// the caller is expected to have written the full spec directly — we
/// just validate in-place.
async fn run_create_apply(args: &CreateArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = &args.slug;

    let spec_path = args
        .spec_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--spec-path is required with --apply"))?;

    td_activate_inplace_allowing_dirty_spec_path(&project_root, slug, spec_path)?;
    let worktree_abs = td_workspace_path(&project_root, slug);
    if !worktree_abs.exists() {
        anyhow::bail!("workspace not found: {}", worktree_abs.display());
    }

    let spec_abs = worktree_abs.join(spec_path);

    // Per-section merge path: read payload, merge into base spec, write back.
    if let Some(section) = args.section.as_deref() {
        let pass = td_authoring_pass(args.phase.as_deref());
        let preferred_payload_rel = section_payload_rel(slug, pass, section);
        let legacy_payload_rel = format!(".aw/payloads/{}/{}.md", slug, section);
        let payload_rel = if worktree_abs.join(&preferred_payload_rel).exists() {
            preferred_payload_rel
        } else {
            legacy_payload_rel
        };
        let payload_abs = worktree_abs.join(&payload_rel);
        if !payload_abs.exists() {
            let msg = format!(
                "section payload not found: {} (write the per-section spec fragment there first)",
                payload_abs.display()
            );
            return td_error(slug, msg);
        }
        let payload_body =
            std::fs::read_to_string(&payload_abs).context("failed to read section payload")?;
        let base_body = if spec_abs.exists() {
            std::fs::read_to_string(&spec_abs).context("failed to read base spec")?
        } else {
            // First-section call on a brand-new spec: the skeleton (the
            // YAML frontmatter with `id` and `fill_sections`) is written
            // by mainthread from the brief printed by `aw td create
            // <slug>` (no flags). If this file is missing we bail loud
            // rather than invent a header.
            let msg = format!(
                "spec file not found: {} (write the frontmatter skeleton first; see `aw td create {}`)",
                spec_abs.display(),
                slug
            );
            return td_error(slug, msg);
        };

        let merged = match merge_spec_section(&base_body, section, &payload_body) {
            Ok(m) => m,
            Err(e) => {
                return td_error(slug, format!("section merge failed: {}", e));
            }
        };

        let report = validate_new_td_authoring_content(
            &spec_abs,
            &merged,
            TdContentValidationScope::RequireThrough(section),
        )?;
        if report.has_errors() {
            let errors = td_content_error_messages(&report);
            print_td_content_errors("TD content validation errors", &report);
            let msg = format!(
                "td content validation failed ({} errors): {}",
                errors.len(),
                errors.join("; ")
            );
            return td_error(slug, msg);
        }

        std::fs::write(&spec_abs, merged).context("failed to write merged spec")?;

        if args.phase.is_some() {
            complete_section_apply(&project_root, slug, spec_path, args).await?;
            let _ = std::fs::remove_file(&payload_abs);
            return Ok(());
        }

        // Best-effort cleanup after successful validation/write. Keep payloads
        // intact on validation errors so retries do not lose their input.
        let _ = std::fs::remove_file(&payload_abs);
    }

    if !spec_abs.exists() {
        let msg = format!("spec file not found: {}", spec_abs.display());
        return td_error(slug, msg);
    }

    let validation_scope = args
        .section
        .as_deref()
        .map(TdContentValidationScope::RequireThrough)
        .unwrap_or(TdContentValidationScope::Complete);
    let report = validate_new_td_authoring_file(&spec_abs, validation_scope)?;
    if report.has_errors() {
        let errors = td_content_error_messages(&report);
        print_td_content_errors("TD content validation errors", &report);
        let msg = format!(
            "td content validation failed ({} errors): {}",
            errors.len(),
            errors.join("; ")
        );
        return td_error(slug, msg);
    }

    super::workflow_guard::create_issue_lock(
        &worktree_abs,
        &super::workflow_guard::TransitionLock::new(
            slug,
            "td",
            format!("aw td validate {} --spec-path {}", slug, spec_path),
        )
        .with_phase_from("td_inited")
        .with_dirty_paths([spec_path.to_string()]),
    )
    .await?;

    // Emit dispatch → aw td validate
    print_envelope(&TdEnvelope::Dispatch {
        agent: None,
        slug,
        invoke: Invoke {
            command: "aw td validate",
            args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
        },
    })?;

    Ok(())
}

async fn complete_section_apply(
    project_root: &std::path::Path,
    slug: &str,
    spec_path: &str,
    args: &CreateArgs,
) -> Result<()> {
    let worktree_abs = td_workspace_path(project_root, slug);
    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;
    let issue_path = backend.issue_path(&issue);
    let issue_path_s = issue_path.to_string_lossy().into_owned();
    let pass = td_authoring_pass(args.phase.as_deref());
    let section = args
        .section
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("section apply requires --section"))?;
    let remaining = remaining_after_section_in_spec(&worktree_abs, spec_path, pass, section);
    let active_branch = crate::branch_switch::current_branch(&worktree_abs).unwrap_or_default();

    let next = if let Some(next_section) = remaining.first() {
        let active_phase = lifecycle_pass_phase(pass);
        backend
            .update(
                slug,
                &IssuePatch {
                    phase: Some(active_phase.clone()),
                    ..Default::default()
                },
            )
            .await?;
        let expected_payload = section_payload_rel(slug, pass, next_section);
        let expected_command = format!(
            "aw td create {} --apply --phase {} --section {} --spec-path {}",
            slug, pass, next_section, spec_path
        );
        super::workflow_guard::create_issue_lock(
            &worktree_abs,
            &super::workflow_guard::TransitionLock::new(slug, "td", expected_command)
                .with_expected_payload(expected_payload)
                .with_active_phase(active_phase.clone())
                .with_active_branch(active_branch)
                .with_current_section(next_section.clone())
                .with_remaining_sections(remaining.iter().skip(1).cloned())
                .with_dirty_paths([spec_path.to_string()]),
        )
        .await?;
        Some((
            "Td-Section",
            active_phase,
            "aw td create",
            serde_json::json!({
                "slug": slug,
                "apply": true,
                "phase": pass,
                "section": next_section,
                "spec_path": spec_path,
            }),
        ))
    } else if pass == "applicability" {
        let active_phase = "td_applicability_created".to_string();
        backend
            .update(
                slug,
                &IssuePatch {
                    phase: Some(active_phase.clone()),
                    ..Default::default()
                },
            )
            .await?;
        super::workflow_guard::complete_issue_lock(&worktree_abs, slug, "td").await?;
        maybe_push_remote(&worktree_abs, &issue_path, slug).await?;
        Some((
            "Td-Applicability-Complete",
            active_phase,
            "aw td review",
            serde_json::json!({
                "slug": slug,
                "phase": "applicability",
                "spec_path": spec_path,
            }),
        ))
    } else {
        super::workflow_guard::complete_issue_lock(&worktree_abs, slug, "td").await?;
        backend
            .update(
                slug,
                &IssuePatch {
                    phase: Some("td_created".to_string()),
                    ..Default::default()
                },
            )
            .await?;
        maybe_push_remote(&worktree_abs, &issue_path, slug).await?;
        Some((
            "Td-Contract-Complete",
            "td_created".to_string(),
            "aw td review",
            serde_json::json!({
                "slug": slug,
                "phase": "contract",
                "spec_path": spec_path,
            }),
        ))
    };

    let Some((stage, next_phase, next_command, next_args)) = next else {
        return Ok(());
    };
    commit_lifecycle_with_extra(
        &worktree_abs,
        slug,
        &format!("{pass} section: {section}"),
        stage,
        &[spec_path, issue_path_s.as_str()],
        &[
            ("Lifecycle-Phase", next_phase.as_str()),
            ("Lifecycle-Pass", pass),
            ("TD-Section", section),
            ("Previous-Phase", issue.phase.as_deref().unwrap_or("")),
            ("Next-Phase", next_phase.as_str()),
            ("Next-Command", next_command),
        ],
    )?;

    print_envelope(&TdEnvelope::Dispatch {
        agent: None,
        slug,
        invoke: Invoke {
            command: next_command,
            args: next_args,
        },
    })?;
    Ok(())
}

// ── td validate ──────────────────────────────────────────────────────

async fn run_validate(args: ValidateArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;

    // Phase 1 compat shim: path-mode validate routes to `td check` and
    // emits a deprecation line on stderr. Slug-mode is unchanged.
    // @spec .aw/tech-design/projects/score/specs/score-namespaces.md#changes
    match crate::validate::classify(&args.slug, &project_root) {
        crate::validate::PathShape::Slug(_) => {}
        shape @ (crate::validate::PathShape::Prefix(_) | crate::validate::PathShape::File(_)) => {
            eprintln!("deprecated: use 'aw td check' instead");
            return run_validate_readonly(shape, args.json);
        }
    }

    let slug = &args.slug;

    // Phase 1 compat shim: `--check` flag routes to `td check` and
    // emits a deprecation line on stderr.
    // @spec .aw/tech-design/projects/score/specs/score-namespaces.md#changes
    if args.check {
        eprintln!("deprecated: use 'aw td check' instead");
        return run_slug_check(slug, args.spec_path.as_deref(), args.json);
    }

    if let Some(spec_path) = args.spec_path.as_deref() {
        td_activate_inplace_allowing_dirty_spec_path(&project_root, slug, spec_path)?;
    } else {
        td_activate_inplace_if_present(&project_root, slug)?;
    }
    let worktree_abs = td_workspace_path(&project_root, slug);
    if !worktree_abs.exists() {
        anyhow::bail!("workspace not found: {}", worktree_abs.display());
    }

    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");

    let result = match phase {
        "td_inited" => handle_create_milestone(&args, &backend, &worktree_abs, &issue).await,
        "td_created" | "td_revised" => {
            handle_review_milestone(&args, &backend, &worktree_abs, &issue).await
        }
        "td_reviewed" => handle_revise_milestone(&args, &backend, &worktree_abs, &issue).await,
        "td_merged" => handle_post_merge_lifecycle(&backend, &worktree_abs, &issue, slug).await,
        other => {
            let msg = format!("unexpected phase '{}' for td validate", other);
            print_envelope(&TdEnvelope::Error {
                slug,
                message: &msg,
            })?;
            Ok(())
        }
    };
    if result.is_ok() {
        super::workflow_guard::complete_issue_lock(&worktree_abs, slug, "td").await?;
    }
    result
}

/// Post-merge lifecycle handler — invoked when `aw td validate` sees
/// an issue already at phase `td_merged`. Implements R8 (backfill
/// ship_commit from git log) and R4 placeholder (loop-close
/// verification — deferred to follow-up issue F1 which extracts
/// gen_code_capture).
/// @spec aw-td-validate-lifecycle-extension.md#logic
async fn handle_post_merge_lifecycle(
    backend: &LocalBackend,
    worktree_abs: &std::path::Path,
    issue: &crate::issues::Issue,
    slug: &str,
) -> Result<()> {
    use crate::issues::ShipStatus;

    // R8: backfill ship_commit from git log if missing.
    // @spec aw-td-validate-lifecycle-extension.md#logic
    if issue.ship_commit.is_none() {
        if let Some(commit) = find_ship_commit_from_log(worktree_abs, slug)? {
            let patch = IssuePatch {
                ship_commit: Some(commit.clone()),
                // Also normalize ship_status if missing — old issues at td_merged
                // are by definition step1_shipped.
                ship_status: if issue.ship_status.is_none() {
                    Some(ShipStatus::Step1Shipped)
                } else {
                    None
                },
                ..Default::default()
            };
            backend.update(slug, &patch).await?;
            print_envelope(&TdEnvelope::Done {
                slug,
                message: &format!(
                    "backfilled ship_commit={} (from Lifecycle-Stage: Td-Merge git log)",
                    &commit[..8.min(commit.len())]
                ),
            })?;
            return Ok(());
        }
    }

    // R4 (placeholder — loop-close verification): re-running gen-code in-memory
    // and diffing requires extracting gen_code_capture (deferred to follow-up
    // issue F1). For now, surface the current ship_status so users can see
    // where the lifecycle is.
    let status = issue
        .ship_status
        .map(|s| match s {
            ShipStatus::NotStarted => "not_started",
            ShipStatus::Step1Shipped => "step1_shipped",
            ShipStatus::LoopClosed => "loop_closed",
            ShipStatus::Rejected => "rejected",
        })
        .unwrap_or("unset");
    let commit_short = issue
        .ship_commit
        .as_deref()
        .map(|c| &c[..8.min(c.len())])
        .unwrap_or("none");
    print_envelope(&TdEnvelope::Done {
        slug,
        message: &format!(
            "tech-design merged; ship_status={}, ship_commit={}; \
             loop-close verification (R4) deferred to follow-up F1",
            status, commit_short
        ),
    })?;
    Ok(())
}

/// Walk `git log` in the worktree for the most recent commit whose
/// message contains `Lifecycle-Stage: Td-Merge` for this slug, and
/// return its hash. Used by R8 backfill.
/// @spec aw-td-validate-lifecycle-extension.md#logic
fn find_ship_commit_from_log(worktree_abs: &std::path::Path, slug: &str) -> Result<Option<String>> {
    let git_bin = crate::git::find_git_bin().ok_or_else(|| anyhow::anyhow!("git not found"))?;
    let needle = format!("Lifecycle-Stage: Td-Merge");
    let slug_needle = format!("Lifecycle-Slug: {}", slug);
    let output = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree_abs)
        .args(["log", "--format=%H%x00%B%x1e", "--all", "--grep", &needle])
        .output()
        .context("git log failed")?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for entry in stdout.split('\x1e') {
        let entry = entry.trim_start_matches('\n');
        if entry.is_empty() {
            continue;
        }
        let mut parts = entry.splitn(2, '\x00');
        let hash = parts.next().unwrap_or("").trim();
        let body = parts.next().unwrap_or("");
        if body.contains(&slug_needle) {
            return Ok(Some(hash.to_string()));
        }
    }
    Ok(None)
}

/// `aw td validate <slug> --check`: resolve the slug's worktree, then
/// behave like path-mode read-only validate. No commit, no phase advance,
/// no envelope — purely a rule check. When `spec_path` is given, validate
/// just that file; otherwise walk the worktree's tech-design tree.
fn run_slug_check(slug: &str, spec_path: Option<&str>, json: bool) -> Result<()> {
    let project_root = crate::find_project_root()?;
    td_activate_inplace_if_present(&project_root, slug)?;
    let worktree_abs = td_workspace_path(&project_root, slug);
    if !worktree_abs.exists() {
        anyhow::bail!("workspace not found: {}", worktree_abs.display());
    }
    let target = match spec_path {
        Some(p) => worktree_abs.join(p),
        None => crate::shared::workspace::tech_design_path(&worktree_abs),
    };
    if !target.exists() {
        anyhow::bail!("validate target not found: {}", target.display());
    }
    let shape = crate::validate::classify(
        target
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("non-utf8 path"))?,
        &project_root,
    );
    run_validate_readonly(shape, json)
}

/// Path-mode `aw td validate <prefix|file>` and `aw td check`: resolve
/// spec files, run the shared TD content gate, print findings, exit non-zero
/// on any error-severity finding. Read-only — no commit, no phase advance, no
/// envelope.
fn run_validate_readonly(shape: crate::validate::PathShape, json: bool) -> Result<()> {
    let files = crate::validate::resolve_spec_files(&shape)?;
    if files.is_empty() {
        eprintln!("no spec files found under target");
        return Ok(());
    }

    let report = crate::validate::run_rules(&files);

    if json {
        println!("{}", serde_json::to_string_pretty(&report.findings)?);
    } else {
        eprintln!("── validate ──────────────────────────────────────────");
        eprintln!("scanned {} spec file(s)", files.len());
        if report.is_empty() {
            eprintln!("  0 findings");
        } else {
            eprintln!("  {} finding(s):", report.findings.len());
            for f in &report.findings {
                eprintln!("  {}", f.format());
            }
        }
    }

    if report.has_errors() {
        std::process::exit(1);
    }
    Ok(())
}

/// td_inited → td_created: verify spec, commit, dispatch reviewer.
///
/// Retry-cap semantics (shared with `aw wi validate`): on failure,
/// bump `fill_retry_count` on the issue frontmatter and encode the count
/// in the emitted error envelope:
/// - `retry=1` — mainthread re-dispatches td-author with error feedback.
/// - `retry=2 takeover` — mainthread runs `aw td create --apply` itself.
/// - `retry=N arbitrate` (N >= 3) — terminal; surfaces to user.
/// On success, reset `fill_retry_count` to 0.
async fn handle_create_milestone(
    args: &ValidateArgs,
    backend: &LocalBackend,
    worktree_path: &std::path::Path,
    issue: &crate::issues::Issue,
) -> Result<()> {
    let slug = &args.slug;
    let spec_path = args
        .spec_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--spec-path is required for td validate"))?;

    let spec_abs = worktree_path.join(spec_path);
    if !spec_abs.exists() {
        let msg = format!("spec file not found: {}", spec_abs.display());
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    let report = validate_new_td_authoring_file(&spec_abs, TdContentValidationScope::Complete)?;
    if report.has_errors() {
        let errors = td_content_error_messages(&report);
        let prev = issue.fill_retry_count.unwrap_or(0);
        let new_count = prev + 1;
        backend
            .update(
                slug,
                &IssuePatch {
                    fill_retry_count: Some(new_count),
                    ..Default::default()
                },
            )
            .await?;
        rollback_worktree_file(worktree_path, spec_path)?;

        let qualifier = if new_count < 2 {
            format!("retry={}", new_count)
        } else if new_count == 2 {
            "retry=2 takeover".to_string()
        } else {
            format!("retry={} arbitrate", new_count)
        };
        let msg = format!(
            "spec validation failed [{}]: {}",
            qualifier,
            errors.join("; ")
        );
        print_td_content_errors("TD content validation errors", &report);
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    // Advance phase + reset retry counter
    let patch = IssuePatch {
        phase: Some("td_created".to_string()),
        validation_errors: Some(vec![]),
        fill_retry_count: Some(0),
        ..Default::default()
    };
    backend.update(slug, &patch).await?;

    // Commit
    let issue_path_s = issue_path_arg(backend, issue);
    let issue_path = std::path::PathBuf::from(&issue_path_s);
    maybe_push_remote(worktree_path, &issue_path, slug).await?;
    commit_lifecycle(
        worktree_path,
        slug,
        "spec authored",
        "Td-Create",
        &[spec_path, issue_path_s.as_str()],
    )?;

    // Dispatch reviewer
    print_envelope(&TdEnvelope::Dispatch {
        agent: None,
        slug,
        invoke: Invoke {
            command: "aw td review",
            args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
        },
    })?;

    Ok(())
}

/// td_created/td_revised → td_reviewed: read review, dispatch by verdict.
async fn handle_review_milestone(
    args: &ValidateArgs,
    backend: &LocalBackend,
    worktree_path: &std::path::Path,
    issue: &crate::issues::Issue,
) -> Result<()> {
    let slug = &args.slug;
    let spec_path = args
        .spec_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--spec-path required"))?;

    let issue_path_s = issue_path_arg(backend, issue);
    let issue_path = std::path::PathBuf::from(&issue_path_s);

    // Guard: must have uncommitted changes (reviewer wrote something)
    if !worktree_has_uncommitted_change(worktree_path, spec_path)? {
        print_envelope(&TdEnvelope::Error {
            slug,
            message: "no review detected — spec unchanged since last commit; \
                      reviewer may have crashed. Re-dispatch aw-td-reviewer.",
        })?;
        return Ok(());
    }

    // Read spec and look for # Reviews section with verdict
    let content = std::fs::read_to_string(worktree_path.join(spec_path))?;
    let verdict = parse_td_review_verdict(&content);

    let prior_count = issue.review_count.unwrap_or(0);
    let new_count = prior_count.saturating_add(1);

    let patch = IssuePatch {
        phase: Some("td_reviewed".to_string()),
        review_count: Some(new_count),
        validation_errors: Some(vec![]),
        ..Default::default()
    };
    backend.update(slug, &patch).await?;

    let detail = match &verdict {
        ReviewVerdict::Approved => format!("approved (review #{})", new_count),
        ReviewVerdict::NeedsRevision => format!("needs-revision (review #{})", new_count),
    };
    maybe_push_remote(worktree_path, &issue_path, slug).await?;
    commit_lifecycle(
        worktree_path,
        slug,
        &detail,
        "Td-Review",
        &[spec_path, issue_path_s.as_str()],
    )?;

    match verdict {
        ReviewVerdict::Approved => {
            // Safety net: verify spec is codegen-ready before merging
            let cg_errors = check_codegen_ready(&content);
            if !cg_errors.is_empty() {
                eprintln!("Codegen-ready check failed (safety net at review approval):");
                for e in &cg_errors {
                    eprintln!("  - {}", e);
                }
                let msg = format!(
                    "approved but not codegen-ready ({} errors): {}. \
                     Re-dispatch author to fix Mermaid Plus frontmatter.",
                    cg_errors.len(),
                    cg_errors.join("; ")
                );
                print_envelope(&TdEnvelope::Error {
                    slug,
                    message: &msg,
                })?;
                return Ok(());
            }

            print_envelope(&TdEnvelope::Dispatch {
                agent: None,
                slug,
                invoke: Invoke {
                    command: "aw cb gen",
                    args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
                },
            })?;
        }
        ReviewVerdict::NeedsRevision if new_count == 1 => {
            print_envelope(&TdEnvelope::Dispatch {
                agent: None,
                slug,
                invoke: Invoke {
                    command: "aw td revise",
                    args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
                },
            })?;
        }
        ReviewVerdict::NeedsRevision if new_count == 2 => {
            print_envelope(&TdEnvelope::Dispatch {
                agent: None,
                slug,
                invoke: Invoke {
                    command: "aw td arbitrate",
                    args: serde_json::json!({ "slug": slug }),
                },
            })?;
        }
        ReviewVerdict::NeedsRevision => {
            print_envelope(&TdEnvelope::Error {
                slug,
                message: &format!(
                    "invariant violation: review_count={} exceeds ceiling",
                    new_count
                ),
            })?;
        }
    }

    Ok(())
}

/// td_reviewed → td_revised: verify revision (uncommitted OR mainthread takeover
/// commit), validate, advance phase, commit (when subagent path), dispatch
/// reviewer for round 2.
///
/// @spec .aw/tech-design/projects/score/specs/aw-td-revise-payload-merge-and-takeover.md#logic-revise-milestone-takeover-guard
async fn handle_revise_milestone(
    args: &ValidateArgs,
    backend: &LocalBackend,
    worktree_path: &std::path::Path,
    _issue: &crate::issues::Issue,
) -> Result<()> {
    let slug = &args.slug;
    let spec_path = args
        .spec_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--spec-path required"))?;

    let issue_path_s = issue_path_arg(backend, _issue);
    let issue_path = std::path::PathBuf::from(&issue_path_s);

    // Guard: revision must exist somewhere — either uncommitted (subagent
    // wrote the spec, validate is the sole commit point) OR already committed
    // by mainthread takeover (subagent failed; mainthread did the work and
    // committed with the proper Lifecycle-Stage trailer per CLAUDE.md
    // [retry=2 takeover] pattern). Accepting the takeover commit makes
    // validate idempotent w.r.t. recovery — re-running it after a successful
    // takeover dispatches the next phase instead of erroring.
    let has_uncommitted = worktree_has_uncommitted_change(worktree_path, spec_path)?;
    let already_committed_takeover =
        !has_uncommitted && head_is_takeover_revise(worktree_path, spec_path)?;
    if !has_uncommitted && !already_committed_takeover {
        print_envelope(&TdEnvelope::Error {
            slug,
            message: "no revision detected — spec unchanged since last commit and HEAD \
                      is not a takeover revise commit. Re-dispatch aw-td-reviser, or \
                      mainthread-takeover by editing the spec and committing with trailer \
                      'Lifecycle-Stage: Td-Revise'.",
        })?;
        return Ok(());
    }

    let spec_abs = worktree_path.join(spec_path);
    let report = validate_td_content_file(&spec_abs, TdContentValidationScope::Complete)?;
    if report.has_errors() {
        let errors = td_content_error_messages(&report);
        if has_uncommitted {
            rollback_worktree_file(worktree_path, spec_path)?;
        }
        print_td_content_errors("Revised TD content validation errors", &report);
        let msg = format!("revision validation failed: {}", errors.join("; "));
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    let patch = IssuePatch {
        phase: Some("td_revised".to_string()),
        flagged_sections: Some(vec![]),
        validation_errors: Some(vec![]),
        ..Default::default()
    };
    backend.update(slug, &patch).await?;

    if already_committed_takeover {
        // Mainthread already committed the revise; advance phase + dispatch
        // reviewer without making another commit. Idempotent.
    } else {
        maybe_push_remote(worktree_path, &issue_path, slug).await?;
        commit_lifecycle(
            worktree_path,
            slug,
            "revised",
            "Td-Revise",
            &[spec_path, issue_path_s.as_str()],
        )?;
    }

    // Dispatch reviewer (round 2)
    print_envelope(&TdEnvelope::Dispatch {
        agent: None,
        slug,
        invoke: Invoke {
            command: "aw td review",
            args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
        },
    })?;

    Ok(())
}

// ── Review verdict parsing ───────────────────────────────────────────

enum ReviewVerdict {
    Approved,
    NeedsRevision,
}

/// Parse review verdict from spec content. Looks for `# Reviews` section
/// with `**Verdict:** approved` or `**Verdict:** needs-revision`.
fn parse_td_review_verdict(content: &str) -> ReviewVerdict {
    let reviews_start = content.find("# Reviews");
    if let Some(start) = reviews_start {
        let reviews_section = &content[start..];
        if reviews_section.contains("**Verdict:** approved")
            || reviews_section.contains("**Verdict**: approved")
            || reviews_section.contains("Verdict: approved")
        {
            return ReviewVerdict::Approved;
        }
    }
    ReviewVerdict::NeedsRevision
}

// ── Git helpers ──────────────────────────────────────────────────────

fn rollback_worktree_file(worktree_path: &std::path::Path, rel_path: &str) -> Result<()> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let _ = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree_path)
        .args(["checkout", "--", rel_path])
        .output();
    Ok(())
}

fn worktree_has_uncommitted_change(
    worktree_path: &std::path::Path,
    rel_path: &str,
) -> Result<bool> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree_path)
        .args(["diff", "--name-only", "HEAD", "--", rel_path])
        .output()
        .context("git diff failed")?;
    Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

/// Detect whether HEAD is a mainthread-takeover revise commit. Returns true
/// when (i) HEAD's commit message body contains `Lifecycle-Stage: Td-Revise`
/// AND (ii) `rel_path` was modified between HEAD~1 and HEAD. This lets
/// `handle_revise_milestone` accept a takeover commit as a valid revision
/// without forcing the strict "validate is sole commit point" invariant
/// to break the documented [retry=2 takeover] recovery path.
///
/// /// @spec .aw/tech-design/projects/score/specs/aw-td-revise-payload-merge-and-takeover.md#logic-revise-milestone-takeover-guard
fn head_is_takeover_revise(worktree_path: &std::path::Path, rel_path: &str) -> Result<bool> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;

    // (i) HEAD message contains the revise trailer.
    let log = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree_path)
        .args(["log", "-1", "--format=%B"])
        .output()
        .context("git log -1 failed")?;
    if !log.status.success() {
        return Ok(false);
    }
    let body = String::from_utf8_lossy(&log.stdout);
    if !body.contains("Lifecycle-Stage: Td-Revise") {
        return Ok(false);
    }

    // (ii) rel_path was modified in HEAD vs HEAD~1.
    let diff = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree_path)
        .args(["diff", "--name-only", "HEAD~1", "HEAD", "--", rel_path])
        .output()
        .context("git diff HEAD~1 HEAD failed")?;
    if !diff.status.success() {
        return Ok(false);
    }
    Ok(!String::from_utf8_lossy(&diff.stdout).trim().is_empty())
}

/// Extract flagged section types from review findings.
/// Looks for patterns like `- [overview]`, `- [logic]`, etc.
fn extract_flagged_sections(reviews_text: &str) -> Vec<String> {
    let mut flagged = Vec::new();
    for line in reviews_text.lines() {
        let trimmed = line.trim();
        let rest = if let Some(r) = trimmed.strip_prefix("- [") {
            r
        } else if let Some(r) = trimmed.strip_prefix("* [") {
            r
        } else {
            continue;
        };
        if let Some(close) = rest.find(']') {
            let section = rest[..close].trim().to_lowercase();
            if !section.is_empty() && !flagged.contains(&section) {
                flagged.push(section);
            }
        }
    }
    flagged
}

// ── td review ───────────────────────────────────────────────────────

async fn run_review(args: ReviewArgs) -> Result<()> {
    if args.apply {
        run_review_apply(&args).await
    } else {
        run_review_brief(&args).await
    }
}

/// Brief mode: print spec content + review guidelines for the reviewer agent.
async fn run_review_brief(args: &ReviewArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = &args.slug;
    let spec_path = args
        .spec_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--spec-path is required"))?;

    td_activate_inplace_allowing_dirty_spec_path(&project_root, slug, spec_path)?;
    let worktree_abs = td_workspace_path(&project_root, slug);
    if !worktree_abs.exists() {
        anyhow::bail!("workspace not found: {}", worktree_abs.display());
    }

    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");
    let pass = td_review_pass(args.phase.as_deref(), phase);
    let expected_phases: &[&str] = if pass == "applicability" {
        &["td_applicability_created"]
    } else {
        &["td_created", "td_revised"]
    };
    if !expected_phases.contains(&phase) {
        anyhow::bail!(
            "issue '{}' has phase '{}', expected {} for {} review",
            slug,
            phase,
            expected_phases.join(" or "),
            pass
        );
    }

    let spec_abs = worktree_abs.join(spec_path);
    if !spec_abs.exists() {
        anyhow::bail!("spec file not found: {}", spec_abs.display());
    }

    let spec_content = std::fs::read_to_string(&spec_abs)?;
    let round = issue.review_count.unwrap_or(0) + 1;
    let active_branch = crate::branch_switch::current_branch(&worktree_abs).unwrap_or_default();
    let already_locked = super::workflow_guard::parse_projection(&issue.body)
        .map(|projection| projection.locked)
        .unwrap_or(false);
    let review_payload = review_payload_rel(slug, &pass);
    let review_payload_created = initialize_td_payload_file(
        &worktree_abs,
        &review_payload,
        &td_review_payload_template(round),
    )?;
    if !already_locked {
        let expected_command = format!(
            "aw td review {} --apply --phase {} --spec-path {}",
            slug, pass, spec_path
        );
        super::workflow_guard::create_issue_lock(
            &worktree_abs,
            &super::workflow_guard::TransitionLock::new(slug, "td", expected_command)
                .with_expected_payload(review_payload.clone())
                .with_active_phase(phase)
                .with_active_branch(active_branch)
                .with_current_section(format!("{pass}-review"))
                .with_dirty_paths([spec_path.to_string()]),
        )
        .await?;
        let issue_path_s = issue_path_arg(&backend, &issue);
        commit_lifecycle_with_extra(
            &worktree_abs,
            slug,
            &format!("{pass} review queued"),
            "Td-Review-Queue",
            &[issue_path_s.as_str()],
            &[
                ("Lifecycle-Phase", phase),
                ("Lifecycle-Pass", &pass),
                ("TD-Section", "review"),
                ("Next-Command", "see WI workflow projection"),
            ],
        )?;
    }

    if !args.human {
        let command = format!(
            "aw td review {} --apply --phase {} --spec-path {}",
            slug, pass, spec_path
        );
        let env = serde_json::json!({
            "action": "dispatch",
            "agent": null,
            "slug": slug,
            "next": next_dispatch(
                command.clone(),
                "complete the TD review payload and apply it",
                Some(&review_payload),
            ),
            "payload_initialized": review_payload_created,
            "target": {
                "spec_path": spec_path,
                "pass": pass,
                "round": round,
                "issue_file": backend.issue_path(&issue).to_string_lossy(),
            },
            "invoke": {
                "command": "aw td review",
                "args": {
                    "slug": slug,
                    "phase": pass,
                    "spec_path": spec_path,
                    "round": round,
                    "payload_path": review_payload,
                },
            },
        });
        print_json_value(&env, args.pretty)?;
        let _ = args.json;
        return Ok(());
    }

    println!("# aw-td-reviewer brief");
    println!();
    println!("Issue:      {} ({})", slug, issue.title);
    println!("Checkout:   {}", worktree_abs.display());
    println!("Spec file:  {}", spec_path);
    println!("Pass:       {}", pass);
    println!("Round:      {}", round);
    println!();
    println!("## Task");
    println!();
    println!("Review the tech-design spec below. Evaluate each section for:");
    println!("- Correctness: does the section accurately describe the design?");
    println!("- Completeness: are all necessary details included?");
    println!("- Consistency: do sections reference each other correctly?");
    println!("- Clarity: is the spec machine-readable and unambiguous?");
    println!();
    if round > 1 {
        println!(
            "This is round {}. Focus on whether the reviser addressed prior findings.",
            round
        );
        println!("Read the previous review in the `# Reviews` section below.");
        println!();
    }
    println!("## Output");
    println!();
    println!("Write the review payload to `{}`.", review_payload);
    println!(
        "Payload: {}.",
        if review_payload_created {
            "initialized"
        } else {
            "existing"
        }
    );
    println!("The CLI will append it to the spec when the expected command runs.");
    println!();
    println!("```markdown");
    println!("# Reviews");
    println!();
    println!("### Review {}", round);
    println!("**Verdict:** approved|needs-revision");
    println!();
    println!("- [<section-type>] <finding \u{2014} concrete suggestion>");
    println!("- [<section-type>] <finding \u{2014} concrete suggestion>");
    println!("```");
    println!();
    println!("Where `<section-type>` matches the spec's fill_sections (e.g., logic,");
    println!("cli, unit-test, e2e-test, changes).");
    println!("For `needs-revision`, include at least one `[section-type]` finding.");
    println!();
    println!("### Verdict calibration");
    println!();
    println!("- `approved` if spec is clear, correct, and complete enough to implement from.");
    println!("- `needs-revision` if any section has errors, omissions, or inconsistencies");
    println!("  that would cause ambiguity during implementation.");
    println!("- Stylistic nits alone do not warrant `needs-revision`.");
    println!();
    println!("## When done");
    println!();
    println!(
        "Run: aw td review {} --apply --phase {} --spec-path {}",
        slug, pass, spec_path
    );
    println!();
    println!("## Spec content");
    println!();
    println!("{}", spec_content);

    if !issue.body.is_empty() {
        println!();
        println!("## Issue context");
        println!();
        println!("{}", issue.body);
    }

    Ok(())
}

/// Apply mode: validate review format, emit dispatch → validate.
async fn run_review_apply(args: &ReviewArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = &args.slug;
    let spec_path = args
        .spec_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--spec-path is required with --apply"))?;

    td_activate_inplace_allowing_dirty_spec_path(&project_root, slug, spec_path)?;
    let worktree_abs = td_workspace_path(&project_root, slug);
    if !worktree_abs.exists() {
        anyhow::bail!("workspace not found: {}", worktree_abs.display());
    }

    let spec_abs = worktree_abs.join(spec_path);
    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;
    let issue_phase = issue.phase.as_deref().unwrap_or("");
    let pass = td_review_pass(args.phase.as_deref(), issue_phase);
    let mut content = std::fs::read_to_string(&spec_abs).context("failed to read spec file")?;

    if args.phase.is_some() {
        let payload_rel = review_payload_rel(slug, &pass);
        let payload_abs = worktree_abs.join(&payload_rel);
        if !payload_abs.exists() {
            let msg = format!(
                "review payload not found: {} (write the review there first)",
                payload_abs.display()
            );
            print_envelope(&TdEnvelope::Error {
                slug,
                message: &msg,
            })?;
            return Ok(());
        }
        let payload = std::fs::read_to_string(&payload_abs)
            .with_context(|| format!("failed to read review payload {}", payload_abs.display()))?;
        if !content.ends_with('\n') {
            content.push('\n');
        }
        if !content.contains("# Reviews") && !payload.contains("# Reviews") {
            content.push_str("\n# Reviews\n\n");
        } else {
            content.push('\n');
        }
        content.push_str(payload.trim());
        content.push('\n');
        std::fs::write(&spec_abs, &content).context("failed to append review payload")?;
        let _ = std::fs::remove_file(&payload_abs);
    }

    // Validate review section exists
    if content.find("# Reviews").is_none() {
        let msg =
            "spec has no '# Reviews' section \u{2014} append your review before running --apply";
        print_envelope(&TdEnvelope::Error { slug, message: msg })?;
        return Ok(());
    }

    // Validate verdict line exists
    let has_verdict = content.contains("**Verdict:** approved")
        || content.contains("**Verdict:** needs-revision")
        || content.contains("**Verdict**: approved")
        || content.contains("**Verdict**: needs-revision");

    if !has_verdict {
        let msg = "review missing verdict line \u{2014} add '**Verdict:** approved' or '**Verdict:** needs-revision'";
        print_envelope(&TdEnvelope::Error { slug, message: msg })?;
        return Ok(());
    }

    // For needs-revision, require at least one [section-type] finding
    let verdict = parse_td_review_verdict(&content);
    if matches!(verdict, ReviewVerdict::NeedsRevision) {
        let reviews_start = content.find("# Reviews").unwrap_or(0);
        let flagged = extract_flagged_sections(&content[reviews_start..]);
        if flagged.is_empty() {
            let msg = "needs-revision verdict requires at least one [section-type] finding";
            print_envelope(&TdEnvelope::Error { slug, message: msg })?;
            return Ok(());
        }
    }

    if args.phase.is_some() {
        complete_phase_review_apply(&worktree_abs, slug, spec_path, &issue, &pass, &content)
            .await?;
        return Ok(());
    }

    super::workflow_guard::create_issue_lock(
        &worktree_abs,
        &super::workflow_guard::TransitionLock::new(
            slug,
            "td",
            format!("aw td validate {} --spec-path {}", slug, spec_path),
        )
        .with_phase_from("td_created")
        .with_dirty_paths([spec_path.to_string()]),
    )
    .await?;

    // Emit dispatch → validate
    print_envelope(&TdEnvelope::Dispatch {
        agent: None,
        slug,
        invoke: Invoke {
            command: "aw td validate",
            args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
        },
    })?;

    Ok(())
}

async fn complete_phase_review_apply(
    worktree_abs: &std::path::Path,
    slug: &str,
    spec_path: &str,
    issue: &crate::issues::Issue,
    pass: &str,
    content: &str,
) -> Result<()> {
    let backend = LocalBackend::from_project_root(worktree_abs);
    let verdict = parse_td_review_verdict(content);
    let issue_path = backend.issue_path(issue);
    let issue_path_s = issue_path.to_string_lossy().into_owned();
    let active_branch = crate::branch_switch::current_branch(worktree_abs).unwrap_or_default();

    match (pass, verdict) {
        ("applicability", ReviewVerdict::Approved) => {
            let contract = "contract";
            let contract_queue = td_section_queue_for_spec(worktree_abs, spec_path, contract);
            let next_section = contract_queue
                .first()
                .ok_or_else(|| anyhow::anyhow!("contract section queue is empty"))?;
            let active_phase = lifecycle_pass_phase(contract);
            backend
                .update(
                    slug,
                    &IssuePatch {
                        phase: Some(active_phase.clone()),
                        ..Default::default()
                    },
                )
                .await?;
            let expected_payload = section_payload_rel(slug, contract, next_section);
            let expected_command = format!(
                "aw td create {} --apply --phase {} --section {} --spec-path {}",
                slug, contract, next_section, spec_path
            );
            super::workflow_guard::create_issue_lock(
                worktree_abs,
                &super::workflow_guard::TransitionLock::new(slug, "td", expected_command)
                    .with_expected_payload(expected_payload)
                    .with_active_phase(active_phase.clone())
                    .with_active_branch(active_branch)
                    .with_current_section(next_section.clone())
                    .with_remaining_sections(contract_queue.iter().skip(1).cloned())
                    .with_dirty_paths([spec_path.to_string()]),
            )
            .await?;
            maybe_push_remote(worktree_abs, &issue_path, slug).await?;
            commit_lifecycle_with_extra(
                worktree_abs,
                slug,
                "applicability review approved",
                "Td-Applicability-Review",
                &[spec_path, issue_path_s.as_str()],
                &[
                    ("Lifecycle-Phase", active_phase.as_str()),
                    ("Lifecycle-Pass", "applicability"),
                    ("TD-Section", "review"),
                    ("Previous-Phase", issue.phase.as_deref().unwrap_or("")),
                    ("Next-Phase", active_phase.as_str()),
                    ("Next-Command", "aw td create"),
                ],
            )?;
            print_envelope(&TdEnvelope::Dispatch {
                agent: None,
                slug,
                invoke: Invoke {
                    command: "aw td create",
                    args: serde_json::json!({
                        "slug": slug,
                        "apply": true,
                        "phase": contract,
                        "section": next_section,
                        "spec_path": spec_path,
                    }),
                },
            })?;
        }
        ("applicability", ReviewVerdict::NeedsRevision) => {
            let next_section = first_flagged_or_first_section(content, "applicability")
                .ok_or_else(|| anyhow::anyhow!("applicability section queue is empty"))?;
            create_section_revision_lock(
                worktree_abs,
                slug,
                spec_path,
                "applicability",
                &next_section,
                &active_branch,
            )
            .await?;
            let active_phase = lifecycle_pass_phase("applicability");
            backend
                .update(
                    slug,
                    &IssuePatch {
                        phase: Some(active_phase.clone()),
                        ..Default::default()
                    },
                )
                .await?;
            maybe_push_remote(worktree_abs, &issue_path, slug).await?;
            commit_lifecycle_with_extra(
                worktree_abs,
                slug,
                "applicability review needs revision",
                "Td-Applicability-Review",
                &[spec_path, issue_path_s.as_str()],
                &[
                    ("Lifecycle-Phase", active_phase.as_str()),
                    ("Lifecycle-Pass", "applicability"),
                    ("TD-Section", "review"),
                    ("Previous-Phase", issue.phase.as_deref().unwrap_or("")),
                    ("Next-Phase", active_phase.as_str()),
                    ("Next-Command", "aw td create"),
                ],
            )?;
            print_envelope(&TdEnvelope::Dispatch {
                agent: None,
                slug,
                invoke: Invoke {
                    command: "aw td create",
                    args: serde_json::json!({
                        "slug": slug,
                        "apply": true,
                        "phase": "applicability",
                        "section": next_section,
                        "spec_path": spec_path,
                    }),
                },
            })?;
        }
        (_, ReviewVerdict::Approved) => {
            let cg_errors = check_codegen_ready(content);
            if !cg_errors.is_empty() {
                let msg = format!(
                    "approved but not codegen-ready ({} errors): {}",
                    cg_errors.len(),
                    cg_errors.join("; ")
                );
                print_envelope(&TdEnvelope::Error {
                    slug,
                    message: &msg,
                })?;
                return Ok(());
            }
            let prior_count = issue.review_count.unwrap_or(0);
            let new_count = prior_count.saturating_add(1);
            backend
                .update(
                    slug,
                    &IssuePatch {
                        phase: Some("td_reviewed".to_string()),
                        review_count: Some(new_count),
                        validation_errors: Some(vec![]),
                        ..Default::default()
                    },
                )
                .await?;
            super::workflow_guard::complete_issue_lock(worktree_abs, slug, "td").await?;
            maybe_push_remote(worktree_abs, &issue_path, slug).await?;
            commit_lifecycle_with_extra(
                worktree_abs,
                slug,
                &format!("contract review approved (review #{new_count})"),
                "Td-Review",
                &[spec_path, issue_path_s.as_str()],
                &[
                    ("Lifecycle-Phase", "td_reviewed"),
                    ("Lifecycle-Pass", "contract"),
                    ("TD-Section", "review"),
                    ("Previous-Phase", issue.phase.as_deref().unwrap_or("")),
                    ("Next-Phase", "td_reviewed"),
                    ("Next-Command", "aw cb gen"),
                ],
            )?;
            print_envelope(&TdEnvelope::Dispatch {
                agent: None,
                slug,
                invoke: Invoke {
                    command: "aw cb gen",
                    args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
                },
            })?;
        }
        (_, ReviewVerdict::NeedsRevision) => {
            let prior_count = issue.review_count.unwrap_or(0);
            let new_count = prior_count.saturating_add(1);
            if new_count >= 2 {
                backend
                    .update(
                        slug,
                        &IssuePatch {
                            phase: Some("td_reviewed".to_string()),
                            review_count: Some(new_count),
                            ..Default::default()
                        },
                    )
                    .await?;
                super::workflow_guard::complete_issue_lock(worktree_abs, slug, "td").await?;
                maybe_push_remote(worktree_abs, &issue_path, slug).await?;
                commit_lifecycle_with_extra(
                    worktree_abs,
                    slug,
                    &format!("contract review needs arbitration (review #{new_count})"),
                    "Td-Review",
                    &[spec_path, issue_path_s.as_str()],
                    &[
                        ("Lifecycle-Phase", "td_reviewed"),
                        ("Lifecycle-Pass", "contract"),
                        ("TD-Section", "review"),
                        ("Previous-Phase", issue.phase.as_deref().unwrap_or("")),
                        ("Next-Phase", "td_reviewed"),
                        ("Next-Command", "aw td arbitrate"),
                    ],
                )?;
                print_envelope(&TdEnvelope::Dispatch {
                    agent: None,
                    slug,
                    invoke: Invoke {
                        command: "aw td arbitrate",
                        args: serde_json::json!({ "slug": slug }),
                    },
                })?;
                return Ok(());
            }

            let next_section = first_flagged_or_first_section(content, "contract")
                .ok_or_else(|| anyhow::anyhow!("contract section queue is empty"))?;
            let active_phase = lifecycle_pass_phase("contract");
            backend
                .update(
                    slug,
                    &IssuePatch {
                        phase: Some(active_phase.clone()),
                        review_count: Some(new_count),
                        ..Default::default()
                    },
                )
                .await?;
            create_section_revision_lock(
                worktree_abs,
                slug,
                spec_path,
                "contract",
                &next_section,
                &active_branch,
            )
            .await?;
            maybe_push_remote(worktree_abs, &issue_path, slug).await?;
            commit_lifecycle_with_extra(
                worktree_abs,
                slug,
                &format!("contract review needs revision (review #{new_count})"),
                "Td-Review",
                &[spec_path, issue_path_s.as_str()],
                &[
                    ("Lifecycle-Phase", active_phase.as_str()),
                    ("Lifecycle-Pass", "contract"),
                    ("TD-Section", "review"),
                    ("Previous-Phase", issue.phase.as_deref().unwrap_or("")),
                    ("Next-Phase", active_phase.as_str()),
                    ("Next-Command", "aw td create"),
                ],
            )?;
            print_envelope(&TdEnvelope::Dispatch {
                agent: None,
                slug,
                invoke: Invoke {
                    command: "aw td create",
                    args: serde_json::json!({
                        "slug": slug,
                        "apply": true,
                        "phase": "contract",
                        "section": next_section,
                        "spec_path": spec_path,
                    }),
                },
            })?;
        }
    }
    Ok(())
}

fn first_flagged_or_first_section(content: &str, pass: &str) -> Option<String> {
    let queue = td_section_queue_for_content(content, pass);
    let reviews_start = content.find("# Reviews").unwrap_or(0);
    let flagged = extract_flagged_sections(&content[reviews_start..]);
    flagged
        .into_iter()
        .find(|section| queue.iter().any(|candidate| candidate == section))
        .or_else(|| queue.first().cloned())
}

async fn create_section_revision_lock(
    worktree_abs: &std::path::Path,
    slug: &str,
    spec_path: &str,
    pass: &str,
    section: &str,
    active_branch: &str,
) -> Result<()> {
    let active_phase = lifecycle_pass_phase(pass);
    let expected_payload = section_payload_rel(slug, pass, section);
    let expected_command = format!(
        "aw td create {} --apply --phase {} --section {} --spec-path {}",
        slug, pass, section, spec_path
    );
    let remaining = remaining_after_section_in_spec(worktree_abs, spec_path, pass, section);
    super::workflow_guard::create_issue_lock(
        worktree_abs,
        &super::workflow_guard::TransitionLock::new(slug, "td", expected_command)
            .with_expected_payload(expected_payload)
            .with_active_phase(active_phase)
            .with_active_branch(active_branch.to_string())
            .with_current_section(section.to_string())
            .with_remaining_sections(remaining)
            .with_dirty_paths([spec_path.to_string()]),
    )
    .await
}

// ── td revise ───────────────────────────────────────────────────────

async fn run_revise(args: ReviseArgs) -> Result<()> {
    if args.apply {
        run_revise_apply(&args).await
    } else {
        run_revise_brief(&args).await
    }
}

/// Brief mode: print spec + review findings + flagged sections for the reviser.
async fn run_revise_brief(args: &ReviseArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = &args.slug;
    let spec_path = args
        .spec_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--spec-path is required"))?;

    td_activate_inplace_if_present(&project_root, slug)?;
    let worktree_abs = td_workspace_path(&project_root, slug);
    if !worktree_abs.exists() {
        anyhow::bail!("workspace not found: {}", worktree_abs.display());
    }

    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");
    if phase != "td_reviewed" {
        anyhow::bail!(
            "issue '{}' has phase '{}', expected 'td_reviewed'",
            slug,
            phase
        );
    }

    let spec_abs = worktree_abs.join(spec_path);
    if !spec_abs.exists() {
        anyhow::bail!("spec file not found: {}", spec_abs.display());
    }

    let spec_content = std::fs::read_to_string(&spec_abs)?;

    // Extract flagged sections from the spec's # Reviews
    let flagged = if let Some(start) = spec_content.find("# Reviews") {
        extract_flagged_sections(&spec_content[start..])
    } else {
        vec![]
    };

    let next_section = first_flagged_or_first_section(&spec_content, "contract");
    let mut payload_path = None;
    let mut payload_initialized = false;
    if let Some(section) = next_section.as_deref() {
        let rel = format!(".aw/payloads/{}/{}.md", slug, section);
        payload_initialized = initialize_td_payload_file(
            &worktree_abs,
            &rel,
            &td_section_payload_template(section)?,
        )?;
        payload_path = Some(rel);
    }

    if !args.human {
        let next = if let (Some(section), Some(payload)) =
            (next_section.as_ref(), payload_path.as_ref())
        {
            next_dispatch(
                format!(
                    "aw td revise {} --apply --section {} --spec-path {}",
                    slug, section, spec_path
                ),
                "fill the next flagged TD section payload and apply it",
                Some(payload),
            )
        } else {
            next_none("TD revise found no flagged or fallback section")
        };
        let env = serde_json::json!({
            "action": "dispatch",
            "agent": null,
            "slug": slug,
            "next": next,
            "payload_initialized": payload_initialized,
            "target": {
                "spec_path": spec_path,
                "flagged_sections": flagged,
                "issue_file": backend.issue_path(&issue).to_string_lossy(),
            },
            "invoke": {
                "command": "aw td revise",
                "args": {
                    "slug": slug,
                    "section": next_section,
                    "spec_path": spec_path,
                    "payload_path": payload_path,
                },
            },
        });
        print_json_value(&env, args.pretty)?;
        let _ = args.json;
        return Ok(());
    }

    println!("# aw-td-reviser brief");
    println!();
    println!("Issue:      {} ({})", slug, issue.title);
    println!("Checkout:   {}", worktree_abs.display());
    println!("Spec file:  {}", spec_path);
    if !flagged.is_empty() {
        println!("Flagged:    {}", flagged.join(", "));
    }
    println!();
    println!("## Task");
    println!();
    println!("Read the latest review findings in the `# Reviews` section of the spec.");
    println!("Fix ONLY the flagged sections listed above. Do not modify unflagged sections.");
    println!();
    println!("For each flagged section:");
    println!("- Read the reviewer's specific finding for that section type");
    println!("- Rewrite the section content to address the finding");
    println!("- Keep the annotation (`<!-- type: X lang: Y -->`) unchanged");
    println!("- Ensure the revised content still passes validation rules");
    println!();
    println!("## When done");
    println!();
    println!(
        "Run: aw td revise {} --apply --spec-path {}",
        slug, spec_path
    );
    println!();
    println!("## Spec content");
    println!();
    println!("{}", spec_content);

    Ok(())
}

/// Apply mode: validate revised spec, emit dispatch → validate.
/// `aw td revise --apply [--section X]` — merge per-section payload (when
/// `--section` set) into the spec, validate the post-merge content, emit
/// dispatch envelope for `aw td validate`. Mirrors `run_create_apply`.
///
/// @spec .aw/tech-design/projects/score/specs/aw-td-revise-payload-merge-and-takeover.md#logic-revise-apply-section-merge
async fn run_revise_apply(args: &ReviseArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = &args.slug;
    let spec_path = args
        .spec_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--spec-path is required with --apply"))?;

    td_activate_inplace_allowing_dirty_spec_path(&project_root, slug, spec_path)?;
    let worktree_abs = td_workspace_path(&project_root, slug);
    if !worktree_abs.exists() {
        anyhow::bail!("workspace not found: {}", worktree_abs.display());
    }

    let spec_abs = worktree_abs.join(spec_path);

    // Per-section merge path: read payload, merge into base spec, write back
    // BEFORE validation. Mirrors `run_create_apply` so the validator sees the
    // post-merge result instead of the spec at HEAD (which may still carry
    // content the reviser is trying to remove — e.g. a deprecated section).
    if let Some(section) = args.section.as_deref() {
        let payload_rel = format!(".aw/payloads/{}/{}.md", slug, section);
        let payload_abs = worktree_abs.join(&payload_rel);
        if !payload_abs.exists() {
            let msg = format!(
                "section payload not found: {} (write the per-section revision fragment there first)",
                payload_abs.display()
            );
            print_envelope(&TdEnvelope::Error {
                slug,
                message: &msg,
            })?;
            return Ok(());
        }
        let payload_body =
            std::fs::read_to_string(&payload_abs).context("failed to read section payload")?;
        let base_body =
            std::fs::read_to_string(&spec_abs).context("failed to read base spec for revise")?;

        let merged = match merge_spec_section(&base_body, section, &payload_body) {
            Ok(m) => m,
            Err(e) => {
                print_envelope(&TdEnvelope::Error {
                    slug,
                    message: &format!("revise section merge failed: {}", e),
                })?;
                return Ok(());
            }
        };
        std::fs::write(&spec_abs, merged).context("failed to write merged spec")?;

        // Best-effort cleanup: remove the per-section payload after merge
        // so repeated runs don't accidentally re-merge stale content.
        let _ = std::fs::remove_file(&payload_abs);
    }

    let validation_scope = args
        .section
        .as_deref()
        .map(TdContentValidationScope::RequireThrough)
        .unwrap_or(TdContentValidationScope::Complete);
    let report = validate_td_content_file(&spec_abs, validation_scope)?;
    if report.has_errors() {
        let errors = td_content_error_messages(&report);
        print_td_content_errors("Revised TD content validation errors", &report);
        let msg = format!(
            "revision validation failed ({} errors): {}",
            errors.len(),
            errors.join("; ")
        );
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    super::workflow_guard::create_issue_lock(
        &worktree_abs,
        &super::workflow_guard::TransitionLock::new(
            slug,
            "td",
            format!("aw td validate {} --spec-path {}", slug, spec_path),
        )
        .with_phase_from("td_reviewed")
        .with_dirty_paths([spec_path.to_string()]),
    )
    .await?;

    // Emit dispatch → validate
    print_envelope(&TdEnvelope::Dispatch {
        agent: None,
        slug,
        invoke: Invoke {
            command: "aw td validate",
            args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
        },
    })?;

    Ok(())
}

// ── cb gen ─────────────────────────────────────────────────────────

/// Implementation of `aw cb gen` — generates code from an approved TD spec.
/// Writes canonical phase `cb_genned` and trailer `Cb-Gen`.
///
/// @spec .aw/tech-design/projects/score/specs/score-namespaces.md#changes
pub(crate) async fn run_gen_code(args: GenCodeArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let requested_slug = &args.slug;

    let bootstrapped_issue = bootstrap_td_issue(&project_root, requested_slug).await?;
    let slug = workflow_slug_for_issue(&bootstrapped_issue, requested_slug);
    let slug = slug.as_str();

    td_activate_inplace_if_present(&project_root, slug)?;
    let worktree_abs = td_workspace_path(&project_root, slug);
    if !worktree_abs.exists() {
        let msg = format!("workspace not found: {}", worktree_abs.display());
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");
    if phase != "td_reviewed" {
        let msg = format!(
            "cannot gen-code: phase is '{}', expected 'td_reviewed'",
            phase
        );
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    let spec_path_owned: String = match args.spec_path.as_deref() {
        Some(p) => p.to_string(),
        None => match discover_worktree_spec(&worktree_abs) {
            Some(p) => p,
            None => {
                anyhow::bail!(
                    "--spec-path is required for cb gen (auto-discovery found no \
                     unique spec under .aw/tech-design/ in the current checkout)"
                );
            }
        },
    };
    let spec_path = spec_path_owned.as_str();
    let spec_abs = worktree_abs.join(spec_path);
    if !spec_abs.exists() {
        let msg = format!("spec file not found: {}", spec_abs.display());
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    // Run codegen
    let report = run_apply_worktree(&spec_abs, &worktree_abs)
        .map_err(|e| anyhow::anyhow!("codegen failed: {}", e))?;

    // Print summary to stderr
    eprintln!(
        "gen-code: {} files ({} created), {} blocks",
        report.files.len(),
        report.files_created(),
        report.total_blocks_updated()
    );
    for f in &report.files {
        eprintln!(
            "  {} {}",
            if f.created { "+" } else { "~" },
            f.path.display()
        );
    }

    // Update phase. Phase 1 migration: write canonical `cb_genned`
    // (legacy `td_gen_coded` is still accepted by readers).
    // @spec .aw/tech-design/projects/score/specs/score-namespaces.md#changes
    let patch = IssuePatch {
        phase: Some("cb_genned".to_string()),
        remove_labels: vec![
            super::workflow_guard::LOCK_LABEL.to_string(),
            super::workflow_guard::TD_LOCK_LABEL.to_string(),
            super::workflow_guard::CB_LOCK_LABEL.to_string(),
        ],
        ..Default::default()
    };
    backend.update(slug, &patch).await?;

    // Commit generated files + issue. Rule 2-2 (hand-written) entries are
    // skipped by the generator — they have no `created`/`updated`/blocks-set
    // flag and the target file may not exist yet, so `git add` would fail.
    // Only commit paths that the generator actually wrote.
    let issue_path_s = issue_path_arg(&backend, &issue);
    let issue_path = std::path::PathBuf::from(&issue_path_s);
    let mut commit_paths: Vec<String> = report
        .files
        .iter()
        .filter(|f| f.created || f.updated || f.blocks_updated > 0)
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();
    commit_paths.push(issue_path_s.clone());
    let refs: Vec<&str> = commit_paths.iter().map(|s| s.as_str()).collect();
    // Phase 1 migration: canonical trailer is `Cb-Gen`
    // (readers still parse legacy `Td-GenCode`).
    // @spec .aw/tech-design/projects/score/specs/score-namespaces.md#changes
    maybe_push_remote(&worktree_abs, &issue_path, slug).await?;
    commit_lifecycle(&worktree_abs, slug, "code generated", "Cb-Gen", &refs)?;

    // Phase 3 (R8): post-codegen dispatch decision.
    // Count emitted HANDWRITE markers in the worktree source tree. If any
    // remain, dispatch to `aw cb fill`. Otherwise (0-marker fast-path,
    // R11) retain the historical `aw td merge` dispatch.
    // @spec .aw/tech-design/projects/score/specs/score-cb-fill-workflow.md#logic
    let marker_count = super::cb_fill::count_worktree_handwrite_markers(&worktree_abs);
    if marker_count > 0 {
        print_envelope(&TdEnvelope::Dispatch {
            agent: None,
            slug,
            invoke: Invoke {
                command: "aw cb fill",
                args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
            },
        })?;
    } else {
        print_envelope(&TdEnvelope::Dispatch {
            agent: None,
            slug,
            invoke: Invoke {
                command: "aw td merge",
                args: serde_json::json!({ "slug": slug, "spec_path": spec_path }),
            },
        })?;
    }

    Ok(())
}

// ── td merge ────────────────────────────────────────────────────────

async fn run_merge(args: MergeArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = &args.slug;
    let starting_branch = crate::branch_switch::current_branch(&project_root)?;
    let dedicated_branch_mode = should_use_td_branch(&starting_branch);

    td_activate_inplace_if_present(&project_root, slug)?;
    let branch = td_branch_name(slug);
    let branch_present =
        crate::branch_switch::branch_exists_local(&project_root, &branch).unwrap_or(false);
    if dedicated_branch_mode && !branch_present {
        let msg = format!("workspace not found: branch '{}' does not exist", branch);
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    let backend = LocalBackend::from_project_root(&project_root);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");
    // Accept cb_genned (canonical Phase 1+), cb_filled (Phase 3 post-fill),
    // cb_reviewed (Phase 4 post-review; verdict applied by `aw cb review --apply`),
    // td_gen_coded (legacy reader alias for one release), td_reviewed
    // (no-codegen path), or td_merged (retry).
    // @spec .aw/tech-design/projects/score/specs/score-cb-fill-workflow.md#logic (R10)
    // @spec .aw/tech-design/projects/score/specs/aw-td-merge-accepts-cb-reviewed.md#schema (R1, R3)
    if !crate::issues::types::td_phase::is_mergeable(phase)
        && phase != crate::issues::types::td_phase::TD_REVIEWED
        && phase != crate::issues::types::td_phase::TD_MERGED
    {
        let msg = format!(
            "cannot merge: phase is '{}', expected a mergeable TD/CB phase",
            phase
        );
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    // Bug 2 fix: refuse to merge a spec whose every `action: create | modify`
    // entry points at a non-existent file. That signature means gen-code
    // skipped emission (e.g. a hand-written batch with no scaffold) and the
    // implementation step never happened. Allow `--allow-empty-impl` for the
    // legitimate "spec-only" case.
    if !args.allow_empty_impl && phase != "td_merged" {
        let mut missing_total: Vec<(std::path::PathBuf, Vec<String>)> = Vec::new();
        let mut entries_total = 0usize;
        let td_root = crate::shared::workspace::tech_design_path(&project_root);
        if td_root.exists() {
            for entry in walkdir::WalkDir::new(&td_root)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                if entry.path().extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let Ok(content) = std::fs::read_to_string(entry.path()) else {
                    continue;
                };
                let total = crate::generate::apply::extract_change_entries_count(&content);
                entries_total += total;
                let missing =
                    crate::generate::apply::missing_implementation_paths(&content, &project_root);
                if !missing.is_empty() {
                    missing_total.push((entry.path().to_path_buf(), missing));
                }
            }
        }
        if entries_total > 0 && !missing_total.is_empty() {
            let total_missing: usize = missing_total.iter().map(|(_, m)| m.len()).sum();
            // Block when the entire implementation is missing — that's the
            // gen-code-skipped signature. Warn-only when partially missing.
            let block = total_missing == entries_total;
            let mut preview: Vec<String> = Vec::new();
            for (spec, m) in missing_total.iter().take(3) {
                let spec_rel = spec
                    .strip_prefix(&project_root)
                    .unwrap_or(spec)
                    .display()
                    .to_string();
                for p in m.iter().take(3) {
                    preview.push(format!("    {} \u{2192} missing {}", spec_rel, p));
                }
            }
            if block {
                let msg = format!(
                    "refusing to merge: spec lists {} file(s) but {} are missing on disk \
                     (codegen likely skipped; run `aw cb gen {}` then implement, \
                     or pass --allow-empty-impl for spec-only merges).\n{}",
                    entries_total,
                    total_missing,
                    slug,
                    preview.join("\n"),
                );
                print_envelope(&TdEnvelope::Error {
                    slug,
                    message: &msg,
                })?;
                return Ok(());
            } else {
                eprintln!(
                    "[td merge] WARNING: {} of {} spec-listed files missing on disk:",
                    total_missing, entries_total,
                );
                for line in &preview {
                    eprintln!("{}", line);
                }
            }
        }
    }

    // Atomic close: advance phase to td_merged AND state to closed, which
    // moves the issue file open/<slug>.md → closed/<slug>.md via
    // LocalBackend::write. Stage both paths and commit a single Td-Merged
    // trailer so the rename + frontmatter advance land together when the
    // worktree branch merges into main. Idempotent: skip if already at
    // td_merged (retry after partial failure).
    // @spec .aw/tech-design/projects/score/bugs/aw-td-merge-atomic-lifecycle.md
    if phase != "td_merged" {
        let patch = IssuePatch {
            state: Some(crate::issues::IssueState::Closed),
            phase: Some("td_merged".to_string()),
            ship_status: Some(crate::issues::ShipStatus::Step1Shipped),
            add_labels: vec!["phase:td_merged".to_string()],
            remove_labels: td_merge_labels_to_remove(),
            flagged_sections: Some(vec![]),
            validation_errors: Some(vec![]),
            ..Default::default()
        };
        backend.update(slug, &patch).await?;

        // Push the now-closed temp lifecycle issue file through the remote
        // backend so GitHub/GitLab is closed in lock-step with the local state.
        let closed_issue = backend
            .get(slug)
            .await?
            .ok_or_else(|| anyhow::anyhow!("closed issue '{}' was not readable", slug))?;
        let closed_path = backend.issue_path(&closed_issue);
        maybe_push_remote(&project_root, &closed_path, slug).await?;

        let git_bin = crate::git::find_git_bin()
            .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
        let commit_msg = format!(
            "td({slug}) \u{2014} merged + closed\n\n\
             Lifecycle-Slug: {slug}\n\
             Work-Item: {slug}\n\
             Lifecycle-Stage: Td-Merged",
        );
        let commit = std::process::Command::new(&git_bin)
            .arg("-C")
            .arg(&project_root)
            .args(["commit", "--allow-empty", "-m", &commit_msg])
            .output()
            .context("git commit failed")?;
        if !commit.status.success() {
            anyhow::bail!(
                "git commit failed: {}",
                String::from_utf8_lossy(&commit.stderr).trim()
            );
        }
    }

    if !dedicated_branch_mode {
        let msg = format!(
            "tech-design merged for '{}' on current branch '{}'",
            slug, starting_branch
        );
        print_envelope(&TdEnvelope::Done {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    let git_bin = crate::git::find_git_bin().ok_or_else(|| anyhow::anyhow!("git not found"))?;

    // Resolve effective target branch: CLI override → issue.target_branch → current branch → config fallback.
    let issue_target_branch = if dedicated_branch_mode && issue.target_branch.is_none() {
        Some(starting_branch.clone())
    } else {
        issue.target_branch.clone()
    };
    let target_branch = super::merge_target::resolve_merge_target(
        args.target_branch.clone(),
        issue_target_branch,
        &project_root,
    )
    .map_err(|e| {
        let _ = print_envelope(&TdEnvelope::Error {
            slug,
            message: &e.to_string(),
        });
        e
    })?;

    // Ensure the main repo is on the intended target branch before merging.
    let checkout_out = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(&project_root)
        .args(["checkout", &target_branch])
        .output()
        .context("git checkout failed")?;
    if !checkout_out.status.success() {
        let err = String::from_utf8_lossy(&checkout_out.stderr);
        let msg = format!("git checkout {} failed: {}", target_branch, err.trim());
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    let merge_out = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(&project_root)
        .args([
            "merge",
            "--no-ff",
            &branch,
            "-m",
            &format!("Merge tech-design td-{}", slug),
        ])
        .output()
        .context("git merge failed")?;

    if !merge_out.status.success() {
        let err = String::from_utf8_lossy(&merge_out.stderr);
        let msg = format!(
            "git merge {} into {} failed: {}. Resolve conflicts manually.",
            branch,
            target_branch,
            err.trim()
        );
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    // Clean up branch. Phase C: in-place only — no worktree dir to remove.
    // The host repo is the workspace; we already switched it to
    // `target_branch` above. Just delete the local `td-<slug>` branch since
    // the merge subsumed its history.
    let _ = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(&project_root)
        .args(["branch", "-d", &branch])
        .output();

    let msg = format!("tech-design merged for '{}'", slug);
    print_envelope(&TdEnvelope::Done {
        slug,
        message: &msg,
    })?;

    Ok(())
}

fn td_merge_labels_to_remove() -> Vec<String> {
    vec![
        super::workflow_guard::LOCK_LABEL.to_string(),
        super::workflow_guard::TD_LOCK_LABEL.to_string(),
        super::workflow_guard::CB_LOCK_LABEL.to_string(),
        "phase:td_reviewed".to_string(),
        "phase:td_gen_coded".to_string(),
        "phase:cb_genned".to_string(),
        "phase:cb_filled".to_string(),
        "phase:cb_reviewed".to_string(),
    ]
}

// ── td arbitrate ────────────────────────────────────────────────────

async fn run_arbitrate(args: ArbitrateArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = &args.slug;

    td_activate_inplace_if_present(&project_root, slug)?;
    let worktree_abs = td_workspace_path(&project_root, slug);
    if !worktree_abs.exists() {
        let msg = format!("workspace not found: {}", worktree_abs.display());
        print_envelope(&TdEnvelope::Error {
            slug,
            message: &msg,
        })?;
        return Ok(());
    }

    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;

    eprintln!(
        "\u{26a0} Tech-design for '{}' needs human arbitration.",
        slug
    );
    eprintln!("  Issue: {}", issue.title);
    eprintln!("  Checkout: {}", worktree_abs.display());
    eprintln!("  2 review rounds completed without approval.");
    eprintln!("  Read the # Reviews section in the spec, then either:");
    eprintln!("    - Approve: aw td merge {}", slug);
    eprintln!("    - Reset phase and re-run the TD lifecycle");

    print_envelope(&TdEnvelope::Done {
        slug,
        message: "escalated to human arbitration \u{2014} 2 review rounds exhausted",
    })?;

    Ok(())
}

// ── Audit ──────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct AuditEntry {
    spec: String,
    codegen_ready: bool,
    codegen_errors: Vec<String>,
    changes_count: usize,
    drift: Vec<DriftItem>,
}

#[derive(serde::Serialize)]
struct DriftItem {
    path: String,
    kind: String,
    detail: String,
}

#[derive(serde::Serialize)]
struct AuditReport {
    total_specs: usize,
    with_changes: usize,
    codegen_ready: usize,
    drift_total: usize,
    entries: Vec<AuditEntry>,
}

/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub(crate) fn run_audit(args: AuditArgs) -> Result<()> {
    use walkdir::WalkDir;

    // Legacy flag removal (R9 of split-validate-audit issue).
    if args.ready_only {
        anyhow::bail!(
            "`--ready-only` has been removed. The codegen-ready check has moved to \
             `aw td validate <path>`; audit is now code-side only. Run \
             `aw td validate .aw/tech-design/` to scan specs for codegen-readiness."
        );
    }
    if args.drift {
        anyhow::bail!(
            "`--drift` has been removed. Drift is now the default behavior of \
             `aw cb check <path>`, with Clean/Drift/MarkerGap/Uncovered classified \
             in one walk. Run `aw cb check projects/` to scan all code."
        );
    }

    let project_root = crate::find_project_root()?;

    // Path-mode: new unified code-side walk (R5 + R7).
    if let Some(target) = args.path.as_deref() {
        return run_audit_unified(target, args.json, args.group_by, &project_root);
    }

    let td_dir = crate::shared::workspace::tech_design_path(&project_root);

    if !td_dir.is_dir() {
        anyhow::bail!(
            "tech-design root not found in {}: {}",
            project_root.display(),
            td_dir.display()
        );
    }

    let mut entries = Vec::new();
    let mut total_specs = 0;

    for entry in WalkDir::new(&td_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map_or(true, |ext| ext != "md") {
            continue;
        }
        total_specs += 1;

        let spec_content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Check for Changes section
        let change_entries = extract_changes_for_audit(&spec_content);
        if change_entries.is_empty() && args.ready_only {
            continue;
        }

        // Codegen readiness
        let codegen_errors = check_codegen_ready(&spec_content);
        let codegen_ready = codegen_errors.is_empty()
            && !change_entries.is_empty()
            && has_mermaid_plus(&spec_content);

        if args.ready_only && !codegen_ready {
            continue;
        }

        // Check drift: file existence vs action
        let mut drift = Vec::new();
        for ce in &change_entries {
            let target = project_root.join(&ce.path);
            let exists = target.exists();
            let action_lower = ce.action.to_lowercase();

            if action_lower == "create" && exists {
                drift.push(DriftItem {
                    path: ce.path.clone(),
                    kind: "create_exists".to_string(),
                    detail: "action=create but file already exists".to_string(),
                });
            } else if (action_lower == "modify" || action_lower == "update") && !exists {
                drift.push(DriftItem {
                    path: ce.path.clone(),
                    kind: "modify_missing".to_string(),
                    detail: "action=modify but file does not exist".to_string(),
                });
            } else if !exists {
                drift.push(DriftItem {
                    path: ce.path.clone(),
                    kind: "file_missing".to_string(),
                    detail: "referenced file does not exist".to_string(),
                });
            }
        }

        let rel_spec = entry
            .path()
            .strip_prefix(&project_root)
            .unwrap_or(entry.path())
            .display()
            .to_string();

        entries.push(AuditEntry {
            spec: rel_spec,
            codegen_ready,
            codegen_errors,
            changes_count: change_entries.len(),
            drift,
        });
    }

    let with_changes = entries.iter().filter(|e| e.changes_count > 0).count();
    let codegen_ready_count = entries.iter().filter(|e| e.codegen_ready).count();
    let drift_total: usize = entries.iter().map(|e| e.drift.len()).sum();

    let report = AuditReport {
        total_specs,
        with_changes,
        codegen_ready: codegen_ready_count,
        drift_total,
        entries,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        eprintln!("Tech Design Audit");
        eprintln!("─────────────────");
        eprintln!("  Total specs:    {}", report.total_specs);
        eprintln!("  With Changes:   {}", report.with_changes);
        eprintln!("  Codegen ready:  {}", report.codegen_ready);
        eprintln!("  Drift items:    {}", report.drift_total);
        eprintln!();

        for entry in &report.entries {
            if entry.changes_count == 0 && entry.drift.is_empty() && entry.codegen_errors.is_empty()
            {
                continue;
            }

            let status = if entry.codegen_ready {
                "READY"
            } else {
                "-----"
            };
            eprintln!(
                "[{}] {} ({} files)",
                status, entry.spec, entry.changes_count
            );

            for err in &entry.codegen_errors {
                eprintln!("  codegen: {}", err);
            }
            for d in &entry.drift {
                eprintln!("  drift({}): {} — {}", d.kind, d.path, d.detail);
            }
        }
    }

    Ok(())
}

/// Path-mode audit: walk the given code-space target, run the unified
/// single-pass classifier (`audit_file_unified`), emit findings, exit
/// non-zero on any non-`Clean` result.
///
/// Accepts a single `.rs` file or a directory prefix. Other extensions
/// (`.py`, `.ts`) currently fall through the walker — we don't fail loud
/// yet because CODEGEN blocks in those languages use the same marker
/// grammar; the generators just haven't landed.
fn run_audit_unified(
    target: &str,
    json: bool,
    group_by: Option<AuditGroupBy>,
    project_root: &std::path::Path,
) -> Result<()> {
    use crate::generate::audit::{audit_file_unified, build_spec_file_index, UnifiedReport};
    use walkdir::WalkDir;

    let target_path = std::path::Path::new(target);
    let target_abs = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        project_root.join(target_path)
    };

    if !target_abs.exists() {
        anyhow::bail!("audit target not found: {}", target_abs.display());
    }

    let index = build_spec_file_index(project_root)?;

    let mut files: Vec<std::path::PathBuf> = Vec::new();
    if target_abs.is_file() {
        files.push(target_abs.clone());
    } else {
        for entry in WalkDir::new(&target_abs).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            // Only audit .rs today — CODEGEN markers exist in .py/.ts too but
            // the regenerate-diff side can't run without the generator.
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            files.push(path.to_path_buf());
        }
    }

    let mut all_reports: Vec<UnifiedReport> = Vec::new();
    for f in &files {
        match audit_file_unified(f, project_root, &index) {
            Ok(reports) => all_reports.extend(reports),
            Err(e) => eprintln!("  [ERROR] {}: {}", f.display(), e),
        }
    }

    // Count non-clean findings for exit code. Handwrite/Aggregate are
    // intentional and don't fail the audit; only true drift / coverage
    // gaps do.
    let non_clean: Vec<&UnifiedReport> = all_reports.iter().filter(|r| !r.is_clean()).collect();

    if json {
        if let Some(group_by) = group_by {
            println!(
                "{}",
                serde_json::to_string_pretty(&group_findings_json(&all_reports, group_by))?
            );
        } else {
            let json_findings: Vec<serde_json::Value> =
                all_reports.iter().map(unified_to_json).collect();
            println!("{}", serde_json::to_string_pretty(&json_findings)?);
        }
    } else {
        let mut clean = 0;
        let mut drift = 0;
        let mut marker_gap = 0;
        let mut uncovered = 0;
        let mut aggregate = 0;
        let mut unresolvable = 0;
        let mut handwrite = 0;
        for r in &all_reports {
            match r {
                UnifiedReport::Clean { .. } => clean += 1,
                UnifiedReport::Drift { .. } => drift += 1,
                UnifiedReport::MarkerGap { .. } => marker_gap += 1,
                UnifiedReport::Uncovered { .. } => uncovered += 1,
                UnifiedReport::Aggregate { .. } => aggregate += 1,
                UnifiedReport::Unresolvable { .. } => unresolvable += 1,
                UnifiedReport::Handwrite { .. } => handwrite += 1,
            }
        }
        eprintln!("── audit ─────────────────────────────────────────────────");
        eprintln!(
            "scanned {} file(s); {} clean / {} drift / {} marker-gap / {} uncovered / {} aggregate / {} unresolvable / {} handwrite",
            files.len(),
            clean,
            drift,
            marker_gap,
            uncovered,
            aggregate,
            unresolvable,
            handwrite,
        );
        if let Some(group_by) = group_by {
            print_grouped_text(&all_reports, group_by);
        } else {
            for r in &all_reports {
                match r {
                    UnifiedReport::Drift {
                        file,
                        spec_ref,
                        diff,
                    } => {
                        eprintln!("  [DRIFT] {} ({}): {}", file.display(), spec_ref, diff);
                    }
                    UnifiedReport::MarkerGap {
                        file,
                        item_line,
                        line_no,
                        enclosing_spec_ref,
                    } => {
                        eprintln!(
                            "  [MARKER-GAP] {}:{} — `{}` inside block {}",
                            file.display(),
                            line_no,
                            item_line.trim(),
                            enclosing_spec_ref,
                        );
                    }
                    UnifiedReport::Uncovered {
                        file,
                        item_line,
                        line_no,
                        claiming_specs,
                    } => {
                        eprintln!(
                            "  [UNCOVERED] {}:{} — `{}` (file claimed by {} spec(s))",
                            file.display(),
                            line_no,
                            item_line.trim(),
                            claiming_specs.len(),
                        );
                    }
                    UnifiedReport::Unresolvable {
                        file,
                        spec_ref,
                        reason,
                    } => {
                        eprintln!(
                            "  [UNRESOLVABLE] {} ({}): {}",
                            file.display(),
                            spec_ref,
                            reason
                        );
                    }
                    UnifiedReport::Handwrite {
                        file,
                        gap,
                        tracker,
                        line_start,
                        line_end,
                        ..
                    } => {
                        eprintln!(
                            "  [HANDWRITE] {}:{}-{} — gap={} tracker={}",
                            file.display(),
                            line_start,
                            line_end,
                            gap,
                            tracker,
                        );
                    }
                    UnifiedReport::Clean { .. } | UnifiedReport::Aggregate { .. } => {}
                }
            }
        }
    }

    if !non_clean.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

/// Group findings by `gap` / `file` / `status` and emit a sorted text
/// summary on stderr.
fn print_grouped_text(reports: &[crate::generate::audit::UnifiedReport], group_by: AuditGroupBy) {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<&crate::generate::audit::UnifiedReport>> = BTreeMap::new();
    for r in reports {
        let key = group_key(r, group_by);
        groups.entry(key).or_default().push(r);
    }
    for (key, members) in &groups {
        eprintln!("\n[{}] ({} finding(s))", key, members.len());
        for r in members {
            eprintln!("  • {}: {}", r.status(), r.file().display());
        }
    }
}

/// JSON shape for `--group-by`: an object keyed by group label with each
/// value an array of finding objects (same per-finding shape as the
/// flat output).
fn group_findings_json(
    reports: &[crate::generate::audit::UnifiedReport],
    group_by: AuditGroupBy,
) -> serde_json::Value {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    for r in reports {
        let key = group_key(r, group_by);
        groups.entry(key).or_default().push(unified_to_json(r));
    }
    serde_json::to_value(&groups).unwrap_or(serde_json::Value::Null)
}

fn group_key(r: &crate::generate::audit::UnifiedReport, group_by: AuditGroupBy) -> String {
    match group_by {
        AuditGroupBy::Gap => r.gap().unwrap_or("(none)").to_string(),
        AuditGroupBy::File => r.file().display().to_string(),
        AuditGroupBy::Status => r.status().to_string(),
    }
}

fn unified_to_json(r: &crate::generate::audit::UnifiedReport) -> serde_json::Value {
    use crate::generate::audit::UnifiedReport;
    match r {
        UnifiedReport::Clean { file, spec_ref } => serde_json::json!({
            "status": "clean", "file": file.display().to_string(), "spec_ref": spec_ref,
        }),
        UnifiedReport::Drift {
            file,
            spec_ref,
            diff,
        } => serde_json::json!({
            "status": "drift", "file": file.display().to_string(), "spec_ref": spec_ref, "diff": diff,
        }),
        UnifiedReport::MarkerGap {
            file,
            item_line,
            line_no,
            enclosing_spec_ref,
        } => serde_json::json!({
            "status": "marker_gap", "file": file.display().to_string(), "line": line_no,
            "item": item_line, "enclosing_spec_ref": enclosing_spec_ref,
        }),
        UnifiedReport::Uncovered {
            file,
            item_line,
            line_no,
            claiming_specs,
        } => serde_json::json!({
            "status": "uncovered", "file": file.display().to_string(), "line": line_no,
            "item": item_line,
            "claiming_specs": claiming_specs.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        }),
        UnifiedReport::Aggregate { file, spec_ref } => serde_json::json!({
            "status": "aggregate", "file": file.display().to_string(), "spec_ref": spec_ref,
        }),
        UnifiedReport::Unresolvable {
            file,
            spec_ref,
            reason,
        } => serde_json::json!({
            "status": "unresolvable", "file": file.display().to_string(), "spec_ref": spec_ref, "reason": reason,
        }),
        UnifiedReport::Handwrite {
            file,
            gap,
            tracker,
            reason,
            line_start,
            line_end,
        } => serde_json::json!({
            "status": "handwrite",
            "file": file.display().to_string(),
            "gap": gap,
            "tracker": tracker,
            "reason": reason,
            "line_start": line_start,
            "line_end": line_end,
        }),
    }
}

/// Legacy `--drift` pass (kept temporarily for reference; unreachable from
/// CLI after R9 flag removal — `run_audit_unified` is the new entry point).
#[allow(dead_code)]
fn _legacy_run_drift_audit_removed() {}

/// `--drift` pass: walk generated `.rs` files in the repo, run both
/// `audit_file` (content drift) and `audit_markers` (structural @spec gap)
/// on each, print a consolidated report, exit non-zero on any finding.
///
/// (Superseded by `run_audit_unified` after R9 flag removal; retained
/// temporarily as a reference implementation until the next cleanup pass.)
#[allow(dead_code)]
///
/// "Generated" here means the file contains at least one `CODEGEN-BEGIN`
/// marker — that's the cheap way to identify codegen-tracked files
/// without parsing every spec's `changes` list.
fn run_drift_audit(project_root: &std::path::Path) -> Result<()> {
    use crate::generate::audit::{audit_file, audit_markers, ReportKind};
    use walkdir::WalkDir;

    eprintln!();
    eprintln!("── drift + @spec coverage audit ──────────────────────────");

    let skip_prefixes = ["target", ".git", "node_modules"];

    let mut targets: Vec<std::path::PathBuf> = Vec::new();
    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().map_or(true, |ext| ext != "rs") {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(project_root)
            .unwrap_or(entry.path());
        if skip_prefixes.iter().any(|p| rel.starts_with(p)) {
            continue;
        }
        // Cheap pre-filter — only files that carry at least one CODEGEN
        // marker are relevant; everything else is hand-written.
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if !content.contains("CODEGEN-BEGIN") {
                continue;
            }
        } else {
            continue;
        }
        targets.push(entry.path().to_path_buf());
    }

    targets.sort();

    let mut total_blocks = 0usize;
    let mut drifts = Vec::new();
    let mut gaps = Vec::new();
    let mut unresolvable = Vec::new();
    let mut aggregate = 0usize;

    for t in &targets {
        let rel = t
            .strip_prefix(project_root)
            .unwrap_or(t)
            .display()
            .to_string();
        match audit_file(t, project_root) {
            Ok(reports) => {
                for r in reports {
                    total_blocks += 1;
                    match r.kind {
                        ReportKind::Clean => {}
                        ReportKind::Drift { diff } => {
                            drifts.push((rel.clone(), r.spec_ref.clone(), diff));
                        }
                        ReportKind::Aggregate => {
                            aggregate += 1;
                        }
                        ReportKind::Unresolvable { reason } => {
                            unresolvable.push((rel.clone(), r.spec_ref.clone(), reason));
                        }
                    }
                }
            }
            Err(e) => eprintln!("  [ERROR] audit_file {}: {}", rel, e),
        }
        match audit_markers(t) {
            Ok(gs) => {
                for g in gs {
                    gaps.push((rel.clone(), g.line_no, g.item_line, g.enclosing_spec_ref));
                }
            }
            Err(e) => eprintln!("  [ERROR] audit_markers {}: {}", rel, e),
        }
    }

    for (file, spec_ref, diff) in &drifts {
        eprintln!("[DRIFT]    {} ({}): {}", file, spec_ref, diff);
    }
    for (file, line, item, encl) in &gaps {
        eprintln!(
            "[GAP]      {}:{} (block: {}): item `{}` has no @spec marker",
            file, line, encl, item
        );
    }
    for (file, spec_ref, reason) in &unresolvable {
        eprintln!("[UNRESOLVE] {} ({}): {}", file, spec_ref, reason);
    }

    eprintln!();
    eprintln!(
        "audited {} files, {} codegen blocks: {} drift, {} gaps, {} unresolvable, {} aggregate (deferred)",
        targets.len(),
        total_blocks,
        drifts.len(),
        gaps.len(),
        unresolvable.len(),
        aggregate,
    );

    if !drifts.is_empty() || !gaps.is_empty() {
        anyhow::bail!(
            "drift audit failed: {} drift, {} gaps",
            drifts.len(),
            gaps.len(),
        );
    }
    Ok(())
}

/// Quick check for Mermaid Plus blocks (has ```mermaid with --- frontmatter).
fn has_mermaid_plus(content: &str) -> bool {
    let blocks = extract_mermaid_plus_blocks(content);
    blocks.iter().any(|b| !b.frontmatter_raw.is_empty())
}

/// Extract change entries for audit (minimal parser, no sdd dependency on private fn).
struct AuditChangeEntry {
    path: String,
    action: String,
}

fn extract_changes_for_audit(spec_content: &str) -> Vec<AuditChangeEntry> {
    let mut entries = Vec::new();
    let mut in_changes = false;
    let mut in_yaml = false;
    let mut yaml_content = String::new();

    for line in spec_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") && trimmed.to_lowercase().contains("change") {
            in_changes = true;
            continue;
        }
        if in_changes && trimmed.starts_with("## ") {
            break;
        }
        if in_changes && trimmed == "```yaml" {
            in_yaml = true;
            yaml_content.clear();
            continue;
        }
        if in_yaml && trimmed == "```" {
            // Parse YAML
            if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&yaml_content) {
                let files_key = val.get("files").or_else(|| val.get("changes"));
                if let Some(serde_yaml::Value::Sequence(files)) = files_key {
                    for f in files {
                        let path = f
                            .get("path")
                            .or_else(|| f.get("file"))
                            .and_then(|v| v.as_str());
                        let action = f
                            .get("action")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        if let Some(p) = path {
                            entries.push(AuditChangeEntry {
                                path: p.to_string(),
                                action: action.to_string(),
                            });
                        }
                    }
                }
            }
            in_yaml = false;
            continue;
        }
        if in_yaml {
            yaml_content.push_str(line);
            yaml_content.push('\n');
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    fn init_git_repo(root: &std::path::Path) {
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
            vec!["commit", "--allow-empty", "-m", "init", "-q"],
        ] {
            let out = std::process::Command::new("git")
                .args(&args)
                .current_dir(root)
                .output()
                .expect("git command");
            assert!(
                out.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }

    fn git_stdout(root: &std::path::Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("git command");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    #[test]
    fn td_branch_activation_only_uses_main() {
        assert!(should_use_td_branch("main"));
        assert!(!should_use_td_branch("project-score"));
        assert!(!should_use_td_branch("feature/sdd"));
        assert!(!should_use_td_branch("td-existing"));
    }

    fn issue_with_title(title: &str) -> Issue {
        Issue {
            issue_type: crate::issues::IssueType::Enhancement,
            title: title.to_string(),
            state: IssueState::Open,
            id: None,
            github_id: Some(3940),
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec!["project:jet".to_string()],
            created_at: None,
            updated_at: None,
            slug: "3940".to_string(),
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
        }
    }

    #[test]
    fn default_td_spec_path_uses_issue_title_not_numeric_id() {
        let issue =
            issue_with_title("enhancement(jet): emit parity-ready jet browser observation bundles");

        assert_eq!(
            default_spec_path_for_issue(&issue, "3940", "projects/jet/specs/"),
            ".aw/tech-design/projects/jet/specs/emit-parity-ready-jet-browser-observation-bundles.md"
        );
    }

    #[test]
    fn default_td_spec_path_falls_back_when_title_has_no_words() {
        let issue = issue_with_title("#3940");

        assert_eq!(
            default_spec_path_for_issue(&issue, "3940", "projects/jet/specs/"),
            ".aw/tech-design/projects/jet/specs/issue-3940.md"
        );
    }

    #[test]
    fn default_td_spec_path_uses_project_td_path_from_config() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        std::fs::write(
            tmp.path().join(".aw/config.toml"),
            r#"
[[projects]]
name = "agentic-workflow"
aliases = ["aw"]
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design"
label = "project:agentic-workflow"
"#,
        )
        .unwrap();

        let mut issue = issue_with_title("Manage AW init templates as greenfield-ready artifacts");
        issue.labels = vec!["project:agentic-workflow".to_string()];

        assert_eq!(
            default_spec_path_for_issue_in_project(tmp.path(), &issue, "4162"),
            "projects/agentic-workflow/tech-design/logic/manage-aw-init-templates-as-greenfield-ready-artifacts.md"
        );
    }

    #[test]
    fn td_lifecycle_commit_helper_preserves_validate_stage_trailer() {
        if !git_available() {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_git_repo(root);
        let issue_store = tempfile::tempdir().unwrap();
        let issue = issue_store.path().join("issues/open/123.md");
        std::fs::create_dir_all(issue.parent().unwrap()).unwrap();
        std::fs::write(&issue, "# issue\n").unwrap();
        let issue_arg = issue.to_string_lossy().into_owned();

        commit_lifecycle_with_extra(
            root,
            "123",
            "workflow lock completed",
            "Td-Lock-Complete",
            &[issue_arg.as_str()],
            &[("Lifecycle-Phase", "td_created")],
        )
        .unwrap();

        let log = git_stdout(root, &["log", "-1", "--pretty=%B"]);
        assert!(log.contains("Lifecycle-Slug: 123"));
        assert!(log.contains("Lifecycle-Stage: Td-Lock-Complete"));
        assert!(log.contains("Lifecycle-Phase: td_created"));
    }

    #[test]
    fn dirty_payload_prefix_is_allowed_for_td_revise_apply() {
        if !git_available() {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_git_repo(root);
        let payload = root.join(".aw/payloads/1/changes.md");
        std::fs::create_dir_all(payload.parent().unwrap()).unwrap();
        std::fs::write(&payload, "## Changes\n").unwrap();

        ensure_clean_or_only_dirty_paths(root, &["tech-design/spec.md", ".aw/payloads/1/"])
            .expect("td revise apply should allow matching payload fragments");
    }

    #[test]
    fn dirty_marker_payload_file_is_allowed_for_cb_fill_apply() {
        if !git_available() {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_git_repo(root);
        let payload = root.join(".aw/payloads/1/missing-generator:component:e4ee6075.md");
        std::fs::create_dir_all(payload.parent().unwrap()).unwrap();
        std::fs::write(&payload, "fill\n").unwrap();

        ensure_clean_or_only_dirty_paths(
            root,
            &[".aw/payloads/1/missing-generator:component:e4ee6075.md"],
        )
        .expect("cb fill apply should allow the expected marker payload");
    }

    #[test]
    fn merge_spec_section_replaces_matching_type() {
        let base = concat!(
            "---\n",
            "id: x\n",
            "fill_sections: [overview, logic, changes]\n",
            "---\n\n",
            "#",
            "# Overview\n",
            "<",
            "!-- type: overview lang: markdown -->\n\n",
            "Old overview.\n\n",
            "#",
            "# Logic\n",
            "<",
            "!-- type: logic lang: mermaid -->\n\n",
            "TBD\n\n",
            "#",
            "# Changes\n",
            "<",
            "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes: []\n",
            "```\n",
        );
        let payload = concat!(
            "#",
            "# Logic: validate\n",
            "<",
            "!-- type: logic lang: mermaid -->\n\n",
            "new logic body\n",
        );
        let merged = merge_spec_section(base, "logic", payload).unwrap();
        // Logic section is replaced with the payload's heading + body.
        assert!(
            merged.contains("## Logic: validate"),
            "merged should use payload heading, got:
{}",
            merged
        );
        assert!(merged.contains("new logic body"));
        assert!(
            !merged.contains("TBD"),
            "old TBD logic content should be gone"
        );
        // Other sections untouched.
        assert!(merged.contains("## Overview"));
        assert!(merged.contains("Old overview."));
        assert!(merged.contains("## Changes"));
        assert!(merged.contains("changes: []"));
    }

    #[test]
    fn merge_spec_section_appends_when_type_missing() {
        let base = concat!(
            "---\n",
            "id: x\n",
            "---\n\n",
            "#",
            "# Overview\n",
            "<",
            "!-- type: overview lang: markdown -->\n\n",
            "body\n",
        );
        let payload = concat!(
            "#",
            "# Logic\n",
            "<",
            "!-- type: logic lang: mermaid -->\n\n",
            "new section\n",
        );
        let merged = merge_spec_section(base, "logic", payload).unwrap();
        // Original content preserved.
        assert!(merged.contains("## Overview"));
        // New section appended.
        let ov_idx = merged.find("## Overview").unwrap();
        let logic_idx = merged.find("## Logic").unwrap();
        assert!(
            logic_idx > ov_idx,
            "Logic should be appended after Overview"
        );
    }

    #[test]
    fn merge_spec_section_appends_missing_type_to_fill_sections() {
        let base = concat!(
            "---\n",
            "id: x\n",
            "fill_sections: [logic]\n",
            "---\n\n",
            "#",
            "# Logic\n",
            "<",
            "!-- type: logic lang: mermaid -->\n\n",
            "logic\n",
        );
        let payload = concat!(
            "#",
            "# Changes\n",
            "<",
            "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes: []\n",
            "```\n",
        );
        let merged = merge_spec_section(base, "changes", payload).unwrap();

        assert!(merged.contains("fill_sections: [logic, changes]"));
        assert!(merged.contains("## Changes"));
    }

    #[test]
    fn merge_spec_section_repairs_fill_sections_for_existing_type() {
        let base = concat!(
            "---\n",
            "id: x\n",
            "fill_sections: [logic]\n",
            "---\n\n",
            "#",
            "# Logic\n",
            "<",
            "!-- type: logic lang: mermaid -->\n\n",
            "logic\n\n",
            "#",
            "# Changes\n",
            "<",
            "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes: []\n",
            "```\n",
        );
        let payload = concat!(
            "#",
            "# Changes\n",
            "<",
            "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes:\n",
            "  - path: src/lib.rs\n",
            "    action: create\n",
            "    section: logic\n",
            "    impl_mode: hand-written\n",
            "```\n",
        );
        let merged = merge_spec_section(base, "changes", payload).unwrap();

        assert!(merged.contains("fill_sections: [logic, changes]"));
        assert!(merged.contains("path: src/lib.rs"));
    }

    #[test]
    fn merge_spec_section_replaces_duplicate_matching_types_once() {
        let base = concat!(
            "---\n",
            "id: x\n",
            "---\n\n",
            "#",
            "# Scenarios\n",
            "<",
            "!-- type: scenarios lang: yaml -->\n\n",
            "old first\n",
            "#",
            "# Scenarios\n",
            "<",
            "!-- type: scenarios lang: yaml -->\n\n",
            "old duplicate\n",
            "#",
            "# Logic\n",
            "<",
            "!-- type: logic lang: mermaid -->\n\n",
            "logic body\n",
        );
        let payload = concat!(
            "#",
            "# Scenarios\n",
            "<",
            "!-- type: scenarios lang: yaml -->\n\n",
            "new scenarios\n",
        );

        let merged = merge_spec_section(base, "scenarios", payload).unwrap();

        assert!(merged.contains("new scenarios"));
        assert!(!merged.contains("old first"));
        assert!(!merged.contains("old duplicate"));
        assert_eq!(
            merged
                .matches("<!-- type: scenarios lang: yaml -->")
                .count(),
            1
        );
        assert!(merged.contains("logic body"));
    }

    #[test]
    fn preserve_dest_rel_uses_in_tree_path_when_src_under_td() {
        // Source already under .aw/tech-design/<sub>/ — must mirror the
        // exact relative path, ignoring labels. Without the fix this would
        // flatten to the label-derived dir and ship a duplicate spec on
        // td merge.
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path();
        let src_rel =
            std::path::Path::new(".aw/tech-design/crates/cclab-agent/agent-protocols-spec.md");
        let src_abs = project_root.join(src_rel);
        std::fs::create_dir_all(src_abs.parent().unwrap()).unwrap();
        std::fs::write(&src_abs, "stub").unwrap();
        // Labels would otherwise pick projects/score/specs/ — confirm we
        // ignore them when src is in-tree.
        let labels = vec![];
        let dest = preserve_or_derive_dest_rel(
            &src_abs,
            project_root,
            &labels,
            "claim-agent-protocols-spec",
        );
        assert_eq!(
            dest, ".aw/tech-design/crates/cclab-agent/agent-protocols-spec.md",
            "in-tree spec must be mirrored to its original location, not flattened",
        );
    }

    #[test]
    fn preserve_dest_rel_falls_back_to_label_dir_when_src_outside_td() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path();
        let src_abs = project_root.join("scratch/external-spec.md");
        std::fs::create_dir_all(src_abs.parent().unwrap()).unwrap();
        std::fs::write(&src_abs, "stub").unwrap();
        // Also ensure the td_root exists so the canonicalize path is
        // exercised.
        std::fs::create_dir_all(project_root.join(".aw/tech-design")).unwrap();
        let labels = vec!["crate:sdd".to_string()];
        let dest = preserve_or_derive_dest_rel(&src_abs, project_root, &labels, "some-slug");
        assert_eq!(
            dest, ".aw/tech-design/projects/agentic-workflow/logic/external-spec.md",
            "out-of-tree src should fall back to label-derived dir",
        );
    }

    #[test]
    fn derive_spec_dir_uses_project_label_name() {
        let labels = vec!["type:enhancement".to_string(), "project:cue".to_string()];
        assert_eq!(derive_spec_dir(&labels), "projects/cue/logic/");
    }

    #[test]
    fn derive_spec_dir_preserves_crate_routing() {
        let labels = vec!["type:bug".to_string(), "crate:sdd".to_string()];
        assert_eq!(derive_spec_dir(&labels), "projects/agentic-workflow/logic/");
    }

    #[test]
    fn derive_spec_dir_routes_issue_title_to_ddd_concern() {
        let issue = issue_with_title("enhancement(jet): browser cli protocol schema");
        assert_eq!(
            derive_spec_dir_for_issue(&issue),
            "projects/jet/interfaces/"
        );

        let issue = issue_with_title("test(jet): parity fixture conformance gate");
        assert_eq!(derive_spec_dir_for_issue(&issue), "projects/jet/validate/");
    }

    #[test]
    fn issue_open_rel_uses_local_issue_slug_when_it_differs_from_tracker_id() {
        let issue = crate::issues::Issue {
            issue_type: crate::issues::IssueType::Enhancement,
            title: "test issue".into(),
            state: crate::issues::IssueState::Open,
            id: None,
            github_id: Some(2222),
            gitlab_id: None,
            url: None,
            author: None,
            labels: Vec::new(),
            created_at: None,
            updated_at: None,
            slug: "bug-r6a-locate-in-crate-spec-root-picks-first".into(),
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
        };

        let tmp = tempfile::tempdir().unwrap();
        let backend = LocalBackend::at(tmp.path().join("issues"));
        assert!(issue_path_arg(&backend, &issue)
            .ends_with("issues/open/bug-r6a-locate-in-crate-spec-root-picks-first.md"));
    }

    #[test]
    fn issue_path_arg_uses_tracker_slug_for_stub_issue() {
        let issue = crate::issues::Issue {
            issue_type: crate::issues::IssueType::Enhancement,
            title: "test issue".into(),
            state: crate::issues::IssueState::Open,
            id: None,
            github_id: Some(2222),
            gitlab_id: None,
            url: None,
            author: None,
            labels: Vec::new(),
            created_at: None,
            updated_at: None,
            slug: String::new(),
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
        };

        let tmp = tempfile::tempdir().unwrap();
        let backend = LocalBackend::at(tmp.path().join("issues"));
        assert!(issue_path_arg(&backend, &issue).ends_with("issues/open/2222.md"));
    }

    #[test]
    fn lifecycle_staging_skips_absolute_retired_aw_issue_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let worktree = tmp.path();
        let issue_path = worktree.join(".aw/issues/open/4124.md");

        assert!(!should_stage_lifecycle_path(
            worktree,
            &issue_path.to_string_lossy()
        ));
    }

    #[test]
    fn td_merge_label_cleanup_removes_stale_phase_and_lock_labels() {
        let labels = td_merge_labels_to_remove();

        assert!(labels.contains(&crate::cli::workflow_guard::LOCK_LABEL.to_string()));
        assert!(labels.contains(&crate::cli::workflow_guard::TD_LOCK_LABEL.to_string()));
        assert!(labels.contains(&"phase:cb_genned".to_string()));
        assert!(labels.contains(&"phase:cb_reviewed".to_string()));
    }

    #[test]
    fn td_section_queue_excludes_deprecated_and_legacy_metadata_types() {
        let queue = td_section_queue("applicability");
        assert_eq!(queue, vec!["logic".to_string(), "unit-test".to_string()]);
        assert!(!queue.contains(&"overview".to_string()));
        assert!(!queue.contains(&"requirements".to_string()));
        assert!(!queue.contains(&"doc".to_string()));
        assert!(!queue.contains(&"scenarios".to_string()));
        assert!(!queue.contains(&"changes".to_string()));
        assert!(queue.contains(&"unit-test".to_string()));
        assert!(queue.contains(&"logic".to_string()));
        for section in &queue {
            let section_type = section
                .parse::<crate::models::spec_rules::SectionType>()
                .unwrap();
            assert!(
                !crate::generate::generators::primitive_registry::is_prose_section(section_type),
                "{section} should not be in the active new-TD queue"
            );
        }
    }

    #[test]
    fn td_section_payload_template_scaffolds_typed_section() {
        let template = td_section_payload_template("logic").unwrap();
        assert!(template.contains("## Logic"));
        assert!(template.contains("<!-- type: logic lang: mermaid -->"));
        assert!(template.contains("```mermaid"));
        assert!(template.contains("(fill)"));
    }

    #[test]
    fn td_section_payload_template_rejects_new_scenarios() {
        let err = td_section_payload_template("scenarios").unwrap_err();
        assert!(err
            .to_string()
            .contains("section 'scenarios' is not supported for new TD payloads"));
    }

    #[test]
    fn remaining_after_section_uses_spec_fill_sections_when_present() {
        let spec = concat!(
            "---\n",
            "id: selected-sections\n",
            "fill_sections: [logic, cli, unit-test]\n",
            "---\n\n",
            "## Logic\n",
            "<!-- type: logic lang: mermaid -->\n\n",
            "```mermaid\n",
            "---\n",
            "id: selected_sections\n",
            "entry: start\n",
            "nodes:\n",
            "  start: { kind: start }\n",
            "edges: []\n",
            "---\n",
            "flowchart TD\n",
            "```\n",
        );

        assert_eq!(
            remaining_after_section_in_content(spec, "applicability", "logic"),
            vec!["cli".to_string(), "unit-test".to_string()]
        );
    }

    #[test]
    fn td_review_payload_template_requires_agent_edit() {
        let template = td_review_payload_template(2);
        assert!(template.contains("# Reviews"));
        assert!(template.contains("### Review 2"));
        assert!(template.contains("**Verdict:** <verdict>"));
        assert!(template.contains("(fill)"));
        assert!(!template.contains("**Verdict:** approved"));
        assert!(!template.contains("**Verdict:** needs-revision"));
    }

    #[test]
    fn initialize_td_payload_file_preserves_existing_content() {
        let tmp = tempfile::tempdir().unwrap();
        let rel = ".aw/payloads/123/applicability/logic.md";

        assert!(initialize_td_payload_file(tmp.path(), rel, "first\n").unwrap());
        assert_eq!(
            std::fs::read_to_string(tmp.path().join(rel)).unwrap(),
            "first\n"
        );

        assert!(!initialize_td_payload_file(tmp.path(), rel, "second\n").unwrap());
        assert_eq!(
            std::fs::read_to_string(tmp.path().join(rel)).unwrap(),
            "first\n"
        );
    }

    #[test]
    fn td_error_envelope_returns_error_for_shell_control_flow() {
        let err = td_error("3940", "td content validation failed")
            .expect_err("error envelopes must produce a non-zero command result");
        assert!(err.to_string().contains("td content validation failed"));
    }

    #[test]
    fn section_apply_validation_allows_future_sections_to_be_missing() {
        let spec = concat!(
            "---\n",
            "id: section-apply\n",
            "fill_sections: [schema, logic, changes]\n",
            "---\n\n",
            "# Section Apply\n\n",
            "#",
            "# Schema\n",
            "<",
            "!-- type: schema lang: yaml -->\n\n",
            "```yaml\n",
            "kind: applied\n",
            "```\n",
        );

        assert!(validate_spec_for_section_apply(spec, "schema").is_empty());
        assert!(validate_spec(spec)
            .iter()
            .any(|error| error.contains("section type 'logic'")));
    }

    #[test]
    fn section_apply_validation_requires_current_section_in_fill_sections() {
        let spec = concat!(
            "---\n",
            "id: section-apply\n",
            "fill_sections: [schema, changes]\n",
            "---\n\n",
            "# Section Apply\n\n",
            "#",
            "# Logic\n",
            "<",
            "!-- type: logic lang: mermaid -->\n\n",
            "```mermaid\n",
            "flowchart TD\n",
            "  A --> B\n",
            "```\n",
        );

        let errors = validate_spec_for_section_apply(spec, "logic");
        assert!(errors
            .iter()
            .any(|error| error.contains("current section 'logic'")));
    }

    #[test]
    fn legacy_extract_sections_ignores_headings_inside_long_source_fence() {
        let body = concat!(
            "# Demo\n\n",
            "## Source\n",
            "<!-- type: source lang: rust -->\n\n",
            "```````rust\n",
            "fn fixture() {\n",
            "    let md = r#\"\n",
            "## Source\n",
            "<!-- type: source lang: tsx -->\n",
            "```tsx\n",
            "export function App() {}\n",
            "```\n",
            "\n",
            "## Changes\n",
            "<!-- type: changes lang: yaml -->\n",
            "\"#;\n",
            "}\n",
            "```````\n\n",
            "## Changes\n",
            "<!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes: []\n",
            "```\n",
        );

        let sections = extract_sections(body);
        let found: Vec<_> = sections
            .iter()
            .map(|(_heading, ann, _content)| ann.section_type.as_str())
            .collect();
        assert_eq!(found, vec!["source", "changes"]);
    }

    #[test]
    fn shared_td_content_gate_allows_specs_without_legacy_changes_section() {
        let dir = tempfile::TempDir::new().unwrap();
        let spec_path = dir.path().join("no-changes.md");
        let spec = concat!(
            "---\n",
            "id: no-changes\n",
            "fill_sections: [schema]\n",
            "---\n\n",
            "# No Changes\n\n",
            "#",
            "# Schema\n",
            "<",
            "!-- type: schema lang: yaml -->\n\n",
            "```yaml\n",
            "kind: example\n",
            "```\n",
        );
        std::fs::write(&spec_path, spec).unwrap();

        let report =
            validate_td_content_file(&spec_path, TdContentValidationScope::Complete).unwrap();
        assert!(
            !report.has_errors(),
            "new TDs should not need a legacy changes section: {:?}",
            report.findings
        );
    }

    #[test]
    fn shared_td_content_gate_tolerates_legacy_scenarios_section() {
        let dir = tempfile::TempDir::new().unwrap();
        let spec_path = dir.path().join("legacy-scenarios.md");
        let spec = concat!(
            "---\n",
            "id: legacy-scenarios\n",
            "fill_sections: [scenarios]\n",
            "---\n\n",
            "# Legacy Scenarios\n\n",
            "#",
            "# Scenarios\n",
            "<",
            "!-- type: scenarios lang: yaml -->\n\n",
            "```yaml\n",
            "scenarios:\n",
            "  - id: S1\n",
            "    given: old TD prose scenario exists\n",
            "    when: shared validation scans it\n",
            "    then: it remains tolerated\n",
            "```\n",
        );
        std::fs::write(&spec_path, spec).unwrap();

        let report =
            validate_td_content_file(&spec_path, TdContentValidationScope::Complete).unwrap();
        assert!(
            !report.has_errors(),
            "legacy scenarios should be tolerated by shared validation: {:?}",
            report.findings
        );
    }

    #[test]
    fn new_td_authoring_gate_rejects_scenarios_section() {
        let dir = tempfile::TempDir::new().unwrap();
        let spec_path = dir.path().join("new-scenarios.md");
        let spec = concat!(
            "---\n",
            "id: new-scenarios\n",
            "fill_sections: [scenarios]\n",
            "---\n\n",
            "# New Scenarios\n\n",
            "#",
            "# Scenarios\n",
            "<",
            "!-- type: scenarios lang: yaml -->\n\n",
            "```yaml\n",
            "scenarios:\n",
            "  - id: S1\n",
            "    given: new TD authoring starts\n",
            "    when: it includes scenarios\n",
            "    then: the lifecycle rejects it\n",
            "```\n",
        );
        std::fs::write(&spec_path, spec).unwrap();

        let report =
            validate_new_td_authoring_file(&spec_path, TdContentValidationScope::Complete).unwrap();
        let errors = td_content_error_messages(&report);
        assert!(
            errors.iter().any(|error| error
                .contains("new TDs must not include legacy prose section type 'scenarios'")),
            "new TD authoring should reject scenarios: {errors:?}"
        );
    }

    #[test]
    fn shared_td_content_gate_supports_section_apply_partial_specs() {
        let dir = tempfile::TempDir::new().unwrap();
        let spec_path = dir.path().join("partial.md");
        let spec = concat!(
            "---\n",
            "id: section-apply\n",
            "fill_sections: [schema, logic]\n",
            "---\n\n",
            "# Section Apply\n\n",
            "#",
            "# Schema\n",
            "<",
            "!-- type: schema lang: yaml -->\n\n",
            "```yaml\n",
            "kind: applied\n",
            "```\n",
        );
        std::fs::write(&spec_path, spec).unwrap();

        let partial = validate_td_content_file(
            &spec_path,
            TdContentValidationScope::RequireThrough("schema"),
        )
        .unwrap();
        assert!(
            !partial.has_errors(),
            "partial section apply should pass: {:?}",
            partial.findings
        );

        let complete =
            validate_td_content_file(&spec_path, TdContentValidationScope::Complete).unwrap();
        let errors = td_content_error_messages(&complete);
        assert!(errors
            .iter()
            .any(|error| error.contains("section type 'logic'")));
    }

    #[test]
    fn shared_td_content_gate_requires_section_implementation_edges() {
        let dir = tempfile::TempDir::new().unwrap();
        let spec_path = dir.path().join("missing-impl-mode.md");
        let spec = concat!(
            "---\n",
            "id: missing-impl-mode\n",
            "fill_sections: [schema, changes]\n",
            "---\n\n",
            "# Missing Impl Mode\n\n",
            "#",
            "# Schema\n",
            "<",
            "!-- type: schema lang: yaml -->\n\n",
            "```yaml\n",
            "definitions: {}\n",
            "```\n\n",
            "#",
            "# Changes\n",
            "<",
            "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes:\n",
            "  - path: src/foo.rs\n",
            "    action: modify\n",
            "    section: schema\n",
            "```\n",
        );
        std::fs::write(&spec_path, spec).unwrap();

        let report =
            validate_td_content_file(&spec_path, TdContentValidationScope::Complete).unwrap();
        let errors = td_content_error_messages(&report);

        assert!(errors
            .iter()
            .any(|error| error.contains("missing impl_mode")));
        assert!(errors.iter().any(|error| error.contains(
            "section type 'schema' has no changes[] entry with matching section and impl_mode"
        )));
    }

    #[test]
    fn shared_td_content_gate_accepts_handwritten_section_edge() {
        let spec = concat!(
            "---\n",
            "id: handwritten-section\n",
            "fill_sections: [schema, changes]\n",
            "---\n\n",
            "# Handwritten Section\n\n",
            "#",
            "# Schema\n",
            "<",
            "!-- type: schema lang: yaml -->\n\n",
            "```yaml\n",
            "definitions: {}\n",
            "```\n\n",
            "#",
            "# Changes\n",
            "<",
            "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes:\n",
            "  - path: src/foo.rs\n",
            "    action: modify\n",
            "    section: schema\n",
            "    impl_mode: hand-written\n",
            "```\n",
        );

        assert!(validate_spec(spec).is_empty());
    }

    #[test]
    fn merge_spec_section_preserves_frontmatter() {
        let base = concat!(
            "---\n",
            "id: x\n",
            "---\n\n",
            "#",
            "# Schema\n",
            "<",
            "!-- type: schema lang: yaml -->\n\n",
            "old\n",
        );
        let payload = concat!(
            "#",
            "# Schema\n",
            "<",
            "!-- type: schema lang: yaml -->\n\n",
            "new\n",
        );
        let merged = merge_spec_section(base, "schema", payload).unwrap();
        assert!(
            merged.starts_with(
                "---
id: x
fill_sections: [schema]
---"
            ),
            "frontmatter id should be preserved and fill_sections should be repaired"
        );
        assert!(merged.contains("new"));
        assert!(!merged.contains("old"));
    }
}

// ── td claim (recovery) ──────────────────────────────────────────────

/// `aw td claim <slug>` — recovery verb.
///
/// Adopt an on-disk TD spec into the score lifecycle. Switches to or creates
/// the `td-<slug>` branch only when launched from `main`; otherwise it stays
/// on the current branch. It sets `phase: td_reviewed`, commits the
/// `Lifecycle-Stage: Td-Claim` trailer with a `Claim-Source:` sub-trailer, and
/// emits a dispatch envelope to `aw cb gen`. Idempotent on re-run when the
/// active branch already carries the trailer (use `--force-rebase` to re-run).
/// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/td.md#source
pub async fn run_claim(args: TdClaimArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = &args.slug;

    // R1+R2: Resolve issue. The (None, Some) arm builds a stub that is written
    // after branch activation, so all issue writes stay in the temp issue store.
    let backend = LocalBackend::from_project_root(&project_root);
    let issue_opt = backend.get(slug).await?;
    let (issue, needs_stub_in_worktree) = match (issue_opt, args.from_path.as_deref()) {
        (Some(i), _) => (i, false),
        (None, Some(_path)) => {
            let title = format!("Adopted: {}", slug);
            let stub = crate::issues::Issue {
                issue_type: crate::issues::IssueType::Enhancement,
                title: title.clone(),
                state: IssueState::Open,
                id: None,
                github_id: None,
                gitlab_id: None,
                url: None,
                author: None,
                labels: Vec::new(),
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                updated_at: Some(chrono::Utc::now().to_rfc3339()),
                slug: slug.to_string(),
                body: format!(
                    "# {}\n\nIssue stub created by `aw td claim --from-path`.\n",
                    title
                ),
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
            };
            (stub, true)
        }
        (None, None) => {
            let msg = format!(
                "issue '{}' not found in the temp issue store (pass --from-path to auto-create stub)",
                slug
            );
            print_envelope(&TdEnvelope::Error {
                slug,
                message: &msg,
            })?;
            std::process::exit(1);
        }
    };

    // 2. Branch state machine (Phase C: in-place — no worktree dir).
    let current_branch = crate::branch_switch::current_branch(&project_root)?;
    let trailer_branch = if should_use_td_branch(&current_branch) {
        td_branch_name(slug)
    } else {
        current_branch
    };
    let trailer_present =
        branch_has_trailer(&project_root, &trailer_branch, "Td-Claim").unwrap_or(false);

    if trailer_present && !args.force_rebase {
        // R3: idempotent no-op
        print_envelope(&TdEnvelope::Done {
            slug,
            message: "td claim: already claimed (no-op); pass --force-rebase to overwrite",
        })?;
        return Ok(());
    }

    // Activate the workspace. Idempotent: reuses or creates `td-<slug>` when
    // launched from `main`; otherwise the current branch is the workspace.
    let active_branch = activate_td_workspace_for_lifecycle(&project_root, slug)
        .context("Failed to enter tech-design workspace for claim")?;

    // R1+R3: Create stub in the current checkout after branch activation.
    // Idempotency guard via `wt_backend.get` — partial-resume re-runs see the
    // existing stub and skip the create.
    let wt_backend = LocalBackend::from_project_root(&project_root);
    if needs_stub_in_worktree && wt_backend.get(slug).await?.is_none() {
        wt_backend.create(&issue).await?;
    }

    // 3. Copy spec into the current checkout (only when --from-path supplied; partial
    // resume without the flag converges via update_phase per the spec
    // logic flowchart).
    //
    // Path preservation: when `--from-path` already points at a file under
    // `.aw/tech-design/`, mirror its exact relative path into the
    // checkout. Otherwise fall back to label-derived dir + file_name.
    // Without this, claiming an in-tree spec produced a duplicate at the
    // label-derived location (e.g. `crates/cclab-agent/agent-protocols-spec.md`
    // → `projects/score/specs/agent-protocols-spec.md`) and shipped both
    // copies on `td merge`.
    let mut spec_path_in_worktree: Option<String> = None;
    if let Some(src_path) = args.from_path.as_deref() {
        let src = std::path::Path::new(src_path);
        if !src.exists() {
            let msg = format!("--from-path not found on disk: {}", src_path);
            print_envelope(&TdEnvelope::Error {
                slug,
                message: &msg,
            })?;
            std::process::exit(1);
        }
        let dest_rel = preserve_or_derive_dest_rel(src, &project_root, &issue.labels, slug);
        let dest_abs = project_root.join(&dest_rel);
        if let Some(parent) = dest_abs.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::copy(src, &dest_abs).with_context(|| {
            format!(
                "copying spec from {} to {}",
                src.display(),
                dest_abs.display()
            )
        })?;
        spec_path_in_worktree = Some(dest_rel);
    }

    // 4. R5: Set phase: td_reviewed with disambiguating error context.
    let patch = IssuePatch {
        phase: Some(crate::issues::types::td_phase::TD_REVIEWED.to_string()),
        branch: Some(active_branch),
        ..Default::default()
    };
    wt_backend
        .update(slug, &patch)
        .await
        .with_context(|| format!("issue file missing in workspace {}", project_root.display()))?;

    // 5. Commit Lifecycle-Stage: Td-Claim with Claim-Source sub-trailer
    let claim_source = args.from_path.clone().unwrap_or_else(|| {
        spec_path_in_worktree
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string())
    });
    let updated_issue = wt_backend
        .get(slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found after claim update", slug))?;
    let issue_path = wt_backend.issue_path(&updated_issue);
    let issue_path_s = issue_path.to_string_lossy().into_owned();
    let mut paths: Vec<&str> = vec![issue_path_s.as_str()];
    if let Some(ref sp) = spec_path_in_worktree {
        paths.push(sp.as_str());
    }
    maybe_push_remote(&project_root, &issue_path, slug).await?;
    commit_lifecycle_with_extra(
        &project_root,
        slug,
        &format!("claim {}", claim_source),
        "Td-Claim",
        &paths,
        &[("Claim-Source", &claim_source), ("Claim-Type", "td-spec")],
    )?;

    // 6. Emit dispatch envelope to aw cb gen
    let invoke_args = match spec_path_in_worktree {
        Some(sp) => serde_json::json!({ "slug": slug, "spec_path": sp }),
        None => serde_json::json!({ "slug": slug }),
    };
    print_envelope(&TdEnvelope::Dispatch {
        agent: None,
        slug,
        invoke: Invoke {
            command: "aw cb gen",
            args: invoke_args,
        },
    })?;

    let _ = args.json; // pretty-printed by default
    Ok(())
}

/// Check whether the given branch's git history contains a commit with the
/// given `Lifecycle-Stage:` trailer value.
fn branch_has_trailer(repo: &std::path::Path, branch: &str, stage: &str) -> Option<bool> {
    let git_bin = crate::git::find_git_bin()?;
    let out = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(repo)
        .args(["log", "--format=%B", branch])
        .output()
        .ok()?;
    if !out.status.success() {
        return Some(false);
    }
    let body = String::from_utf8_lossy(&out.stdout);
    let needle = format!("Lifecycle-Stage: {}", stage);
    Some(body.contains(&needle))
}

/// Like `commit_lifecycle` but appends extra trailers (e.g. `Claim-Source:`,
/// `Claim-Type:`) below `Lifecycle-Stage:`.
fn commit_lifecycle_with_extra(
    worktree_path: &std::path::Path,
    slug: &str,
    detail: &str,
    stage: &str,
    paths: &[&str],
    extra_trailers: &[(&str, &str)],
) -> Result<()> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;

    stage_lifecycle_paths(worktree_path, &git_bin, paths)?;

    let mut msg = format!(
        "td({slug}) \u{2014} {detail}\n\n\
         Lifecycle-Slug: {slug}\n\
         Work-Item: {slug}\n\
         Lifecycle-Stage: {stage}",
    );
    for (k, v) in extra_trailers {
        msg.push_str(&format!("\n{}: {}", k, v));
    }

    let mut command = std::process::Command::new(&git_bin);
    command.arg("-C").arg(worktree_path).arg("commit");
    if !has_staged_changes(worktree_path, &git_bin)? {
        command.arg("--allow-empty");
    }
    let commit = command
        .args(["-m", &msg])
        .output()
        .context("git commit failed")?;
    if !commit.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit.stderr).trim()
        );
    }
    Ok(())
}

// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/td.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
      Adds TD `--phase` section-queue apply support, WI projection lock updates,
      hook-compatible expected payload/command state, and deterministic
      section-level lifecycle trailers.
```
