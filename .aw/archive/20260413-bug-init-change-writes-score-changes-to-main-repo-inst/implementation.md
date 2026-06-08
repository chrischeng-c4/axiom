---
id: implementation
type: change_implementation
change_id: bug-init-change-writes-score-changes-to-main-repo-inst
---

# Implementation

## Changed Files

- `.score/tech_design/crates/sdd/logic/issue-centric-workflow.md` — Added Storage Model section, clarified change_dir pseudocode

## Diff

```diff
diff --git a/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md b/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md
index 10770fc4..e650aec1 100644
--- a/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md
+++ b/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md
@@ -19,6 +19,23 @@ Make the issue the single unit of work in SDD. Every change requires an issue. I
 | Multi-issue changes allowed | One issue = one change |
 | No worktree isolation | Each change gets its own worktree at `.score/worktrees/<slug>` |
 
+### Storage Model
+
+<!-- type: storage-model lang: markdown -->
+
+SDD uses a control plane / data plane split across the main repo and worktrees:
+
+| Layer | Location | Contents |
+|-------|----------|----------|
+| **Control plane** | Main repo (project_root) | `.score/issues/` (lifecycle + phase), `.score/changes/<id>/` (STATE, specs, prompts, payloads, implementation.md), `.score/config.toml`, `.score/archive/` |
+| **Data plane** | Worktree (`.score/worktrees/<slug>/`) | Code changes (`crates/`, `projects/`, etc.), `.score/tech_design/` changes |
+
+**Key invariant**: `.score/changes/<id>/` always lives on `project_root` (main repo), NOT inside the worktree. This is correct by design:
+- `score run-change` runs from main repo and must always find change artifacts
+- Phase state (in issue frontmatter) is on main repo, accessible without knowing which worktree
+- The worktree provides git branch isolation for code changes only
+- `init_change` creates change artifacts first, then creates the worktree — this order is intentional
+
 ### Components
 
 | Component | File | Change |
@@ -438,13 +455,21 @@ changes:
     details: |
       Replace --description with --issue <slug> flow:
       let issue = Issue::load_by_slug(slug)?;
+      // Step 1: Create change artifacts on main repo (control plane)
+      // change_dir is ALWAYS relative to project_root (main repo), not worktree.
+      // See Storage Model section above.
+      let change_dir = project_root.join(".score/changes").join(slug);
+      create_change_dir(&change_dir, &issue)?;  // writes STATE.yaml, groups/, etc.
+
+      // Step 2: Create worktree for code isolation (data plane)
       let worktree_path = format!(".score/worktrees/{}", slug);
       git_worktree_add(&worktree_path, &branch_name)?;
+
+      // Step 3: Update issue frontmatter with phase + worktree reference
       issue.phase = Some("change_inited".to_string());
       issue.branch = Some(branch_name.to_string());
       issue.worktree_path = Some(worktree_path);
       issue.save()?;
-      let change_dir = format!(".score/changes/{}/", slug);
 
   - file: crates/sdd/src/workflow/mod.rs
     action: modify
```

## Review: bug-init-change-writes-score-changes-to-main-repo-inst-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: bug-init-change-writes-score-changes-to-main-repo-inst

**Summary**: Storage Model section added. Pseudocode clarified with explicit project_root base and step numbering. Execution order documented as by design.



## Alignment Warnings

6 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md | missing_section_annotation | Section 'Diagrams' at line 39 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md | missing_section_annotation | Section 'API Spec' at line 121 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md | format_priority_violation | Section 'Requirements' (type: requirements) requires a ```mermaid code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/issue-centric-workflow.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
