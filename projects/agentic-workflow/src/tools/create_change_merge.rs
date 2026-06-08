// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_merge/definition.md#source
// CODEGEN-BEGIN
//! Programmatic merge tool for change-merge.
//!
//! `sdd_workflow_create_change_merge` — single tool that:
//! 1. Reads all change specs from `changes/{id}/specs/`
//! 2. Extracts `main_spec_ref` from each spec's frontmatter
//! 3. Strips change-spec-only fields
//! 4. Writes cleaned specs to `.aw/tech-design/{main_spec_ref}`
//! 5. Updates phase to `ChangeArchived`
//!
//! No agent needed. No CRR loop. Single programmatic operation.

use crate::models::state::StatePhase;
use crate::models::SddConfig;
use crate::models::WorkflowArtifact;
use crate::tools::common_change_spec as common;
use crate::tools::merge_git_ops::{find_git_binary, post_archive_git_ops, resolve_worktree_dir};
use crate::tools::workflow_common;
use crate::tools::{get_required_string, ToolDefinition};
use crate::workflow::helpers;
use crate::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

// ─── Tool Definition ──────────────────────────────────────────────────────────

/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_merge/definition.md#source
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_change_merge".to_string(),
        description:
            "Programmatic merge: copy all change specs to .aw/tech-design/ and archive the change"
                .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID"
                }
            }
        }),
    }
}
// CODEGEN-END
// ─── Workflow (programmatic merge) ────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_merge/workflow.md#source
// CODEGEN-BEGIN
// ─── Workflow (programmatic merge) ────────────────────────────────────────────

/// Execute programmatic merge.
///
/// For each spec in `changes/{id}/specs/`:
/// 1. Read content and extract `main_spec_ref` from frontmatter
/// 2. If `main_spec_ref` is null, derive from spec_id + change scope
/// 3. Strip change-spec-only frontmatter fields
/// 4. Write to `.aw/tech-design/{main_spec_ref}`
/// 5. Update phase → ChangeArchived
/// 6. Return archive instructions
/// @spec projects/agentic-workflow/tech-design/core/tools/create_change_merge/workflow.md#source
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    // REQ: change-merge R9 — worktree-first path resolution.
    // work_root is the worktree when worktree-first mode is active, else
    // project_root (legacy / test fallback). All `.aw/` writes (specs,
    // archive, alignment) land on work_root so main only receives them via
    // `git merge cclab/<slug>` in step 3 of post_archive_git_ops.
    let work_root: PathBuf = resolve_worktree_dir(project_root, &change_id)
        .unwrap_or_else(|| project_root.to_path_buf());

    let change_dir = work_root.join(".aw/changes").join(&change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;

    // REQ: change-merge R6, R7, R8 — pre-flight gates. Abort before any
    // side effect if the worktree is dirty, the branch is missing, or no
    // source specs exist. Skipped in non-git contexts (tests).
    pre_flight_validate(&work_root, project_root, &change_id, &change_dir)?;

    let spec_paths = helpers::find_specs_to_merge(&change_dir);

    // Load validated config — fails early if platform sections missing
    let config = SddConfig::load_validated(project_root)?;
    let repo_platform = config.repo_platform;

    if spec_paths.is_empty() {
        // No specs to merge — just archive
        workflow_common::update_phase(&change_dir, StatePhase::ChangeArchived)?;
        let archive_rel = build_archive_path(&change_id);
        let archive_abs = work_root.join(&archive_rel);
        std::fs::create_dir_all(archive_abs.parent().unwrap_or(&archive_abs))?;
        std::fs::rename(&change_dir, &archive_abs)?;

        // REQ: worktree-per-change — close the issue in the worktree's
        // working copy BEFORE post_archive_git_ops so `git add .aw/` picks
        // up the open→closed rename and it lands in the branch commit.
        let issue_moved = close_issue_if_exists(&work_root, &change_id);

        // Post-archive git operations (no specs merged)
        // REQ: worktree-per-change — includes steps 3/4 when worktree exists
        let git_ops = post_archive_git_ops(
            project_root,
            &change_id,
            &archive_abs,
            repo_platform.as_ref(),
            &[],
        )?;

        return Ok(serde_json::to_string_pretty(&json!({
            "status": "ok",
            "message": "No specs to merge. Change archived.",
            "archive_path": archive_rel,
            "git_commit_sha": git_ops.git_commit_sha,
            "pr_url": git_ops.pr_url,
            "git_warning": git_ops.git_warning,
            "issue_closed": issue_moved,
            "next_actions": []
        }))?);
    }

    // REQ: change-merge R9 — specs promote inside the worktree, then reach
    // main via git merge in step 3. Writing to project_root would bypass the
    // branch.
    let specs_root = crate::shared::workspace::tech_design_path(&work_root);
    let mut merged = Vec::new();
    let mut audit_log: Vec<String> = Vec::new();

    // Detect git binary once for all specs
    let git_bin = find_git_binary();

    // Pre-validation + merge pass: validate all specs and compute merge results
    // before writing any files. A hard error or conflict aborts the merge.
    struct MergeResult {
        spec_id: String,
        target_rel: String,
        content: String,
        audit_action: String,
    }

    let mut merge_results: Vec<MergeResult> = Vec::new();
    let mut conflicts: Vec<String> = Vec::new();

    for spec_path in &spec_paths {
        let spec_id = spec_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        if !spec_path.exists() {
            anyhow::bail!("Spec file not found: {}", spec_path.display());
        }

        let content = std::fs::read_to_string(spec_path)?;

        // Validate main_spec_ref: must be present and contain a subfolder path.
        let target_rel = match common::read_main_spec_ref(&content) {
            Some(ref_path) => {
                if !ref_path.contains('/') {
                    anyhow::bail!("merge rejected root-level main_spec_ref: {}", ref_path);
                }
                ref_path
            }
            None => {
                anyhow::bail!(
                    "Spec '{}': missing main_spec_ref. All specs must specify a subfolder \
                     target path.",
                    spec_id
                );
            }
        };

        // Strip change-spec-only fields (fill_sections, filled_sections, etc.)
        let cleaned = common::strip_change_spec_fields(&content);

        let target_path = resolve_merge_target_path(&work_root, &specs_root, &target_rel);

        // Check for .base.md sibling file for 3-way merge
        let base_path = spec_path.with_file_name(format!("{}.base.md", spec_id));

        if base_path.exists() && target_path.exists() {
            // 3-way merge path
            if let Some(ref git) = git_bin {
                let ours_content = std::fs::read_to_string(&target_path)?;
                let base_content = std::fs::read_to_string(&base_path)?;

                match merge_3way(git, &ours_content, &base_content, &cleaned) {
                    Ok(merged_content) => {
                        merge_results.push(MergeResult {
                            spec_id,
                            target_rel,
                            content: merged_content,
                            audit_action: "3way-merge".to_string(),
                        });
                    }
                    Err(e) => {
                        conflicts.push(format!("{}: {}", spec_id, e));
                    }
                }
            } else {
                // git not available — fallback to overwrite with warning
                audit_log.push(format!(
                    "[merge] warning: git not found, falling back to overwrite for {}",
                    target_rel
                ));
                merge_results.push(MergeResult {
                    spec_id,
                    target_rel,
                    content: cleaned,
                    audit_action: "overwrite".to_string(),
                });
            }
        } else if target_path.exists() {
            // No .base.md but target exists — section-level merge.
            // Preserves existing sections in the target, updates/adds
            // sections from the change spec. This avoids destructive
            // overwrites that blow away pre-existing rich spec content.
            // REQ: bug-create-change-merge-archive-moves-not-committed-sp (defect 2)
            let existing = std::fs::read_to_string(&target_path)?;
            let merged_content = merge_sections_into_target(&existing, &cleaned);
            merge_results.push(MergeResult {
                spec_id,
                target_rel,
                content: merged_content,
                audit_action: "section-merge".to_string(),
            });
        } else {
            // Target doesn't exist — create new file
            merge_results.push(MergeResult {
                spec_id,
                target_rel,
                content: cleaned,
                audit_action: "create".to_string(),
            });
        }
    }

    // If any specs had conflicts, abort the entire merge — no files written
    if !conflicts.is_empty() {
        anyhow::bail!(
            "3-way merge conflicts detected — merge aborted, no files written:\n{}",
            conflicts.join("\n")
        );
    }

    // Write pass: all specs are valid and merged; write results to disk.
    let mut merged_target_paths: Vec<std::path::PathBuf> = Vec::new();
    for result in &merge_results {
        let target_path = resolve_merge_target_path(&work_root, &specs_root, &result.target_rel);
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        audit_log.push(format!(
            "[merge] {} {}",
            result.audit_action, result.target_rel
        ));

        std::fs::write(&target_path, &result.content)?;
        merged_target_paths.push(target_path);
        merged.push(json!({
            "spec_id": result.spec_id,
            "target": format!(".aw/tech-design/{}", result.target_rel),
        }));
    }

    // ── Post-write alignment checks (non-blocking warnings) ──────────────
    let (alignment_warnings, alignment_summary) = run_alignment_checks(&merged_target_paths);

    if !alignment_warnings.is_empty() {
        let summary_str = alignment_summary.as_deref().unwrap_or("violations found");
        audit_log.push(format!("[merge] alignment: {}", summary_str));
    }

    // Update phase → ChangeArchived
    workflow_common::update_phase(&change_dir, StatePhase::ChangeArchived)?;

    let archive_rel = build_archive_path(&change_id);
    let archive_abs = work_root.join(&archive_rel);

    let prompt = format!(
        "# Merge Complete for Change '{}'\n\n\
         {} spec(s) merged to main specs directory.\n\
         Change archived to {}.\n\n\
         SDD workflow complete!",
        change_id,
        merged.len(),
        archive_rel,
    );

    let alignment_warnings_json: Value = if alignment_warnings.is_empty() {
        Value::Null
    } else {
        json!(alignment_warnings
            .iter()
            .map(|w| json!({
                "file": &w.file,
                "kind": &w.kind,
                "message": &w.message,
                "heading": &w.heading,
                "line": w.line,
            }))
            .collect::<Vec<Value>>())
    };

    let extra_fields = json!({
        "specs_merged": merged,
        "archive_path": &archive_rel,
        "audit_log": audit_log,
        "alignment_warnings": alignment_warnings_json,
        "alignment_summary": alignment_summary,
    });

    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeMerge);

    // Build response before moving (build_workflow_response reads change_dir)
    let response = workflow_common::build_workflow_response(
        &change_dir,
        &change_id,
        "create_change_merge",
        prompt,
        executor,
        extra_fields,
        interface,
        project_root,
    )
    .await?;

    // Move change directory to archive
    std::fs::create_dir_all(archive_abs.parent().unwrap_or(&archive_abs))?;
    std::fs::rename(&change_dir, &archive_abs)?;

    // Append alignment warnings to implementation.md in archive (if any)
    if !alignment_warnings.is_empty() {
        append_alignment_to_impl(&archive_abs, &alignment_warnings);
    }

    // REQ: worktree-per-change — move the issue open/→closed/ in the
    // worktree's working copy BEFORE post_archive_git_ops. This ensures
    // `git add .aw/` stages the rename into the cclab/<slug> branch
    // commit, which then reaches main via the step-3 merge. Best-effort:
    // failure is logged but does not block merge completion.
    let issue_closed = close_issue_if_exists(&work_root, &change_id);

    // Post-archive git operations — includes steps 2-5 of the merge sequence
    // REQ: worktree-per-change — may return a hard error on step-3 merge conflict.
    let git_ops = post_archive_git_ops(
        project_root,
        &change_id,
        &archive_abs,
        repo_platform.as_ref(),
        &merged,
    )?;

    // Merge git ops results into the response JSON
    let mut response_value: Value = serde_json::from_str(&response)?;
    if let Some(obj) = response_value.as_object_mut() {
        obj.insert("git_commit_sha".to_string(), json!(git_ops.git_commit_sha));
        obj.insert("pr_url".to_string(), json!(git_ops.pr_url));
        obj.insert("git_warning".to_string(), json!(git_ops.git_warning));
        obj.insert("issue_closed".to_string(), json!(issue_closed));
    }

    Ok(serde_json::to_string_pretty(&response_value)?)
}

