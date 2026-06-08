---
id: implementation
type: change_implementation
change_id: bug-find-project-root-must-resolve-to-main-repo-when-i
---

# Implementation

## Summary

Fix find_project_root worktree resolution (R1-R6).

- R1/R3: extend find_project_root with git-aware post-check via git rev-parse --git-common-dir; redirects to main repo root when CWD is inside a linked worktree
- R2: signature and return type unchanged; compile-time test asserts
- R4: CWD in main repo (root or subdir) unchanged — redirect becomes a no-op
- R5: fallback chain preserves walk-up behavior for non-git tempdir contexts
- R6: close_issue_if_exists call sites already pass project_root (verified inspection); post-fix project_root is main repo root

Tests: 4 new #[test] in projects/score/cli/src/lib.rs using real git binary with tempdir-based repos + linked worktrees. CWD_LOCK mutex serializes process-global CWD mutations. All 85 score-cli lib tests passing.

## Diff

```diff
diff --git a/.score/issues/open/bug-find-project-root-must-resolve-to-main-repo-when-i.md b/.score/issues/open/bug-find-project-root-must-resolve-to-main-repo-when-i.md
index 2c9b1364..1f54b246 100644
--- a/.score/issues/open/bug-find-project-root-must-resolve-to-main-repo-when-i.md
+++ b/.score/issues/open/bug-find-project-root-must-resolve-to-main-repo-when-i.md
@@ -6,9 +6,25 @@ id: 7cfccadc-4d27-4231-8f64-3e9385297586
 labels:
 - crate:sdd
 - type:bug
+phase: change_spec_created
+branch: cclab/bug-find-project-root-must-resolve-to-main-repo-when-i
+git_workflow: worktree
+change_id: bug-find-project-root-must-resolve-to-main-repo-when-i
+iteration: 1
+current_task_id: bug-find-project-root-must-resolve-to-main-repo-when-i-spec
+impl_spec_phase: {}
+task_revisions: {}
+revision_counts: {}
 ---
 
 
+
+
+
+
+
+
+
 ## Problem
 
 `score` CLI resolves `project_root` by walking up from CWD looking for `.score/config.toml`. Every SDD worktree at `.score/worktrees/<slug>/` has its own `.score/config.toml`, so when any `score` command is invoked from inside a worktree, `project_root` resolves to the worktree path — not the main repo.
diff --git a/projects/score/cli/src/lib.rs b/projects/score/cli/src/lib.rs
index 2719ec50..a3da6317 100644
--- a/projects/score/cli/src/lib.rs
+++ b/projects/score/cli/src/lib.rs
@@ -33,12 +33,21 @@ pub use commands::{run_command, Commands};
 
 /// Find the project root by walking up from CWD looking for `.score/config.toml`.
 /// Falls back to CWD if no `.score/` is found (e.g., during `score init`).
+///
+/// When the discovered directory is a git linked worktree (each worktree under
+/// `.score/worktrees/<slug>/` carries its own `.score/config.toml`), redirect to
+/// the main repo root via `git rev-parse --git-common-dir`. Downstream callers
+/// such as `resolve_worktree_dir` and `close_issue_if_exists` need the main
+/// repo's perspective to merge branches and move issues into `main/.score/`.
+///
+/// See `.score/tech_design/crates/sdd/logic/change-merge.md` R1.
+// @spec .score/changes/bug-find-project-root-must-resolve-to-main-repo-when-i/specs/bug-find-project-root-must-resolve-to-main-repo-when-i-spec.md#R1
 pub fn find_project_root() -> anyhow::Result<std::path::PathBuf> {
     let cwd = std::env::current_dir()?;
     let mut dir = cwd.as_path();
     loop {
         if dir.join(".score/config.toml").exists() {
-            return Ok(dir.to_path_buf());
+            return Ok(resolve_main_repo_root(dir));
         }
         match dir.parent() {
             Some(parent) => dir = parent,
@@ -49,3 +58,186 @@ pub fn find_project_root() -> anyhow::Result<std::path::PathBuf> {
         }
     }
 }
+
+/// Redirect `candidate` to the main repo root when it sits inside a git linked
+/// worktree. Returns `candidate` unchanged for non-git contexts or when already
+/// at the main repo.
+fn resolve_main_repo_root(candidate: &std::path::Path) -> std::path::PathBuf {
+    if let Some(main_root) = main_root_via_git(candidate) {
+        if main_root != candidate && main_root.join(".score/config.toml").exists() {
+            return main_root;
+        }
+    }
+    if let Some(main_root) = main_root_via_dotgit_file(candidate) {
+        if main_root != candidate && main_root.join(".score/config.toml").exists() {
+            return main_root;
+        }
+    }
+    candidate.to_path_buf()
+}
+
+/// Ask git for the common (primary) `.git` directory. Parent of that path is
+/// the main repo root. Returns `None` when `git` is unavailable or `candidate`
+/// is not inside a git repo.
+fn main_root_via_git(candidate: &std::path::Path) -> Option<std::path::PathBuf> {
+    let output = std::process::Command::new("git")
+        .args(["-C"])
+        .arg(candidate)
+        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
+        .output()
+        .ok()?;
+    if !output.status.success() {
+        return None;
+    }
+    let git_dir = std::path::PathBuf::from(String::from_utf8(output.stdout).ok()?.trim());
+    git_dir.parent().map(|p| p.to_path_buf())
+}
+
+/// Fallback for when the `git` binary is missing: parse `<candidate>/.git` as a
+/// file containing `gitdir: <main>/.git/worktrees/<name>`. Derive the main repo
+/// root by walking up from the gitdir.
+fn main_root_via_dotgit_file(candidate: &std::path::Path) -> Option<std::path::PathBuf> {
+    let dot_git = candidate.join(".git");
+    if !dot_git.is_file() {
+        return None;
+    }
+    let contents = std::fs::read_to_string(&dot_git).ok()?;
+    let gitdir_line = contents.lines().find_map(|l| l.strip_prefix("gitdir:"))?;
+    let gitdir = std::path::PathBuf::from(gitdir_line.trim());
+    // gitdir looks like `<main>/.git/worktrees/<name>` — main root is 3 levels up.
+    gitdir.parent()?.parent()?.parent().map(|p| p.to_path_buf())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::path::PathBuf;
+    use std::sync::Mutex;
+
+    // find_project_root() reads process-global CWD; serialize tests that
+    // mutate it so parallel cargo test runs don't race.
+    static CWD_LOCK: Mutex<()> = Mutex::new(());
+
+    fn git_available() -> bool {
+        std::process::Command::new("git")
+            .arg("--version")
+            .output()
+            .map(|o| o.status.success())
+            .unwrap_or(false)
+    }
+
+    fn init_git_repo(path: &std::path::Path) {
+        for args in [
+            vec!["init", "-q", "-b", "main"],
+            vec!["config", "user.email", "test@example.com"],
+            vec!["config", "user.name", "Test"],
+            vec!["commit", "--allow-empty", "-m", "init", "-q"],
+        ] {
+            let out = std::process::Command::new("git")
+                .args(&args)
+                .current_dir(path)
+                .output()
+                .expect("git command");
+            assert!(out.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&out.stderr));
+        }
+    }
+
+    // Canonicalize via `std::fs::canonicalize`. On macOS, tempdir paths go
+    // through /private/var/… so comparisons need canonicalized values on both
+    // sides.
+    fn canon(p: &std::path::Path) -> PathBuf {
+        std::fs::canonicalize(p).unwrap()
+    }
+
+    // @spec .score/changes/bug-find-project-root-must-resolve-to-main-repo-when-i/specs/bug-find-project-root-must-resolve-to-main-repo-when-i-spec.md#R1
+    // @spec .score/changes/bug-find-project-root-must-resolve-to-main-repo-when-i/specs/bug-find-project-root-must-resolve-to-main-repo-when-i-spec.md#R3
+    #[test]
+    fn find_project_root_inside_worktree_returns_main() {
+        if !git_available() {
+            return;
+        }
+        let _guard = CWD_LOCK.lock().unwrap();
+        let prev = std::env::current_dir().unwrap();
+
+        let tmp = tempfile::TempDir::new().unwrap();
+        let main_repo = tmp.path().join("main");
+        std::fs::create_dir_all(&main_repo).unwrap();
+        init_git_repo(&main_repo);
+        std::fs::create_dir_all(main_repo.join(".score")).unwrap();
+        std::fs::write(main_repo.join(".score/config.toml"), "").unwrap();
+
+        let worktree_rel = ".score/worktrees/foo";
+        let out = std::process::Command::new("git")
+            .args(["worktree", "add", "-b", "cclab/foo", worktree_rel])
+            .current_dir(&main_repo)
+            .output()
+            .expect("git worktree add");
+        assert!(out.status.success(), "worktree add: {}", String::from_utf8_lossy(&out.stderr));
+
+        let worktree = main_repo.join(worktree_rel);
+        std::fs::create_dir_all(worktree.join(".score")).unwrap();
+        std::fs::write(worktree.join(".score/config.toml"), "").unwrap();
+
+        std::env::set_current_dir(&worktree).unwrap();
+        let resolved = find_project_root().unwrap();
+        std::env::set_current_dir(&prev).unwrap();
+
+        assert_eq!(
+            canon(&resolved),
+            canon(&main_repo),
+            "worktree CWD must resolve to main repo root"
+        );
+    }
+
+    // @spec .score/changes/bug-find-project-root-must-resolve-to-main-repo-when-i/specs/bug-find-project-root-must-resolve-to-main-repo-when-i-spec.md#R4
+    #[test]
+    fn find_project_root_inside_main_repo_unchanged() {
+        if !git_available() {
+            return;
+        }
+        let _guard = CWD_LOCK.lock().unwrap();
+        let prev = std::env::current_dir().unwrap();
+
+        let tmp = tempfile::TempDir::new().unwrap();
+        let repo = tmp.path().join("repo");
+        std::fs::create_dir_all(&repo).unwrap();
+        init_git_repo(&repo);
+        std::fs::create_dir_all(repo.join(".score")).unwrap();
+        std::fs::write(repo.join(".score/config.toml"), "").unwrap();
+        let subdir = repo.join("crates/demo");
+        std::fs::create_dir_all(&subdir).unwrap();
+
+        std::env::set_current_dir(&repo).unwrap();
+        let from_root = find_project_root().unwrap();
+        std::env::set_current_dir(&subdir).unwrap();
+        let from_subdir = find_project_root().unwrap();
+        std::env::set_current_dir(&prev).unwrap();
+
+        assert_eq!(canon(&from_root), canon(&repo));
+        assert_eq!(canon(&from_subdir), canon(&repo));
+    }
+
+    // @spec .score/changes/bug-find-project-root-must-resolve-to-main-repo-when-i/specs/bug-find-project-root-must-resolve-to-main-repo-when-i-spec.md#R5
+    #[test]
+    fn find_project_root_non_git_tempdir_walks_up() {
+        let _guard = CWD_LOCK.lock().unwrap();
+        let prev = std::env::current_dir().unwrap();
+
+        let tmp = tempfile::TempDir::new().unwrap();
+        let proj = tmp.path().join("proj");
+        std::fs::create_dir_all(proj.join(".score")).unwrap();
+        std::fs::write(proj.join(".score/config.toml"), "").unwrap();
+
+        std::env::set_current_dir(&proj).unwrap();
+        let resolved = find_project_root().unwrap();
+        std::env::set_current_dir(&prev).unwrap();
+
+        assert_eq!(canon(&resolved), canon(&proj), "non-git context must fall back to walk-up");
+    }
+
+    // @spec .score/changes/bug-find-project-root-must-resolve-to-main-repo-when-i/specs/bug-find-project-root-must-resolve-to-main-repo-when-i-spec.md#R2
+    #[test]
+    fn find_project_root_signature_unchanged_compile_check() {
+        let _f: fn() -> anyhow::Result<std::path::PathBuf> = find_project_root;
+    }
+}

```

