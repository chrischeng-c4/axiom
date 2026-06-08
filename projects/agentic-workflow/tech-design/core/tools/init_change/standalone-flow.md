---
id: sdd-init-change-standalone-flow-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: capability-to-epic-planning
    claim: capability-to-epic-planning
    coverage: full
    rationale: "Issue initialization and reference-context tools support work-item planning and projection into bounded changes."
---

# Init Change Standalone Flow

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/init_change.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `cleanup_stale_worktree` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 360 | cleanup_stale_worktree(project_root: &Path, slug: &str) -> Result<()> |
| `create_worktree` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 328 | create_worktree(project_root: &Path, slug: &str) -> Result<PathBuf> |
| `definition` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 25 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 561 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_standalone` | projects/agentic-workflow/src/tools/init_change.rs | function | pub | 74 | execute_standalone(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
/// @spec projects/agentic-workflow/tech-design/core/logic/structured-issue.md#requirements
/// @spec projects/agentic-workflow/tech-design/core/logic/state-machine.md#requirements
pub fn execute_standalone(args: &Value, project_root: &Path) -> Result<String> {
    let raw_change_id = super::get_required_string(args, "change_id")?;
    let description = super::get_required_string(args, "description")?;

    // REQ: issue-centric-workflow#U1/U2 — expand UUID-prefix change_id to full slug
    // before any further processing. Ambiguous prefix errors here with candidate list.
    // Only expand if the raw value doesn't already match an on-disk slug (slug wins).
    let change_id = if issue_parser::looks_like_uuid_prefix(&raw_change_id)
        && issue_parser::load_issue_body(project_root, &raw_change_id).is_none()
    {
        match issue_parser::find_slug_by_uuid_prefix(project_root, &raw_change_id)? {
            Some(slug) => slug,
            None => raw_change_id.clone(),
        }
    } else {
        raw_change_id.clone()
    };

    // Validate change_id format
    super::workflow_common::validate_change_id(&change_id)?;

    let interface = super::workflow_common::load_interface(project_root);
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);

    // Error if change directory already exists
    if change_dir.exists() {
        anyhow::bail!(
            "Change '{}' already exists. Use sdd_run_change to continue.",
            change_id
        );
    }

    // NOTE: `git_workflow` and `branch` are deprecated inputs. Worktree isolation
    // is now always on: each change is created on a dedicated branch `cclab/<slug>`
    // in a worktree at `.aw/worktrees/<slug>`. Log a warning if callers still
    // pass these fields, but do not act on them.
    // REQ: worktree-per-change — init_change always creates a worktree
    let deprecated_branch = super::get_optional_string(args, "branch");
    let deprecated_git_workflow = super::get_optional_string(args, "git_workflow");
    if deprecated_branch.is_some() {
        tracing::warn!(
            change_id = %change_id,
            branch = ?deprecated_branch,
            "init_change: `branch` parameter is deprecated — worktree branch is always cclab/<slug>"
        );
    }
    if deprecated_git_workflow.is_some() {
        tracing::warn!(
            change_id = %change_id,
            git_workflow = ?deprecated_git_workflow,
            "init_change: `git_workflow` parameter is deprecated — worktree is always created"
        );
    }

    // Worktree branch is derived deterministically from change_id (= issue slug).
    let worktree_branch = format!("cclab/{}", change_id);
    let worktree_relpath = format!(".aw/worktrees/{}", change_id);

    // Branch uniqueness check — skip if worktree is expected and this change owns it.
    // We still keep the check for legacy callers that pass an explicit branch.
    if let Some(ref branch_name) = deprecated_branch {
        check_branch_uniqueness(project_root, branch_name, &change_id)?;
    }

    let issue_refs: Option<Vec<String>> =
        args.get("issues").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

    // ── Pre-flight validation (NO side effects before these pass) ──────────
    //
    // All three checks must pass before creating any files or worktrees.
    // This prevents orphaned change dirs / worktrees on validation failure.

    // REQ: issue-centric-workflow#R1 — resolve issue slug
    let resolved_slug =
        issue_parser::resolve_issue_slug(project_root, &description, issue_refs.as_deref());
    let issue_slug = match &resolved_slug {
        Some(s) => s.clone(),
        None => anyhow::bail!(
            "No issue found for this change. Provide --issue \"#<num>\" or \
             include issue:<slug> in the description. \
             See: issue-centric-workflow.md R1."
        ),
    };

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R1
    // R1: init_change rejects requests where resolved issue_slug differs from change_id.
    // One issue = one change: the on-disk change_id MUST equal the issue slug.
    if change_id != issue_slug {
        anyhow::bail!(
            "change_id '{}' does not match resolved issue slug '{}'. \
             change_id must equal the issue slug (one issue = one change). \
             Re-run with change_id='{}' or fix the issue reference.",
            change_id,
            issue_slug,
            issue_slug
        );
    }

    // REQ: structured-issue#R2 — hard gate, must be structured
    let issue_body = issue_parser::load_issue_body(project_root, &issue_slug);
    match &issue_body {
        Some(body) if issue_parser::is_structured_issue(body) => { /* OK */ }
        Some(_) => anyhow::bail!(
            "Issue '{}' is not structured (missing ## Problem, ## Requirements, or ## Scope). \
             Run `/aw:issue update {}` to prepare the issue before starting SDD. \
             See: structured-issue.md R2, issue-centric-workflow.md R7.",
            issue_slug,
            issue_slug
        ),
        None => anyhow::bail!(
            "Issue '{}' found by slug but body could not be loaded.",
            issue_slug
        ),
    }

    // Issue working copies are temp-backed. There is no checkout-hosted issue
    // file to require in git before lifecycle side effects.

    // REQ: issue-centric-workflow#R9 — filesystem-only 1:1:1 enforcement.
    // If a worktree already exists for this slug, a change is already in
    // progress. This replaces the old async backend query (flaky in tests).
    let wt_path = project_root.join(".aw/worktrees").join(&issue_slug);
    if wt_path.exists() {
        anyhow::bail!(
            "error: change '{}' already in progress ({} exists). \
             Complete or abandon it before starting a new one.",
            issue_slug,
            wt_path.display()
        );
    }

    // state:open gate — promote required before SDD can start. We still use
    // the backend here because it's the only thing that resolves state from
    // frontmatter, and we need state/phase on a per-issue basis.
    {
        let backend = local_backend(project_root);
        let issue_opt = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(backend.get(&issue_slug)))
                .ok()
                .flatten()
        } else if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(backend.get(&issue_slug)).ok().flatten()
        } else {
            None
        };
        if let Some(issue) = &issue_opt {
            // REQ: R5 — init_change requires state:open
            if issue.state == crate::issues::types::IssueState::Draft {
                anyhow::bail!(
                    "Issue '{}' is still in draft state. Run `aw wi validate {}` \
                     to review and promote to open before starting SDD.",
                    issue_slug,
                    issue_slug
                );
            }
        }
    }

    // ── Side effects start here (all validations passed) ─────────────────

    // 1. Create worktree FIRST — change artifacts will live inside it.
    let worktree_created_path: Option<PathBuf> = match create_worktree(project_root, &change_id) {
        Ok(p) => Some(p),
        Err(e) => {
            tracing::warn!(
                change_id = %change_id,
                error = %e,
                "init_change: failed to create git worktree — continuing without isolation"
            );
            None
        }
    };

    // 2. Change dir lives in the worktree (not on main).
    //    Falls back to project_root if worktree creation failed (tests, non-git).
    let change_root = worktree_created_path.as_deref().unwrap_or(project_root);
    let has_issues = issue_refs.as_ref().is_some_and(|r| !r.is_empty());

    create_change_internal(
        change_root,
        &change_id,
        &description,
        issue_refs.as_deref(),
        Some("worktree"),
        Some(&worktree_branch),
    )?;

    let change_dir = change_root.join(".aw/changes").join(&change_id);
    {
        let mut sm = StateManager::load(&change_dir)?;
        // Phase stays at ChangeInited (default) — route() handles it
        sm.state_mut().branch = Some(worktree_branch.clone());
        sm.state_mut().git_workflow = Some("worktree".to_string());
        sm.state_mut().change_id = change_id.clone();
        // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R4
        // R4: save() bubbles sync_to_issue() errors up — no STATE.yaml fallback.
        sm.save()?; // Writes workflow fields to issue frontmatter + operational data to meta.yaml
    }

    let mut written = Vec::<String>::new();
    if has_issues {
        written.push("issues/".to_string());
    }

    let next_tool = "sdd_run_change";

    let artifacts_written = json!(written);

    let worktree_path_json = if worktree_created_path.is_some() {
        json!(worktree_relpath)
    } else {
        Value::Null
    };

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "worktree_path": worktree_path_json,
        "worktree_branch": worktree_branch,
        "structured_issue_detected": true,
        "next_actions": [
            super::workflow_common::next_action(interface, next_tool, json!({"change_id": change_id}))
        ]
    });

    Ok(serde_json::to_string_pretty(&result)?)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/init_change.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:standardize-gap-sdd-init-change-standalone-flow>"
    description: "Standalone init-change flow for structured issue gates, worktree provisioning, issue frontmatter sync, and next-action routing."
```