fn resolve_merge_target_path(work_root: &Path, specs_root: &Path, target_rel: &str) -> PathBuf {
    if let Some((project, rest)) = target_rel.split_once('/') {
        if let Ok(resolved) =
            crate::services::project_registry::resolve_td_root_from_config(work_root, project)
        {
            return PathBuf::from(resolved.root).join(rest);
        }
    }
    specs_root.join(target_rel)
}
// CODEGEN-END
// ─── Pre-flight Gates (R6, R7, R8) ───────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_merge/preflight.md#source
// CODEGEN-BEGIN
// ─── Pre-flight Gates (R6, R7, R8) ───────────────────────────────────────────

/// Run the two pre-flight gates before any side effect.
///
/// - G1 (dirty worktree): `git -C <work_root> status --porcelain` must be empty.
/// - G2 (branch exists): `refs/heads/cclab/<slug>` must exist (worktree mode only).
///
/// Empty `change_dir/specs/` is intentionally allowed — callers downstream
/// handle the "no-specs archive" case.
///
/// Skipped entirely (with warning log) when no git binary is available — this
/// keeps tempdir/unit tests working without a git repo.
///
/// REQ: change-merge R6, R7.
fn pre_flight_validate(
    work_root: &Path,
    project_root: &Path,
    change_id: &str,
    _change_dir: &Path,
) -> Result<()> {
    // G1 + G2 require git. Skip gracefully in non-git contexts.
    let git_bin = match find_git_binary() {
        Some(g) => g,
        None => {
            tracing::warn!(
                change_id = %change_id,
                "pre-flight G1/G2 skipped: git binary not found"
            );
            return Ok(());
        }
    };

    // G1: dirty worktree
    let status = std::process::Command::new(&git_bin)
        .args(["status", "--porcelain"])
        .current_dir(work_root)
        .output();
    match status {
        Ok(out) if out.status.success() => {
            if !out.stdout.is_empty() {
                anyhow::bail!(
                    "uncommitted work in {} — commit or stash before merging.\n  Run: cd {} && git status",
                    work_root.display(),
                    work_root.display()
                );
            }
        }
        Ok(out) => {
            // Non-zero git status (e.g. not a git repo) — skip with warning
            tracing::warn!(
                stderr = %String::from_utf8_lossy(&out.stderr),
                "pre-flight G1 skipped: git status failed"
            );
            return Ok(());
        }
        Err(e) => {
            tracing::warn!(error = %e, "pre-flight G1 skipped: git status errored");
            return Ok(());
        }
    }

    // G2: branch exists (only when worktree mode). In legacy mode
    // (work_root == project_root), there is no dedicated cclab/<slug> branch
    // to verify.
    if work_root != project_root {
        let branch = format!("cclab/{}", change_id);
        let ref_name = format!("refs/heads/{}", branch);
        let exists = std::process::Command::new(&git_bin)
            .args(["show-ref", "--verify", "--quiet", &ref_name])
            .current_dir(project_root)
            .status();
        match exists {
            Ok(s) if !s.success() => {
                anyhow::bail!(
                    "branch {} not found — re-run init_change or use legacy flow",
                    branch
                );
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(error = %e, "pre-flight G2 skipped: show-ref errored");
            }
        }
    }

    Ok(())
}
// CODEGEN-END
// ─── Issue Closing (part of step 5) ──────────────────────────────────────────

