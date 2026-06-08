---
id: implementation
type: change_implementation
change_id: sdd-merge-gaps-fix
---

# Implementation

## Summary

Fixed 3 merge gaps in create_change_merge / merge_git_ops workflow.

## Diff

```diff
diff --git a/crates/sdd/src/tools/create_change_merge.rs b/crates/sdd/src/tools/create_change_merge.rs
index 104d92b3..cb72eeee 100644
--- a/crates/sdd/src/tools/create_change_merge.rs
+++ b/crates/sdd/src/tools/create_change_merge.rs
@@ -337,8 +337,14 @@ pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<Strin
 
 // ─── Issue Closing (part of step 5) ──────────────────────────────────────────
 
-/// Move the issue with slug `change_id` from `.score/issues/open/` to
-/// `.score/issues/closed/`, updating `state: closed` in frontmatter.
+/// Move the open issue associated with `change_id` to `.score/issues/closed/`,
+/// updating `state: closed` in frontmatter.
+///
+/// Matching strategy (in order):
+/// 1. **Slug match** — open issue file named `{change_id}.md`.
+/// 2. **Frontmatter id match** — scan all open issues and find one whose `id`
+///    field (UUID) equals `change_id`. This handles cases where the issue file
+///    was created with a UUID-based name but references this change by UUID.
 ///
 /// Worktree path / phase fields are cleared so the closed record accurately
 /// reflects that the change is done.
@@ -346,34 +352,64 @@ pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<Strin
 /// Returns `true` if an issue was found and closed, `false` otherwise (legacy
 /// changes without an associated issue).
 // REQ: worktree-per-change — issue open/→closed/ move on merge
+// @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R3
 fn close_issue_if_exists(project_root: &Path, change_id: &str) -> bool {
     use crate::issues::{local_backend, IssueBackend};
 
-    // Only act when an open issue file exists with this slug — otherwise this
-    // is a legacy change with no issue to close.
-    let open_path = project_root
-        .join(".score/issues/open")
-        .join(format!("{}.md", change_id));
-    if !open_path.exists() {
+    // Quick guard: if neither the slug file nor the open/ directory exists,
+    // there is nothing to close. This avoids hitting block_in_place on a
+    // single-threaded runtime when no issue files are present.
+    let open_dir = project_root.join(".score/issues/open");
+    let slug_path = open_dir.join(format!("{}.md", change_id));
+    if !slug_path.exists() && !open_dir.exists() {
         return false;
     }
 
     let backend = local_backend(project_root);
 
-    let result: std::result::Result<(), anyhow::Error> = (|| {
-        // Use block-in-place so we can call the async backend from this sync
-        // context without deadlocking the current tokio runtime.
-        let rt = tokio::runtime::Handle::try_current();
-        let issue_opt = if let Ok(handle) = rt {
-            tokio::task::block_in_place(|| handle.block_on(backend.get(change_id)))?
+    let result: std::result::Result<bool, anyhow::Error> = (|| {
+        let rt_handle = tokio::runtime::Handle::try_current();
+
+        // ── Strategy 1: slug match ─────────────────────────────────────────
+        // Only attempt if the slug file exists — avoids block_in_place on
+        // single-threaded runtimes when there are no open issues.
+        let issue_opt: Option<crate::issues::Issue> = if slug_path.exists() {
+            if let Ok(handle) = &rt_handle {
+                tokio::task::block_in_place(|| handle.block_on(backend.get(change_id)))?
+            } else {
+                let rt = tokio::runtime::Runtime::new()?;
+                rt.block_on(backend.get(change_id))?
+            }
+        } else {
+            None
+        };
+
+        let issue_to_close: Option<crate::issues::Issue> = if issue_opt.is_some() {
+            issue_opt
+        } else if open_dir.exists() {
+            // ── Strategy 2: frontmatter id match ──────────────────────────
+            // Scan all open issues and find one whose `id` field matches.
+            let all_issues = if let Ok(handle) = &rt_handle {
+                tokio::task::block_in_place(|| {
+                    handle.block_on(backend.list(&crate::issues::IssueFilter::default()))
+                })?
+            } else {
+                let rt = tokio::runtime::Runtime::new()?;
+                rt.block_on(backend.list(&crate::issues::IssueFilter::default()))?
+            };
+
+            // Match by frontmatter `id` field — only consider open issues.
+            all_issues
+                .into_iter()
+                .filter(|i| matches!(i.state, crate::issues::IssueState::Open | crate::issues::IssueState::Draft))
+                .find(|i| i.id.as_deref() == Some(change_id))
         } else {
-            let rt = tokio::runtime::Runtime::new()?;
-            rt.block_on(backend.get(change_id))?
+            None
         };
 
-        let mut issue = match issue_opt {
+        let mut issue = match issue_to_close {
             Some(i) => i,
-            None => return Ok(()), // silently no-op if slug resolves to nothing
+            None => return Ok(false), // no issue to close
         };
 
         // Clear workflow state and flip to closed — the backend's write()
@@ -383,21 +419,17 @@ fn close_issue_if_exists(project_root: &Path, change_id: &str) -> bool {
         issue.branch = None;
         issue.git_workflow = None;
 
-        if let Ok(handle) = tokio::runtime::Handle::try_current() {
+        if let Ok(handle) = &rt_handle {
             tokio::task::block_in_place(|| handle.block_on(backend.write(&issue)))?;
         } else {
             let rt = tokio::runtime::Runtime::new()?;
             rt.block_on(backend.write(&issue))?;
         }
-        Ok(())
+        Ok(true)
     })();
 
     match result {
-        Ok(()) => {
-            // Success means the open file should no longer exist; use its
-            // absence as the signal to the caller.
-            !open_path.exists()
-        }
+        Ok(closed) => closed,
         Err(e) => {
             tracing::warn!(
                 change_id = %change_id,
@@ -1132,4 +1164,97 @@ Change updated section two.\n";
         assert!(!change_dir.exists());
     }
 
+    // Fix 3: close_issue_if_exists must match by frontmatter id (UUID) in addition to slug.
+    // @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R3
+    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
+    async fn test_merge_closes_issue_by_frontmatter_id() {
+        // Issue file is named 'bug-unrelated-name.md' but its frontmatter id matches change_id.
+        let change_id = "my-feature-change";
+        let issue_slug = "bug-unrelated-name"; // slug does NOT match change_id
+
+        let tmp = setup_change(change_id, StatePhase::ChangeImplementationReviewed);
+        let change_dir = tmp.path().join(format!(".score/changes/{}", change_id));
+        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
+
+        // Create an open issue whose slug does NOT match, but whose frontmatter id DOES match.
+        let open_dir = tmp.path().join(".score/issues/open");
+        std::fs::create_dir_all(&open_dir).unwrap();
+        std::fs::write(
+            open_dir.join(format!("{}.md", issue_slug)),
+            format!(
+                "---\ntype: bug\ntitle: Unrelated slug but matching id\nstate: open\nid: {}\n---\n\n## Body\n",
+                change_id
+            ),
+        )
+        .unwrap();
+
+        // Minimal valid spec
+        let spec_content = "---\nid: fix-spec\nmain_spec_ref: sdd/logic/fix-spec.md\n---\n\n# Fix Spec\n\nContent.\n";
+        std::fs::write(change_dir.join("specs/fix-spec.md"), spec_content).unwrap();
+
+        let args = json!({
+            "project_path": tmp.path().to_str().unwrap(),
+            "change_id": change_id
+        });
+        let result = execute_workflow(&args, tmp.path()).await.unwrap();
+        let parsed: Value = serde_json::from_str(&result).unwrap();
+        assert_eq!(parsed["status"], "ok");
+        assert_eq!(
+            parsed["issue_closed"], true,
+            "issue_closed must be true when matched by frontmatter id"
+        );
+
+        // Slug-named file moved from open/ to closed/.
+        assert!(
+            !open_dir.join(format!("{}.md", issue_slug)).exists(),
+            "open issue file must be moved to closed/"
+        );
+        let closed_path = tmp.path().join(format!(".score/issues/closed/{}.md", issue_slug));
+        assert!(closed_path.exists(), "closed issue file must exist at {}", closed_path.display());
+
+        // State must be closed.
+        let content = std::fs::read_to_string(&closed_path).unwrap();
+        assert!(content.contains("state: closed"), "closed issue must have state: closed");
+    }
+
+    // Fix 3: when no issue matches (by slug or frontmatter id), issue_closed=false.
+    // @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R3
+    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
+    async fn test_merge_no_issue_match_returns_false() {
+        let change_id = "xyz-nonexistent-issue";
+        let tmp = setup_change(change_id, StatePhase::ChangeImplementationReviewed);
+        let change_dir = tmp.path().join(format!(".score/changes/{}", change_id));
+        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
+
+        // Create an open issue with a different id and different slug.
+        let open_dir = tmp.path().join(".score/issues/open");
+        std::fs::create_dir_all(&open_dir).unwrap();
+        std::fs::write(
+            open_dir.join("some-other-issue.md"),
+            "---\ntype: bug\ntitle: Some other issue\nstate: open\nid: some-completely-different-uuid\n---\n\n## Body\n",
+        )
+        .unwrap();
+
+        let spec_content = "---\nid: xyz-spec\nmain_spec_ref: sdd/logic/xyz-spec.md\n---\n\n# Xyz Spec\n\nContent.\n";
+        std::fs::write(change_dir.join("specs/xyz-spec.md"), spec_content).unwrap();
+
+        let args = json!({
+            "project_path": tmp.path().to_str().unwrap(),
+            "change_id": change_id
+        });
+        let result = execute_workflow(&args, tmp.path()).await.unwrap();
+        let parsed: Value = serde_json::from_str(&result).unwrap();
+        assert_eq!(parsed["status"], "ok");
+        assert_eq!(
+            parsed["issue_closed"], false,
+            "issue_closed must be false when no issue matches by slug or frontmatter id"
+        );
+
+        // The other issue must remain open.
+        assert!(
+            open_dir.join("some-other-issue.md").exists(),
+            "unmatched issue must remain in open/"
+        );
+    }
+
 }
diff --git a/crates/sdd/src/tools/merge_git_ops.rs b/crates/sdd/src/tools/merge_git_ops.rs
index e6a9e40e..d0473fd2 100644
--- a/crates/sdd/src/tools/merge_git_ops.rs
+++ b/crates/sdd/src/tools/merge_git_ops.rs
@@ -11,9 +11,9 @@
 //! The 5-step merge sequence (see `.score/tech_design/crates/sdd/logic/change-merge.md`):
 //! 1. SDD archive (handled upstream in `create_change_merge.rs`)
 //! 2. Auto-commit in worktree branch — warning on failure
-//! 3. `git merge cclab/<slug>` into main — **hard error** on conflict
+//! 5. `gh pr create` (when auto_pr=true and worktree exists) — **before** local merge so branch is still alive — warning on failure
+//! 3. `git merge cclab/<slug>` into main — **hard error** on conflict; merge commit SHA captured after success
 //! 4. `git worktree remove` + `git branch -d` — warning on failure
-//! 5. `gh pr create` — warning on failure
 
 use crate::models::change::RepoPlatformConfig;
 use crate::Result;
@@ -182,13 +182,40 @@ pub(super) fn post_archive_git_ops(
         commit_sha = extract_commit_sha(&commit_stdout, &git_bin, commit_cwd);
     }
 
+    // ── Step 5 (early): Auto-PR before local merge ─────────────────────────
+    //
+    // When auto_pr=true and a worktree exists, create the PR BEFORE local
+    // merge steps 3+4. The PR targets the worktree branch which is still
+    // alive at this point. In legacy in-place flow (no worktree), auto-PR
+    // is skipped since there is no separate branch to PR from.
+    // @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R2
+    let mut step_warnings: Vec<String> = Vec::new();
+    let (pr_url, pr_warning) = if config.auto_pr && worktree_dir.is_some() {
+        match create_pr(project_root, change_id, archive_path, merged_specs, config) {
+            Ok(url) => (Some(url), None),
+            Err(warning) => (None, Some(warning)),
+        }
+    } else {
+        (None, None)
+    };
+    if let Some(pw) = pr_warning {
+        step_warnings.push(pw);
+    }
+
     // ── Step 3: Merge worktree branch into default branch ──────────────────
     //
     // Only runs when a worktree exists. Fail-fast on merge conflict.
-    let mut step_warnings: Vec<String> = Vec::new();
     if worktree_dir.is_some() {
         match merge_worktree_branch(&git_bin, project_root, change_id, &config.default_branch) {
-            MergeOutcome::Ok => {}
+            MergeOutcome::Ok => {
+                // @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R1
+                // After a successful merge, capture the merge commit SHA from the
+                // default branch HEAD (project_root). This is the canonical SHA —
+                // the commit that actually landed on the default branch.
+                if let Some(sha) = capture_head_sha(&git_bin, project_root) {
+                    commit_sha = Some(sha);
+                }
+            }
             MergeOutcome::Warning(w) => step_warnings.push(w),
             MergeOutcome::Conflict(msg) => {
                 anyhow::bail!(
@@ -207,31 +234,11 @@ pub(super) fn post_archive_git_ops(
         }
     }
 
-    // ── Step 5: Auto-PR ─────────────────────────────────────────────────────
-    //
-    // Runs from project_root (main branch now, not the worktree which may be gone).
-    let (pr_url, pr_warning) = if config.auto_pr {
-        match create_pr(project_root, change_id, archive_path, merged_specs, config) {
-            Ok(url) => (Some(url), None),
-            Err(warning) => (None, Some(warning)),
-        }
-    } else {
-        (None, None)
-    };
-
-    // Merge any step-3/step-4 warnings with the PR warning (PR is last).
-    let combined_warning = if step_warnings.is_empty() && pr_warning.is_none() {
+    // Combine all warnings.
+    let combined_warning = if step_warnings.is_empty() {
         None
-    } else if !step_warnings.is_empty() && pr_warning.is_none() {
+    } else {
         Some(step_warnings.join(" | "))
-    } else if step_warnings.is_empty() && pr_warning.is_some() {
-        pr_warning
-    } else {
-        let mut all = step_warnings;
-        if let Some(pw) = pr_warning {
-            all.push(pw);
-        }
-        Some(all.join(" | "))
     };
 
     Ok(GitOpsResult {
@@ -499,6 +506,26 @@ fn build_commit_message(change_id: &str, summary: Option<&str>) -> String {
     }
 }
 
+/// Capture the current HEAD SHA by running `git rev-parse HEAD` in `dir`.
+///
+/// Used after step 3 (worktree merge into default branch) to record the merge
+/// commit SHA on the default branch.
+// @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R1
+fn capture_head_sha(git_bin: &Path, dir: &Path) -> Option<String> {
+    let output = std::process::Command::new(git_bin)
+        .args(["rev-parse", "HEAD"])
+        .current_dir(dir)
+        .output()
+        .ok()?;
+    if output.status.success() {
+        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
+        if !sha.is_empty() {
+            return Some(sha);
+        }
+    }
+    None
+}
+
 /// Extract the commit SHA from git output or by running `git rev-parse HEAD`.
 fn extract_commit_sha(commit_stdout: &str, git_bin: &Path, project_root: &Path) -> Option<String> {
     // Try to parse SHA from commit output (format varies by git version)
@@ -1017,4 +1044,158 @@ path = ".score/tech_design"
         assert!(ops.git_commit_sha.is_none());
         assert!(ops.git_warning.is_none() || !ops.git_warning.as_deref().unwrap_or("").contains("conflict"));
     }
+
+    /// Fix 1: After step 3 merge, git_commit_sha must be the merge commit SHA
+    /// on project_root (not the worktree branch commit), and must not be null.
+    // @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R1
+    #[test]
+    fn test_commit_sha_is_merge_commit_on_default_branch() {
+        let tmp = TempDir::new().unwrap();
+        if !init_repo(tmp.path()) {
+            return;
+        }
+
+        let slug = "fix1-sha-capture";
+        let Some(git) = find_git_binary() else { return };
+
+        // Create a worktree branch and add a file to it.
+        let wt_rel = format!(".score/worktrees/{}", slug);
+        std::fs::create_dir_all(tmp.path().join(".score/worktrees")).unwrap();
+        let add_wt = std::process::Command::new(&git)
+            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
+            .current_dir(tmp.path())
+            .output()
+            .unwrap();
+        assert!(add_wt.status.success(), "git worktree add failed: {}", String::from_utf8_lossy(&add_wt.stderr));
+
+        let wt_abs = tmp.path().join(&wt_rel);
+        std::fs::create_dir_all(wt_abs.join("cclab")).unwrap();
+        std::fs::write(wt_abs.join("cclab/test-file.txt"), "sha test\n").unwrap();
+
+        let archive_abs = tmp.path().join(".score/archive/20260101-fix1-sha");
+        std::fs::create_dir_all(&archive_abs).unwrap();
+        std::fs::write(archive_abs.join("user_input.md"), "SHA capture fix\n").unwrap();
+
+        let config = RepoPlatformConfig {
+            type_: "github".to_string(),
+            repo: "test/repo".to_string(),
+            default_branch: "main".to_string(),
+            auto_commit: true,
+            auto_pr: false,
+        };
+
+        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
+        assert!(result.is_ok(), "post_archive_git_ops failed: {:?}", result.err());
+        let ops = result.unwrap();
+
+        // Must have a SHA — the merge commit on main.
+        assert!(ops.git_commit_sha.is_some(), "git_commit_sha must not be null after merge");
+        let sha = ops.git_commit_sha.unwrap();
+        assert!(sha.len() >= 7, "SHA must be at least 7 chars, got: {}", sha);
+        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()), "SHA must be hex, got: {}", sha);
+
+        // Verify the SHA matches HEAD on project_root (main branch after merge).
+        let head_output = std::process::Command::new(&git)
+            .args(["rev-parse", "HEAD"])
+            .current_dir(tmp.path())
+            .output()
+            .unwrap();
+        let head_sha = String::from_utf8_lossy(&head_output.stdout).trim().to_string();
+        assert!(
+            head_sha.starts_with(&sha) || sha.starts_with(&head_sha[..sha.len().min(head_sha.len())]),
+            "reported SHA {} must match or be prefix of HEAD SHA {}",
+            sha, head_sha
+        );
+    }
+
+    /// Fix 2: auto_pr=true in legacy in-place flow (no worktree) must NOT attempt PR creation.
+    // @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R2
+    #[test]
+    fn test_auto_pr_skipped_in_legacy_flow() {
+        let tmp = TempDir::new().unwrap();
+        if !init_repo(tmp.path()) {
+            return;
+        }
+
+        // No worktree created — legacy in-place flow.
+        let archive_abs = tmp.path().join(".score/archive/20260101-legacy-pr");
+        std::fs::create_dir_all(&archive_abs).unwrap();
+        std::fs::write(archive_abs.join("user_input.md"), "legacy auto-pr test\n").unwrap();
+
+        let config = RepoPlatformConfig {
+            type_: "github".to_string(),
+            repo: "test/repo".to_string(),
+            default_branch: "main".to_string(),
+            auto_commit: true,
+            auto_pr: true, // enabled, but no worktree → must be skipped
+        };
+
+        let result = post_archive_git_ops(tmp.path(), "legacy-pr-change", &archive_abs, Some(&config), &[]);
+        assert!(result.is_ok(), "should not error in legacy flow: {:?}", result.err());
+        let ops = result.unwrap();
+        // PR URL must be None (no worktree → PR skipped).
+        assert!(
+            ops.pr_url.is_none(),
+            "pr_url must be None in legacy flow when no worktree exists; got: {:?}",
+            ops.pr_url
+        );
+    }
+
+    /// Fix 2: auto_pr=false with a worktree must produce no PR URL and no PR warning.
+    // @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R2
+    #[test]
+    fn test_auto_pr_disabled_produces_no_pr_url() {
+        let tmp = TempDir::new().unwrap();
+        if !init_repo(tmp.path()) {
+            return;
+        }
+
+        let slug = "fix2-no-pr";
+        let Some(git) = find_git_binary() else { return };
+
+        let wt_rel = format!(".score/worktrees/{}", slug);
+        std::fs::create_dir_all(tmp.path().join(".score/worktrees")).unwrap();
+        std::process::Command::new(&git)
+            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
+            .current_dir(tmp.path())
+            .output()
+            .unwrap();
+
+        let wt_abs = tmp.path().join(&wt_rel);
+        std::fs::create_dir_all(wt_abs.join("cclab")).unwrap();
+        std::fs::write(wt_abs.join("cclab/test.txt"), "content\n").unwrap();
+
+        let archive_abs = tmp.path().join(".score/archive/20260101-fix2-no-pr");
+        std::fs::create_dir_all(&archive_abs).unwrap();
+        std::fs::write(archive_abs.join("user_input.md"), "auto-pr disabled test\n").unwrap();
+
+        let config = RepoPlatformConfig {
+            type_: "github".to_string(),
+            repo: "test/repo".to_string(),
+            default_branch: "main".to_string(),
+            auto_commit: true,
+            auto_pr: false,
+        };
+
+        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
+        assert!(result.is_ok(), "should not error: {:?}", result.err());
+        let ops = result.unwrap();
+        assert!(ops.pr_url.is_none(), "pr_url must be None when auto_pr=false");
+    }
+
+    /// Fix 3: capture_head_sha returns the current HEAD SHA for a given dir.
+    // @spec .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md#R1
+    #[test]
+    fn test_capture_head_sha_returns_valid_sha() {
+        let tmp = TempDir::new().unwrap();
+        if !init_repo(tmp.path()) {
+            return;
+        }
+        let Some(git) = find_git_binary() else { return };
+        let sha = capture_head_sha(&git, tmp.path());
+        assert!(sha.is_some(), "capture_head_sha must return Some for a valid repo");
+        let sha_str = sha.unwrap();
+        assert_eq!(sha_str.len(), 40, "SHA must be 40 hex chars, got len={}", sha_str.len());
+        assert!(sha_str.chars().all(|c| c.is_ascii_hexdigit()), "SHA must be hex, got: {}", sha_str);
+    }
 }

```

## Review: sdd-merge-gaps-fix-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-merge-gaps-fix

**Summary**: All 3 bugs fixed and tested. R1 (git_commit_sha): capture_head_sha() captures merge commit SHA from project_root after step 3. R2 (auto-PR): create_pr() runs before local merge when auto_pr=true+worktree, skipped in legacy flow. R3 (issue close): close_issue_if_exists matches by slug then frontmatter id, with guard for single-threaded runtimes. All 1394 tests pass including 6 new test functions.



## Alignment Warnings

9 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/merge-gaps-fix.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/merge-gaps-fix.md | missing_section_annotation | Section 'Requirements' at line 18 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/merge-gaps-fix.md | missing_section_annotation | Section 'Scenarios' at line 47 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/merge-gaps-fix.md | missing_section_annotation | Section 'Diagrams' at line 85 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/merge-gaps-fix.md | missing_section_annotation | Section 'API Spec' at line 167 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/merge-gaps-fix.md | missing_section_annotation | Section 'Changes' at line 233 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/merge-gaps-fix.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/merge-gaps-fix.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/sdd/logic/merge-gaps-fix.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
