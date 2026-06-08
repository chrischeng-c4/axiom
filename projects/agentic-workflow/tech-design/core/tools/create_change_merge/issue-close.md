---
id: sdd-tools-create-change-merge-issue-close
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change merge issue close

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
      - "close_issue_if_exists"
    description: "Best-effort issue close helper used before post-archive git operations."
```