/// Move the open issue associated with `change_id` to the temp issue store's closed state,
/// updating `state: closed` in frontmatter.
///
/// Matching strategy (in order):
/// 1. **Slug match** — open issue file named `{change_id}.md`.
/// 2. **Frontmatter id match** — scan all open issues and find one whose `id`
///    field (UUID) equals `change_id`. This handles cases where the issue file
///    was created with a UUID-based name but references this change by UUID.
///
/// Worktree path / phase fields are cleared so the closed record accurately
/// reflects that the change is done.
///
/// Returns `true` if an issue was found and closed, `false` otherwise (legacy
/// changes without an associated issue).
// REQ: worktree-per-change — issue open/→closed/ move on merge
// @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R3
//
// `root` must be the worktree's working copy (or project_root when no
// worktree is active). The open→closed rename is written into that tree
// so the subsequent `git add .aw/` in post_archive_git_ops stages it
// into the branch commit.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_merge/issue-close.md#source
// CODEGEN-BEGIN
// ─── Issue Closing (part of step 5) ──────────────────────────────────────────

/// Move the open issue associated with `change_id` to the temp issue store's closed state,
/// updating `state: closed` in frontmatter.
///
/// Matching strategy (in order):
/// 1. **Slug match** — open issue file named `{change_id}.md`.
/// 2. **Frontmatter id match** — scan all open issues and find one whose `id`
///    field (UUID) equals `change_id`. This handles cases where the issue file
///    was created with a UUID-based name but references this change by UUID.
///
/// Worktree path / phase fields are cleared so the closed record accurately
/// reflects that the change is done.
///
/// Returns `true` if an issue was found and closed, `false` otherwise (legacy
/// changes without an associated issue).
// REQ: worktree-per-change — issue open/→closed/ move on merge
// @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R3
//
// `root` must be the worktree's working copy (or project_root when no
// worktree is active). The open→closed rename is written into that tree
// so the subsequent `git add .aw/` in post_archive_git_ops stages it
// into the branch commit.
fn close_issue_if_exists(root: &Path, change_id: &str) -> bool {
    let root_owned = root.to_path_buf();
    let change_id_owned = change_id.to_string();

    let result = crate::state::run_blocking_io(move || async move {
        use crate::issues::{local_backend, IssueBackend};
        let backend = local_backend(&root_owned);

        // ── Strategy 1: slug match ───────────────────────────────────────
        let issue_opt = backend.get(&change_id_owned).await?;

        let issue_to_close = if issue_opt.is_some() {
            issue_opt
        } else {
            // ── Strategy 2: frontmatter id match ─────────────────────────
            let all_issues = backend.list(&crate::issues::IssueFilter::default()).await?;
            all_issues
                .into_iter()
                .filter(|i| {
                    matches!(
                        i.state,
                        crate::issues::IssueState::Open | crate::issues::IssueState::Draft
                    )
                })
                .find(|i| i.id.as_deref() == Some(change_id_owned.as_str()))
        };

        let mut issue = match issue_to_close {
            Some(i) => i,
            None => return Ok(false),
        };

        // REQ: R7 — Merge writes state:closed, phase:change_archived to issue.
        // REQ: R8 — Clear transient fields, keep change_id/branch/phase.
        issue.state = crate::issues::IssueState::Closed;
        issue.phase = Some("change_archived".to_string());
        issue.git_workflow = None;
        issue.iteration = None;
        issue.current_task_id = None;
        issue.impl_spec_phase = None;
        issue.task_revisions = None;
        issue.revision_counts = None;
        issue.last_action = None;
        issue.session_id = None;
        issue.validation_errors = vec![];

        backend.write(&issue).await?;
        Ok(true)
    });

    match result {
        Ok(closed) => closed,
        Err(e) => {
            tracing::warn!(
                change_id = %change_id,
                error = %e,
                "close_issue_if_exists: failed to close issue on merge"
            );
            false
        }
    }
}
// CODEGEN-END
// ─── 3-Way Merge Support ─────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_merge/merge-helpers.md#source
// CODEGEN-BEGIN
// ─── 3-Way Merge Support ─────────────────────────────────────────────────────

/// Perform a 3-way merge using `git merge-file`.
///
/// Writes ours/base/theirs to temp files, invokes `git merge-file --stdout`,
/// and returns the merged content on clean merge (exit 0) or an error with
/// conflict details on exit >0.
fn merge_3way(
    git: &Path,
    ours: &str,
    base: &str,
    theirs: &str,
) -> std::result::Result<String, String> {
    let tmp_dir = tempfile::tempdir().map_err(|e| format!("failed to create tempdir: {}", e))?;

    let ours_path = tmp_dir.path().join("ours.md");
    let base_path = tmp_dir.path().join("base.md");
    let theirs_path = tmp_dir.path().join("theirs.md");

    std::fs::write(&ours_path, ours).map_err(|e| format!("write ours: {}", e))?;
    std::fs::write(&base_path, base).map_err(|e| format!("write base: {}", e))?;
    std::fs::write(&theirs_path, theirs).map_err(|e| format!("write theirs: {}", e))?;

    let output = std::process::Command::new(git)
        .arg("merge-file")
        .arg("--stdout")
        .arg(&ours_path)
        .arg(&base_path)
        .arg(&theirs_path)
        .output()
        .map_err(|e| format!("git merge-file failed to execute: {}", e))?;

    let merged = String::from_utf8_lossy(&output.stdout).to_string();

    if output.status.success() {
        Ok(merged)
    } else {
        // exit code >0 means conflicts
        let conflict_count = merged.matches("<<<<<<<").count();
        Err(format!(
            "merge conflict ({} marker{})",
            conflict_count,
            if conflict_count == 1 { "" } else { "s" }
        ))
    }
}

// ─── Section-Level Merge ────────────────────────────────────────────────────