## Review: bug-find-project-root-must-resolve-to-main-repo-when-i-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: bug-find-project-root-must-resolve-to-main-repo-when-i

**Summary**: Fix satisfies R1-R6. find_project_root now redirects worktree CWD to main repo via git rev-parse --git-common-dir with .git-file fallback for non-git contexts. Signature unchanged (R2). 4 #[test] cover T1-T4 with CWD serialization. 85/85 score-cli lib tests pass. close_issue_if_exists call sites already pass project_root (R6 inspection).

### Checklist

- [PASS] Code matches all spec requirements
  - R1 git-common-dir detection, R2 signature preserved, R3/R4/R5 scenarios covered, R6 call-site verified
- [PASS] Test Plan → implementation has #[test] functions
  - 4 #[test] functions in lib.rs::tests, matching T1-T4
- [PASS] Existing tests still pass (no regressions)
  - cargo test -p score-cli --lib: 85 passed; 0 failed
- [PASS] Code quality
  - Clear helper split (main_root_via_git + main_root_via_dotgit_file); @spec annotations added
- [PASS] Error handling
  - Fallback chain: git → .git-file → walk-up; all failures collapse to candidate (safe default)



## Alignment Warnings

5 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-find-project-root-must-resolve-to-main-repo-when-i/.score/tech_design/crates/sdd/logic/change-merge.md | missing_section_annotation | Section 'Diagrams' at line 127 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-find-project-root-must-resolve-to-main-repo-when-i/.score/tech_design/crates/sdd/logic/change-merge.md | missing_section_annotation | Section 'API Spec' at line 209 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-find-project-root-must-resolve-to-main-repo-when-i/.score/tech_design/crates/sdd/logic/change-merge.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-find-project-root-must-resolve-to-main-repo-when-i/.score/tech_design/crates/sdd/logic/change-merge.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/worktrees/bug-find-project-root-must-resolve-to-main-repo-when-i/.score/tech_design/crates/sdd/logic/change-merge.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
