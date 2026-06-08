---
change: sdd-merge-gaps-fix
date: 2026-04-10
status: answered
---

# Pre-Clarifications

### Q1: git_commit_sha capture
- **Answer**: Return the merge commit SHA from project_root (git rev-parse HEAD after step 3 merge). This is the most useful SHA — it is the commit that landed on main. If no worktree merge happened (legacy in-place), use the worktree branch commit SHA captured during step 2.

### Q2: auto_pr flow position
- **Answer**: Auto-PR only applies in the worktree flow. When auto_pr=true and a worktree exists, create the PR BEFORE local merge steps 3+4. The PR represents the pending branch merge on GitHub. In legacy in-place flow, skip auto-PR since there is no separate branch.

### Q3: issue close matching
- **Answer**: Match issues by 3 strategies in order: (1) slug == change_id (file named {change_id}.md in open/), (2) frontmatter id UUID matches change_id, (3) scan all open issues and check frontmatter id field. The local backend get() already supports slug+numeric IDs. Add a scan-all fallback in close_issue_if_exists to match by frontmatter id field.

### Q4: cclab config unchanged
- **Answer**: cclab's own .score/config.toml will NOT be modified. auto_pr defaults to false. The fix makes auto_pr opt-in by moving PR creation before merge (so it only runs when explicitly enabled and a worktree branch exists).