/// Merge change-spec sections into an existing target spec, preserving
/// sections that the change doesn't touch.
///
/// Sections are delimited by `## Heading` lines. The frontmatter block
/// (between `---` fences) is always taken from the change spec (it contains
/// the updated metadata). Content before the first `## ` heading (the title
/// / preamble) is taken from the change spec if it is non-empty, otherwise
/// preserved from the target.
///
/// For each `## Heading` in the change spec:
/// - If the target has a section with the same heading, it is **replaced**.
/// - If the target does not have it, it is **appended**.
///
/// Sections in the target that are NOT present in the change spec are
/// **preserved** in their original order (inserted between the last
/// preceding change-spec section and the next one).
///
/// REQ: bug-create-change-merge-archive-moves-not-committed-sp (defect 2)
fn merge_sections_into_target(target: &str, change: &str) -> String {
    let target_sections = parse_markdown_sections(target);
    let change_sections = parse_markdown_sections(change);

    // Use the change spec's frontmatter (updated metadata).
    let mut result = String::new();
    result.push_str(&change_sections.frontmatter);

    // Use the change spec's preamble (title + intro before first ## section)
    // if non-empty; otherwise keep the target's preamble.
    let preamble = if change_sections.preamble.trim().is_empty() {
        &target_sections.preamble
    } else {
        &change_sections.preamble
    };
    result.push_str(preamble);

    // Build a set of headings present in the change spec for quick lookup.
    let change_headings: std::collections::HashSet<&str> = change_sections
        .sections
        .iter()
        .map(|s| s.heading.as_str())
        .collect();

    // Build a set of headings already emitted to avoid duplicates.
    let mut emitted: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Target heading -> content lookup for sections NOT in the change spec.
    let target_section_map: std::collections::HashMap<&str, &str> = target_sections
        .sections
        .iter()
        .map(|s| (s.heading.as_str(), s.body.as_str()))
        .collect();

    // Walk the change spec sections in order.
    // Before emitting each change section, emit any target-only sections
    // that appeared before this position in the target.
    let target_order: Vec<&str> = target_sections
        .sections
        .iter()
        .map(|s| s.heading.as_str())
        .collect();

    let mut target_cursor = 0; // next target section index to check

    for cs in &change_sections.sections {
        // Emit any target-only sections that come before this change section
        // in the target's ordering.
        if let Some(cs_target_idx) = target_order.iter().position(|h| *h == cs.heading.as_str()) {
            while target_cursor < cs_target_idx {
                let th = target_order[target_cursor];
                if !change_headings.contains(th) && !emitted.contains(th) {
                    if let Some(body) = target_section_map.get(th) {
                        result.push_str(&format!("## {}\n", th));
                        result.push_str(body);
                        emitted.insert(th.to_string());
                    }
                }
                target_cursor += 1;
            }
            target_cursor = cs_target_idx + 1;
        }

        // Emit the change section (replaces any existing target section).
        result.push_str(&format!("## {}\n", cs.heading));
        result.push_str(&cs.body);
        emitted.insert(cs.heading.clone());
    }

    // Emit remaining target-only sections that come after the last
    // change section in the target's ordering.
    for th in &target_order[target_cursor..] {
        if !change_headings.contains(th) && !emitted.contains(*th) {
            if let Some(body) = target_section_map.get(th) {
                result.push_str(&format!("## {}\n", *th));
                result.push_str(body);
                emitted.insert(th.to_string());
            }
        }
    }

    result
}

/// Parsed representation of a markdown spec file split into sections.
struct ParsedSections {
    /// The YAML frontmatter block including `---` delimiters + trailing newline.
    frontmatter: String,
    /// Content between end of frontmatter and the first `## ` heading.
    preamble: String,
    /// Ordered list of `## `-level sections.
    sections: Vec<MdSection>,
}

struct MdSection {
    /// The heading text (without the `## ` prefix).
    heading: String,
    /// Everything after the heading line up to (but not including) the next
    /// `## ` heading or end of file.
    body: String,
}

/// Split a markdown spec file into frontmatter, preamble, and `## `-delimited sections.
fn parse_markdown_sections(content: &str) -> ParsedSections {
    let mut frontmatter = String::new();
    let mut rest = content;

    // Extract frontmatter if present
    if content.starts_with("---\n") || content.starts_with("---\r\n") {
        if let Some(end_idx) = content[4..].find("\n---") {
            let fm_end = 4 + end_idx + 4; // skip past closing "---\n"
                                          // Advance past the newline after closing ---
            let fm_end = if content[fm_end..].starts_with('\n') {
                fm_end + 1
            } else {
                fm_end
            };
            frontmatter = content[..fm_end].to_string();
            rest = &content[fm_end..];
        }
    }

    let mut preamble = String::new();
    let mut sections: Vec<MdSection> = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_body = String::new();

    for line in rest.lines() {
        if line.starts_with("## ") && !line.starts_with("### ") {
            // New section boundary
            if let Some(heading) = current_heading.take() {
                sections.push(MdSection {
                    heading,
                    body: current_body.clone(),
                });
                current_body.clear();
            }
            let heading_text = line[3..].trim().to_string();
            current_heading = Some(heading_text);
        } else if current_heading.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        } else {
            preamble.push_str(line);
            preamble.push('\n');
        }
    }

    // Flush last section
    if let Some(heading) = current_heading {
        sections.push(MdSection {
            heading,
            body: current_body,
        });
    }

    ParsedSections {
        frontmatter,
        preamble,
        sections,
    }
}
// CODEGEN-END
// ─── Section-Level Merge ────────────────────────────────────────────────────

// ─── Alignment Check Helpers ─────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_merge/alignment-and-archive.md#source
// CODEGEN-BEGIN
// ─── Alignment Check Helpers ─────────────────────────────────────────────────

/// Alignment warning from post-merge spec check.
struct AlignmentWarning {
    file: String,
    kind: String,
    message: String,
    heading: Option<String>,
    line: Option<usize>,
}

/// Run alignment checks on merged spec target paths.
///
/// Returns `(warnings, summary)` where summary is `Some` if there are violations,
/// e.g. "3 violation(s) in 2 file(s)".
fn run_alignment_checks(
    target_paths: &[std::path::PathBuf],
) -> (Vec<AlignmentWarning>, Option<String>) {
    let mut warnings = Vec::new();
    let mut files_with_violations = 0_usize;

    for path in target_paths {
        let check_result = crate::spec_alignment::check(path);
        let mut has_violations = false;
        for file_result in &check_result.files {
            for violation in &file_result.violations {
                has_violations = true;
                warnings.push(AlignmentWarning {
                    file: file_result.path.clone(),
                    kind: violation.kind.to_string(),
                    message: violation.message.clone(),
                    heading: violation.heading.clone(),
                    line: violation.line,
                });
            }
        }
        if has_violations {
            files_with_violations += 1;
        }
    }

    let summary = if warnings.is_empty() {
        None
    } else {
        Some(format!(
            "{} violation(s) in {} file(s)",
            warnings.len(),
            files_with_violations
        ))
    };

    (warnings, summary)
}

/// Append alignment warnings table to `implementation.md` in the archive.
///
/// Creates the file if it doesn't exist; appends if it does.
fn append_alignment_to_impl(archive_path: &Path, warnings: &[AlignmentWarning]) {
    use std::io::Write;
    let impl_path = archive_path.join("implementation.md");
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&impl_path)
    {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(
                path = %impl_path.display(),
                error = %e,
                "failed to open implementation.md for alignment warnings"
            );
            return;
        }
    };

    let files_checked = {
        let mut seen = std::collections::HashSet::new();
        for w in warnings {
            seen.insert(&w.file);
        }
        seen.len()
    };
    let mut content = String::from("\n\n## Alignment Warnings\n\n");
    content.push_str(&format!(
        "{} violation(s) found across {} spec(s).\n\n",
        warnings.len(),
        files_checked
    ));
    content.push_str("| File | Kind | Message |\n|------|------|---------|");
    for w in warnings {
        content.push_str(&format!("\n| {} | {} | {} |", w.file, w.kind, w.message));
    }
    content.push('\n');

    if let Err(e) = file.write_all(content.as_bytes()) {
        tracing::warn!(
            path = %impl_path.display(),
            error = %e,
            "failed to write alignment warnings to implementation.md"
        );
    }
}

// ─── Archive ─────────────────────────────────────────────────────────────────

