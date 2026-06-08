---
id: sdd-tools-create-change-merge-workflow
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change merge workflow

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_merge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 69 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_merge.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_workflow"
    description: "Programmatic merge workflow for specs, archive, issue close, and git operation response fields."
```