/// Build archive path for a change.
fn build_archive_path(change_id: &str) -> String {
    format!(
        ".aw/archive/{}-{}",
        chrono::Utc::now().format("%Y%m%d"),
        change_id
    )
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_merge/test-support.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod test_support {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

    pub(super) fn setup_change(change_id: &str, phase: StatePhase) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/tech-design")).unwrap();
        // R4: save() needs an issue backing change_id.
        crate::test_util::write_minimal_issue(tmp.path(), change_id);

        // Write minimal config.toml with required platform sections
        let config_content = r#"
[agentic_workflow.repo_platform]
type = "github"
repo = "test/repo"
default_branch = "main"
auto_commit = false
auto_pr = false

[agentic_workflow.tech_design_platform]
type = "local"
path = ".aw/tech-design"
"#;
        std::fs::write(tmp.path().join(".aw/config.toml"), config_content).unwrap();

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = phase;
        sm.save().unwrap();

        tmp
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_change_merge/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod workflow_tests {
    use super::test_support::setup_change;
    use super::*;

    #[tokio::test]
    async fn test_programmatic_merge_with_main_spec_ref() {
        let tmp = setup_change("pm-test", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-test");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // merge_strategy is dead code — no longer stripped, so omit from input
        let spec_content = "---\nid: auth-flow\nmain_spec_ref: sdd/workflow/auth-flow.md\ncreate_complete: true\nfill_sections: [overview]\nfilled_sections: [overview]\n---\n\n# Auth Flow\n\n## Overview\n\nAuth flow spec.\n\n# Reviews\n\n## Review: Test\nAll good.\n";
        std::fs::write(change_dir.join("specs/auth-flow.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["specs_merged"].as_array().unwrap().len(), 1);
        assert_eq!(
            parsed["specs_merged"][0]["target"],
            ".aw/tech-design/sdd/workflow/auth-flow.md"
        );

        // Verify target file exists and change-spec-only fields are stripped
        let target = tmp.path().join(".aw/tech-design/sdd/workflow/auth-flow.md");
        assert!(target.exists());
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(!content.contains("fill_sections"));
        assert!(!content.contains("filled_sections"));
        assert!(!content.contains("create_complete"));
        assert!(!content.contains("# Reviews"));
        assert!(content.contains("Auth flow spec."));

        // Verify change was moved to archive
        assert!(
            !change_dir.exists(),
            "change_dir should be moved to archive"
        );
        let archive_dir = tmp.path().join(parsed["archive_path"].as_str().unwrap());
        assert!(archive_dir.exists(), "archive dir should exist");

        // Archived phase lives in the closed issue (single source of truth under R4/R7).
        let closed_issue = crate::shared::workspace::issues_path(tmp.path())
            .join("closed")
            .join("pm-test.md");
        assert!(
            closed_issue.exists(),
            "closed issue must exist after archive"
        );
        let issue_body = std::fs::read_to_string(&closed_issue).unwrap();
        assert!(
            issue_body.contains("phase: change_archived"),
            "closed issue must record phase: change_archived:\n{}",
            issue_body
        );
    }

    #[tokio::test]
    async fn test_missing_main_spec_ref_rejected() {
        // Null main_spec_ref is now a hard error — no fallback to spec_id.md.
        let tmp = setup_change("pm-noref", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-noref");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let spec_content =
            "---\nid: new-feature\nmain_spec_ref: ~\n---\n\n# New Feature\n\n## Overview\n\nNew.\n";
        std::fs::write(change_dir.join("specs/new-feature.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-noref"
        });
        let result = execute_workflow(&args, tmp.path()).await;
        assert!(result.is_err(), "null main_spec_ref must be a hard error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("main_spec_ref"),
            "error must mention main_spec_ref: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_root_level_path_rejected() {
        // main_spec_ref without '/' is a hard error — merge aborted, no files written.
        let tmp = setup_change("pm-rootpath", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-rootpath");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let spec_content =
            "---\nid: flat-spec\nmain_spec_ref: flat-spec.md\n---\n\n# Flat\n\nContent.\n";
        std::fs::write(change_dir.join("specs/flat-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-rootpath"
        });
        let result = execute_workflow(&args, tmp.path()).await;
        assert!(result.is_err(), "root-level path must return a hard error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("root-level") || err.contains("subfolder"),
            "error must mention root-level restriction: {}",
            err
        );

        // No target file should have been written
        let flat_target = tmp.path().join(".aw/tech-design/flat-spec.md");
        assert!(
            !flat_target.exists(),
            "no target file should be written on hard error"
        );
    }

    #[tokio::test]
    async fn test_audit_log_create() {
        // When target does not exist, audit_log must contain "audit: create .aw/tech-design/..."
        let tmp = setup_change("pm-audit-create", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-audit-create");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        let spec_content = "---\nid: new-spec\nmain_spec_ref: sdd/logic/new-spec.md\n---\n\n# New Spec\n\nContent.\n";
        std::fs::write(change_dir.join("specs/new-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-audit-create"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let audit_log = parsed["audit_log"].as_array().unwrap();
        assert_eq!(audit_log.len(), 1);
        assert_eq!(
            audit_log[0].as_str().unwrap(),
            "[merge] create sdd/logic/new-spec.md"
        );
    }

    #[tokio::test]
    async fn test_audit_log_section_merge() {
        // When target already exists (no .base.md), audit_log must contain "[merge] section-merge {path}"
        // REQ: bug-create-change-merge-archive-moves-not-committed-sp (defect 2)
        let tmp = setup_change(
            "pm-audit-section-merge",
            StatePhase::ChangeImplementationReviewed,
        );
        let change_dir = tmp.path().join(".aw/changes/pm-audit-section-merge");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Pre-create the target file with existing sections
        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(
            target_dir.join("existing-spec.md"),
            "---\nid: existing-spec\nmain_spec_ref: sdd/logic/existing-spec.md\n---\n\n# Existing Spec\n\n## Overview\n\nOld overview.\n\n## Details\n\nOld details.\n",
        ).unwrap();

        // Change spec only touches Overview, not Details
        let spec_content = "---\nid: existing-spec\nmain_spec_ref: sdd/logic/existing-spec.md\n---\n\n# Existing Spec\n\n## Overview\n\nNew overview content.\n";
        std::fs::write(change_dir.join("specs/existing-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-audit-section-merge"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let audit_log = parsed["audit_log"].as_array().unwrap();
        assert!(
            !audit_log.is_empty(),
            "audit_log must have at least one entry"
        );
        assert_eq!(
            audit_log[0].as_str().unwrap(),
            "[merge] section-merge sdd/logic/existing-spec.md"
        );

        // New content must be present, AND old untouched sections must be preserved
        let content = std::fs::read_to_string(target_dir.join("existing-spec.md")).unwrap();
        assert!(
            content.contains("New overview content."),
            "updated section must be present"
        );
        assert!(
            content.contains("Old details."),
            "untouched section must be preserved"
        );
        assert!(
            !content.contains("Old overview."),
            "updated section must replace old content"
        );
    }

    #[tokio::test]
    async fn test_validation_aborts_before_write() {
        // When any spec fails path validation, NO files must be written (all-or-nothing).
        let tmp = setup_change("pm-abort", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-abort");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Valid spec — listed first alphabetically
        let valid_spec = "---\nid: valid-spec\nmain_spec_ref: sdd/valid/valid-spec.md\n---\n\n# Valid\n\nContent.\n";
        std::fs::write(change_dir.join("specs/a-valid-spec.md"), valid_spec).unwrap();

        // Invalid spec (root-level path, no '/') — listed second
        let invalid_spec =
            "---\nid: root-spec\nmain_spec_ref: root-spec.md\n---\n\n# Root\n\nContent.\n";
        std::fs::write(change_dir.join("specs/b-root-spec.md"), invalid_spec).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-abort"
        });
        let result = execute_workflow(&args, tmp.path()).await;
        assert!(
            result.is_err(),
            "merge must fail when any spec has invalid path"
        );

        // The valid spec's target must NOT have been written — validation aborts before write pass
        let valid_target = tmp.path().join(".aw/tech-design/sdd/valid/valid-spec.md");
        assert!(
            !valid_target.exists(),
            "no files should be written when validation aborts the merge"
        );
    }

    #[tokio::test]
    async fn test_3way_merge_clean() {
        // Setup: base snapshot + diverged main spec + change spec → clean 3-way merge
        // Changes are in non-overlapping regions so merge is clean.
        if find_git_binary().is_none() {
            // Skip: git is required for 3-way merge test
            return;
        }

        let tmp = setup_change("pm-3way-clean", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-3way-clean");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Base content (snapshot at change-init time) — has two sections
        let base_content = "\
    ---\nid: merge-test\nmain_spec_ref: sdd/logic/merge-test.md\n---\n\n\
    # Merge Test\n\n\
    ## Section One\n\n\
    Original section one.\n\n\
    ## Section Two\n\n\
    Original section two.\n";

        // Main spec on disk (diverged: modified Section One only)
        let main_content = "\
    ---\nid: merge-test\nmain_spec_ref: sdd/logic/merge-test.md\n---\n\n\
    # Merge Test\n\n\
    ## Section One\n\n\
    Main modified section one.\n\n\
    ## Section Two\n\n\
    Original section two.\n";

        // Change spec (modified Section Two only — non-overlapping with main's change)
        let change_content = "\
    ---\nid: merge-test\nmain_spec_ref: sdd/logic/merge-test.md\n\
    create_complete: true\nfill_sections: [overview]\nfilled_sections: [overview]\n---\n\n\
    # Merge Test\n\n\
    ## Section One\n\n\
    Original section one.\n\n\
    ## Section Two\n\n\
    Change updated section two.\n";

        // Write base snapshot
        std::fs::write(change_dir.join("specs/merge-test.base.md"), base_content).unwrap();

        // Write change spec
        std::fs::write(change_dir.join("specs/merge-test.md"), change_content).unwrap();

        // Write diverged main spec
        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("merge-test.md"), main_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-3way-clean"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Verify the merged file contains both non-overlapping changes:
        // main's modification to section one AND change spec's update to section two
        let merged = std::fs::read_to_string(target_dir.join("merge-test.md")).unwrap();
        assert!(
            merged.contains("Main modified section one."),
            "merged result must preserve main's diverged section one"
        );
        assert!(
            merged.contains("Change updated section two."),
            "merged result must include change spec's updated section two"
        );
        // Neither original line should remain
        assert!(
            !merged.contains("Original section one."),
            "original section one must be replaced by main's change"
        );
        assert!(
            !merged.contains("Original section two."),
            "original section two must be replaced by change spec's update"
        );
    }

    #[tokio::test]
    async fn test_3way_merge_conflict() {
        // Setup: both main and change spec modify the same line → conflict
        if find_git_binary().is_none() {
            // Skip: git is required for 3-way merge conflict test
            return;
        }

        let tmp = setup_change("pm-3way-conflict", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-3way-conflict");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Base content
        let base_content = "---\nid: conflict-test\nmain_spec_ref: sdd/logic/conflict-test.md\n---\n\n# Conflict Test\n\n## Overview\n\nOriginal line here.\n";

        // Main spec: same line changed to something different
        let main_content = "---\nid: conflict-test\nmain_spec_ref: sdd/logic/conflict-test.md\n---\n\n# Conflict Test\n\n## Overview\n\nMain changed this line to version A.\n";

        // Change spec: same line changed to something else
        let change_content = "---\nid: conflict-test\nmain_spec_ref: sdd/logic/conflict-test.md\n---\n\n# Conflict Test\n\n## Overview\n\nChange modified this line to version B.\n";

        std::fs::write(change_dir.join("specs/conflict-test.base.md"), base_content).unwrap();
        std::fs::write(change_dir.join("specs/conflict-test.md"), change_content).unwrap();

        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("conflict-test.md"), main_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-3way-conflict"
        });
        let result = execute_workflow(&args, tmp.path()).await;
        assert!(
            result.is_err(),
            "3-way merge with conflicts must return an error"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("conflict"),
            "error must mention conflict: {}",
            err
        );

        // Verify the main spec was NOT overwritten (all-or-nothing)
        let content = std::fs::read_to_string(target_dir.join("conflict-test.md")).unwrap();
        assert!(
            content.contains("Main changed this line to version A."),
            "main spec must remain unchanged after conflict abort"
        );
    }

    #[tokio::test]
    async fn test_base_md_skipped_by_find_specs() {
        // Verify that .base.md files are not included in find_specs_to_merge() results
        let tmp = setup_change("pm-skip-base", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-skip-base");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        // Write a regular spec and its .base.md companion
        let spec_content =
            "---\nid: some-spec\nmain_spec_ref: sdd/logic/some-spec.md\n---\n\n# Some Spec\n";
        let base_content = "---\nid: some-spec\nmain_spec_ref: sdd/logic/some-spec.md\n---\n\n# Some Spec (base)\n";
        std::fs::write(specs_dir.join("some-spec.md"), spec_content).unwrap();
        std::fs::write(specs_dir.join("some-spec.base.md"), base_content).unwrap();

        let found = helpers::find_specs_to_merge(&change_dir);
        assert_eq!(found.len(), 1, "only the regular spec should be found");
        let found_name = found[0].file_name().unwrap().to_str().unwrap();
        assert_eq!(
            found_name, "some-spec.md",
            "found file must be the regular spec"
        );
        assert!(
            !found
                .iter()
                .any(|p| p.to_str().unwrap().contains(".base.md")),
            ".base.md files must not appear in find_specs_to_merge results"
        );
    }

    #[tokio::test]
    async fn test_no_base_fallback_section_merge() {
        // Verify specs without .base.md use section-merge behavior when target exists
        // REQ: bug-create-change-merge-archive-moves-not-committed-sp (defect 2)
        let tmp = setup_change("pm-no-base", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-no-base");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Pre-create target file with existing sections
        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(
            target_dir.join("no-base-spec.md"),
            "---\nid: no-base-spec\nmain_spec_ref: sdd/logic/no-base-spec.md\n---\n\n# No Base Spec\n\n## Overview\n\nOriginal overview.\n\n## Details\n\nOriginal details.\n\n## History\n\nOriginal history.\n",
        ).unwrap();

        // Change spec without .base.md companion, modifies Overview and adds a new section
        let spec_content = "---\nid: no-base-spec\nmain_spec_ref: sdd/logic/no-base-spec.md\n---\n\n# No Base Spec\n\n## Overview\n\nNew content via section-merge.\n\n## New Section\n\nBrand new section.\n";
        std::fs::write(change_dir.join("specs/no-base-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-no-base"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Audit log must say "section-merge", not "overwrite" or "3way-merge"
        let audit_log = parsed["audit_log"].as_array().unwrap();
        assert!(
            !audit_log.is_empty(),
            "audit_log must have at least one entry"
        );
        let entry = audit_log[0].as_str().unwrap();
        assert!(
            entry.contains("section-merge"),
            "audit must record section-merge action when no .base.md and target exists: {}",
            entry
        );

        // Verify section-level merge: changed section updated, untouched sections preserved, new section added
        let content = std::fs::read_to_string(target_dir.join("no-base-spec.md")).unwrap();
        assert!(
            content.contains("New content via section-merge."),
            "updated section must contain new content"
        );
        assert!(
            !content.contains("Original overview."),
            "updated section must not contain old content"
        );
        assert!(
            content.contains("Original details."),
            "untouched Details section must be preserved"
        );
        assert!(
            content.contains("Original history."),
            "untouched History section must be preserved"
        );
        assert!(
            content.contains("Brand new section."),
            "new section must be appended"
        );
    }

    #[tokio::test]
    async fn test_audit_log_3way_merge() {
        // Verify audit log records '3way-merge' action for successful 3-way merges
        if find_git_binary().is_none() {
            // Skip: git is required for 3-way merge audit test
            return;
        }

        let tmp = setup_change("pm-audit-3way", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-audit-3way");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Use identical content for base and main (trivial merge — theirs wins cleanly)
        let base_and_main = "---\nid: audit-3way\nmain_spec_ref: sdd/logic/audit-3way.md\n---\n\n# Audit 3Way\n\n## Overview\n\nOriginal.\n";
        let change_content = "---\nid: audit-3way\nmain_spec_ref: sdd/logic/audit-3way.md\ncreate_complete: true\nfill_sections: [overview]\nfilled_sections: [overview]\n---\n\n# Audit 3Way\n\n## Overview\n\nUpdated by change.\n";

        std::fs::write(change_dir.join("specs/audit-3way.base.md"), base_and_main).unwrap();
        std::fs::write(change_dir.join("specs/audit-3way.md"), change_content).unwrap();

        let target_dir = tmp.path().join(".aw/tech-design/sdd/logic");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("audit-3way.md"), base_and_main).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-audit-3way"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let audit_log = parsed["audit_log"].as_array().unwrap();
        assert!(
            !audit_log.is_empty(),
            "audit log must have entries for 3-way merge"
        );
        let entry = audit_log[0].as_str().unwrap();
        assert_eq!(
            entry, "[merge] 3way-merge sdd/logic/audit-3way.md",
            "audit log must record 3way-merge action"
        );
    }

    // REQ: worktree-per-change — merge moves the associated issue to closed/
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_merge_closes_open_issue() {
        let tmp = setup_change(
            "enhancement-merge-closes",
            StatePhase::ChangeImplementationReviewed,
        );
        let change_dir = tmp.path().join(".aw/changes/enhancement-merge-closes");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();

        // Create a matching open issue
        let open_dir = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&open_dir).unwrap();
        std::fs::write(
            open_dir.join("enhancement-merge-closes.md"),
            "---\ntype: enhancement\ntitle: Test merge closes issue\nstate: open\nphase: change_inited\nbranch: cclab/enhancement-merge-closes\ngit_workflow: worktree\n---\n\n## Problem\n\nBody.\n",
        )
        .unwrap();

        // Minimal valid spec
        let spec_content = "---\nid: some-spec\nmain_spec_ref: sdd/logic/some-spec.md\n---\n\n# Some Spec\n\nContent.\n";
        std::fs::write(change_dir.join("specs/some-spec.md"), spec_content).unwrap();

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "enhancement-merge-closes"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(
            parsed["issue_closed"], true,
            "issue_closed flag should be set"
        );

        // File moved from open/ to closed/
        assert!(
            !open_dir.join("enhancement-merge-closes.md").exists(),
            "open issue file should be moved"
        );
        let closed_path = crate::shared::workspace::issues_path(tmp.path())
            .join("closed")
            .join("enhancement-merge-closes.md");
        assert!(closed_path.exists(), "closed issue file should exist");

        // REQ: R7 — state: closed, phase: change_archived, branch preserved
        let content = std::fs::read_to_string(&closed_path).unwrap();
        assert!(content.contains("state: closed"));
        assert!(
            content.contains("phase: change_archived"),
            "closed issue should have phase: change_archived:\n{}",
            content
        );
        assert!(
            content.contains("branch:"),
            "closed issue should retain branch field for audit trail:\n{}",
            content
        );
    }

    // Obsolete under R1: test_merge_without_issue_returns_false exercised the
    // "change has no backing issue" scenario. R1 of
    // refactor-eliminate-state-yaml-user-input-md-groups-nesting enforces
    // `change_id == issue_slug`, making this state unreachable — save()
    // would fail long before merge. Kept as a marker so future contributors
    // don't re-introduce the fallback.

    #[tokio::test]
    async fn test_programmatic_merge_no_specs() {
        let tmp = setup_change("pm-empty", StatePhase::ChangeImplementationReviewed);
        let change_dir = tmp.path().join(".aw/changes/pm-empty");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // No spec files

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "pm-empty"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["message"]
            .as_str()
            .unwrap()
            .contains("No specs to merge"));
        // Verify change was moved to archive
        assert!(!change_dir.exists());
    }
}

#[cfg(test)]
mod preflight_worktree_tests {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

    // REQ: change-merge R6 — pre-flight G1 aborts on dirty worktree.
    // Creates a real git repo with a worktree, dirties it with an uncommitted
    // file, and verifies execute_workflow aborts with actionable error.
    #[tokio::test]
    async fn test_preflight_g1_dirty_worktree_aborts() {
        let tmp = TempDir::new().unwrap();
        let slug = "g1-dirty-abort";

        // Need a real git binary to exercise G1
        let Some(git) = find_git_binary() else { return };

        // Init a bare main repo
        let main = tmp.path();
        let _ = std::process::Command::new(&git)
            .args(["init", "-q", "-b", "main"])
            .current_dir(main)
            .status();
        std::fs::write(main.join("seed.txt"), "seed\n").unwrap();
        let _ = std::process::Command::new(&git)
            .args(["add", "."])
            .current_dir(main)
            .status();
        let _ = std::process::Command::new(&git)
            .args([
                "-c",
                "user.email=t@t",
                "-c",
                "user.name=t",
                "commit",
                "-q",
                "-m",
                "seed",
            ])
            .current_dir(main)
            .status();

        // Create a worktree on branch cclab/<slug>
        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(main.join(".aw/worktrees")).unwrap();
        let add_out = std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(main)
            .output()
            .unwrap();
        if !add_out.status.success() {
            return; // git worktree add not supported in this test env
        }

        // Set up a valid change inside the worktree
        let wt_root = main.join(&wt_rel);
        let change_dir = wt_root.join(".aw/changes").join(slug);
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::create_dir_all(wt_root.join(".aw/tech-design")).unwrap();

        // config.toml on main (resolve_project_root reads from project_root)
        let config_content = r#"
    [agentic_workflow.repo_platform]
    type = "github"
    repo = "test/repo"
    default_branch = "main"
    auto_commit = false
    auto_pr = false

    [agentic_workflow.tech_design_platform]
    type = "local"
    path = ".aw/tech-design"
    "#;
        std::fs::write(main.join(".aw/config.toml"), config_content).unwrap();

        // Issue backs the change inside the worktree (R4: save() needs it).
        crate::test_util::write_minimal_issue(&wt_root, slug);

        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeImplementationReviewed;
        sm.save().unwrap();

        let spec_content = "---\nid: s1\nmain_spec_ref: sdd/logic/s1.md\n---\n\n# S1\n\n";
        std::fs::write(change_dir.join("specs/s1.md"), spec_content).unwrap();

        // DIRTY: write an uncommitted file in the worktree root
        std::fs::write(wt_root.join("dirty.txt"), "uncommitted\n").unwrap();

        let args = json!({
            "project_path": main.to_str().unwrap(),
            "change_id": slug
        });
        let result = execute_workflow(&args, main).await;

        assert!(result.is_err(), "G1 should abort on dirty worktree");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("uncommitted work"),
            "error should mention uncommitted work: {}",
            err_msg
        );
        // Repo state unchanged: change_dir still exists, nothing archived
        assert!(
            change_dir.exists(),
            "change_dir must still exist after G1 abort"
        );
        assert!(
            !wt_root.join(".aw/archive").exists(),
            "no archive should be created when G1 aborts"
        );
    }

    // REQ: change-merge R9 — worktree-first path resolution.
    // Verifies that when a worktree exists at .aw/worktrees/<slug>/,
    // specs promote INSIDE the worktree (not on main), archive lands inside
    // the worktree, and main's .aw/tech-design/ is NOT touched by
    // execute_workflow itself (git merge would bring it later).
    #[tokio::test]
    async fn test_programmatic_merge_uses_worktree_work_root() {
        let tmp = TempDir::new().unwrap();
        let slug = "wt-first-merge";

        // Layout: main + config + tech_design on main; the actual change
        // lives inside .aw/worktrees/<slug>/... (simulated without
        // needing a real git worktree — resolve_worktree_dir just checks
        // that the directory exists).
        let main_root = tmp.path();
        let wt_root = main_root.join(".aw/worktrees").join(slug);
        let change_dir = wt_root.join(".aw/changes").join(slug);
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::create_dir_all(wt_root.join(".aw/tech-design")).unwrap();
        std::fs::create_dir_all(main_root.join(".aw/tech-design")).unwrap();

        // Config lives on main
        let config_content = r#"
    [agentic_workflow.repo_platform]
    type = "github"
    repo = "test/repo"
    default_branch = "main"
    auto_commit = false
    auto_pr = false

    [agentic_workflow.tech_design_platform]
    type = "local"
    path = ".aw/tech-design"
    "#;
        std::fs::write(main_root.join(".aw/config.toml"), config_content).unwrap();

        // Issue backs the change inside the worktree (R4: save() needs it).
        crate::test_util::write_minimal_issue(&wt_root, slug);

        // State lives inside the worktree
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeImplementationReviewed;
        sm.save().unwrap();

        // A minimal spec to promote
        let spec_content =
            "---\nid: wt-spec\nmain_spec_ref: sdd/logic/wt-spec.md\n---\n\n# WT Spec\n\nContent.\n";
        std::fs::write(change_dir.join("specs/wt-spec.md"), spec_content).unwrap();

        // Run with project_root = main_root (the normal CLI case)
        let args = json!({
            "project_path": main_root.to_str().unwrap(),
            "change_id": slug
        });
        let result = execute_workflow(&args, main_root).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Verify: spec landed in worktree's tech_design, NOT on main
        let wt_target = wt_root.join(".aw/tech-design/sdd/logic/wt-spec.md");
        let main_target = main_root.join(".aw/tech-design/sdd/logic/wt-spec.md");
        assert!(
            wt_target.exists(),
            "spec should be promoted inside the worktree at {}",
            wt_target.display()
        );
        assert!(
            !main_target.exists(),
            "spec must NOT be written to main/.aw/tech-design/ — git merge \
             is the only path for main to receive it"
        );

        // Verify: archive landed inside worktree
        assert!(
            !change_dir.exists(),
            "change_dir should be moved to archive"
        );
        let wt_archive_parent = wt_root.join(".aw/archive");
        assert!(
            wt_archive_parent.exists(),
            "archive dir should be inside the worktree"
        );
        let main_archive_parent = main_root.join(".aw/archive");
        assert!(
            !main_archive_parent.exists(),
            "archive must NOT be written to main"
        );
    }
}

#[cfg(test)]
mod section_merge_tests {
    use super::*;

    // ─── Section-Level Merge Unit Tests ──────────────────────────────────

    #[test]
    fn test_parse_markdown_sections_basic() {
        let content = "---\nid: test\nmain_spec_ref: sdd/logic/test.md\n---\n\n# My Spec\n\n## Overview\n\nOverview text.\n\n## Details\n\nDetails text.\n";
        let parsed = parse_markdown_sections(content);
        assert!(parsed.frontmatter.contains("id: test"));
        assert!(parsed.preamble.contains("# My Spec"));
        assert_eq!(parsed.sections.len(), 2);
        assert_eq!(parsed.sections[0].heading, "Overview");
        assert!(parsed.sections[0].body.contains("Overview text."));
        assert_eq!(parsed.sections[1].heading, "Details");
        assert!(parsed.sections[1].body.contains("Details text."));
    }

    #[test]
    fn test_parse_markdown_sections_no_frontmatter() {
        let content = "# Title\n\n## Section A\n\nContent A.\n";
        let parsed = parse_markdown_sections(content);
        assert!(parsed.frontmatter.is_empty());
        assert!(parsed.preamble.contains("# Title"));
        assert_eq!(parsed.sections.len(), 1);
        assert_eq!(parsed.sections[0].heading, "Section A");
    }

    #[test]
    fn test_merge_sections_preserves_untouched() {
        let target = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## A\n\nA content.\n\n## B\n\nB content.\n\n## C\n\nC content.\n";
        let change = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## B\n\nB updated.\n";
        let result = merge_sections_into_target(target, change);
        assert!(
            result.contains("A content."),
            "untouched section A must be preserved"
        );
        assert!(
            result.contains("B updated."),
            "changed section B must be updated"
        );
        assert!(
            !result.contains("B content."),
            "old section B must be replaced"
        );
        assert!(
            result.contains("C content."),
            "untouched section C must be preserved"
        );
    }

    #[test]
    fn test_merge_sections_adds_new_section() {
        let target = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## A\n\nA content.\n";
        let change = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## A\n\nA content.\n\n## B\n\nNew section B.\n";
        let result = merge_sections_into_target(target, change);
        assert!(
            result.contains("A content."),
            "existing section A preserved"
        );
        assert!(
            result.contains("New section B."),
            "new section B must be added"
        );
    }

    #[test]
    fn test_merge_sections_new_file_creates_all() {
        // When target is empty, all change sections appear
        let target = "";
        let change =
            "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n## A\n\nA.\n\n## B\n\nB.\n";
        let result = merge_sections_into_target(target, change);
        assert!(result.contains("## A"));
        assert!(result.contains("## B"));
    }

    /// Regression test: R2.5 — merge a change-spec that adds one section
    /// to a target file with 5 pre-existing sections. All 6 must be present.
    #[test]
    fn test_merge_sections_five_existing_plus_one_new() {
        let target = "\
    ---\nid: rich\nmain_spec_ref: x/rich.md\n---\n\n# Rich Spec\n\n\
    ## Section One\n\nOne.\n\n\
    ## Section Two\n\nTwo.\n\n\
    ## Section Three\n\nThree.\n\n\
    ## Section Four\n\nFour.\n\n\
    ## Section Five\n\nFive.\n";

        let change = "\
    ---\nid: rich\nmain_spec_ref: x/rich.md\n---\n\n# Rich Spec\n\n\
    ## Section Six\n\nSix added by change.\n";

        let result = merge_sections_into_target(target, change);
        for (i, label) in ["One", "Two", "Three", "Four", "Five"].iter().enumerate() {
            assert!(
                result.contains(&format!("## Section {}", label)),
                "section {} ({}) must be present in merged output",
                i + 1,
                label
            );
        }
        assert!(
            result.contains("## Section Six"),
            "new section Six must be added"
        );
        assert!(
            result.contains("Six added by change."),
            "new section Six body must be present"
        );
    }

    /// Section ordering: target sections that appear between change sections
    /// must be preserved in their original relative order.
    #[test]
    fn test_merge_sections_preserves_order() {
        let target = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n\
    ## A\n\nA.\n\n\
    ## B\n\nB.\n\n\
    ## C\n\nC.\n\n\
    ## D\n\nD.\n";

        // Change modifies A and D, leaving B and C as target-only
        let change = "---\nid: t\nmain_spec_ref: x/y.md\n---\n\n# Spec\n\n\
    ## A\n\nA updated.\n\n\
    ## D\n\nD updated.\n";

        let result = merge_sections_into_target(target, change);

        // All four sections must be present
        assert!(result.contains("A updated."));
        assert!(result.contains("B."));
        assert!(result.contains("C."));
        assert!(result.contains("D updated."));

        // Order must be A, B, C, D
        let a_pos = result.find("## A").unwrap();
        let b_pos = result.find("## B").unwrap();
        let c_pos = result.find("## C").unwrap();
        let d_pos = result.find("## D").unwrap();
        assert!(a_pos < b_pos, "A must come before B");
        assert!(b_pos < c_pos, "B must come before C");
        assert!(c_pos < d_pos, "C must come before D");
    }
}
// CODEGEN-END
