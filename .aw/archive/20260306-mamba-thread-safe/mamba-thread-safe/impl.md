# Implementation Diff

## Summary

```
Cargo.lock                                         |  78 ++++++-------
 Cargo.toml                                         |   2 +-
 crates/cclab-sdd/src/mcp/tools/agent.rs            |  10 ++
 .../cclab-sdd/src/mcp/tools/change_impl/create.rs  | 128 ++++++++++++++++++++-
 .../cclab-sdd/src/mcp/tools/change_spec/common.rs  |  26 +++++
 crates/cclab-sdd/src/workflow/advance.rs           |   9 ++
 6 files changed, 211 insertions(+), 42 deletions(-)
```

## Diff

```diff
diff --git a/Cargo.lock b/Cargo.lock
index 005f80f..0f448c6 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1158,7 +1158,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1169,7 +1169,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "cclab-api",
@@ -1200,7 +1200,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "pyo3",
@@ -1209,7 +1209,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-core"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "bson",
@@ -1227,7 +1227,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "aes-gcm",
  "argon2",
@@ -1254,7 +1254,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1281,7 +1281,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1294,7 +1294,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-core"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "bitvec",
  "regex",
@@ -1321,7 +1321,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-formula"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-grid-core",
  "nom 7.1.3",
@@ -1331,7 +1331,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-history"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1339,7 +1339,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-server"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "axum 0.7.9",
@@ -1363,7 +1363,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-wasm"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1381,7 +1381,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "async-trait",
  "bincode",
@@ -1410,7 +1410,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1422,7 +1422,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "clap",
@@ -1444,7 +1444,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba-tests"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-mamba",
  "datatest-stable",
@@ -1454,7 +1454,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "image",
  "pyo3",
@@ -1465,7 +1465,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1487,7 +1487,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-nucleus"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "bson",
  "cclab-agent",
@@ -1518,7 +1518,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1548,7 +1548,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "pyo3",
  "serde",
@@ -1558,7 +1558,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-prism"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1587,7 +1587,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1619,7 +1619,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "async-nats",
  "async-trait",
@@ -1660,7 +1660,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1686,7 +1686,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "bson",
  "dotenvy",
@@ -1701,7 +1701,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "cclab-array",
  "cclab-frame",
@@ -1714,7 +1714,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1762,7 +1762,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-server"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "async-stream",
@@ -1787,7 +1787,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1799,14 +1799,14 @@ dependencies = [
 
 [[package]]
 name = "cclab-util"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "pyo3",
 ]
 
 [[package]]
 name = "cclab-vortex"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "bytemuck",
  "env_logger",
@@ -1836,7 +1836,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "cclab-jet-asset",
@@ -1853,7 +1853,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-asset"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "image",
@@ -1864,7 +1864,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-bundler"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "cclab-jet-asset",
@@ -1886,7 +1886,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-dev-server"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1905,7 +1905,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-pkg-manager"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "reqwest",
@@ -1922,7 +1922,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-resolver"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "node-resolve",
@@ -1935,7 +1935,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-transform"
-version = "0.3.28"
+version = "0.3.29"
 dependencies = [
  "anyhow",
  "regex",
diff --git a/Cargo.toml b/Cargo.toml
index 811c830..013af6e 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -47,7 +47,7 @@ members = [
 resolver = "2"
 
 [workspace.package]
-version = "0.3.28"
+version = "0.3.29"
 authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
 edition = "2021"
 license = "MIT"
diff --git a/crates/cclab-sdd/src/mcp/tools/agent.rs b/crates/cclab-sdd/src/mcp/tools/agent.rs
index 782ea36..05e3841 100644
--- a/crates/cclab-sdd/src/mcp/tools/agent.rs
+++ b/crates/cclab-sdd/src/mcp/tools/agent.rs
@@ -467,10 +467,20 @@ pub async fn execute_streaming(
     });
 
     if agent_result.status == "error" && !agent_result.state_changed {
+        let expected_phase_hint = get_verification(&action)
+            .and_then(|v| v.expected_phases.first().cloned())
+            .map(|p| crate::mcp::tools::workflow_common::phase_to_string(&p))
+            .unwrap_or_default();
         response["error"] = json!({
             "type": "agent_failure",
             "message": "Agent failed verification or exited with error",
             "retried": true,
+            "fallback_hint": format!(
+                "Agent '{}' failed for action '{}'. \
+                 Mainthread should perform the action manually, then call \
+                 sdd_run_change(advance_to=\"{}\") to continue.",
+                agent, action, expected_phase_hint
+            ),
         });
     }
 
diff --git a/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs b/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs
index 30bdd05..37cd963 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs
@@ -155,11 +155,21 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "error",
                 "message": format!(
-                    "Spec '{}' failed review after {} revisions (limit: {}). Manual intervention required.",
+                    "Spec '{}' failed review after {} revisions (limit: {}). \
+                     Use next_actions to reset and retry, or fix manually.",
                     spec_id, revisions, MAX_SPEC_REVISIONS
                 ),
                 "spec_id": spec_id,
-                "next_actions": []
+                "next_actions": [{
+                    "tool": "sdd_run_change",
+                    "args": {
+                        "change_id": change_id,
+                        "advance_to": "change_implementation_created"
+                    },
+                    "executor": "mainthread",
+                    "when": "optional",
+                    "description": "Reset revision counter and retry implementation"
+                }]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -323,10 +333,87 @@ fn build_codegen_prompt(
     )
 }
 
+/// Run git command and return stdout (empty string on failure).
+fn git_output(args: &[&str]) -> String {
+    std::process::Command::new("git")
+        .args(args)
+        .output()
+        .ok()
+        .filter(|o| o.status.success())
+        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
+        .unwrap_or_default()
+}
+
+/// Auto-populate implementation.md with git diff baseline so the artifact
+/// is never empty even if the agent fails downstream.
+fn auto_populate_impl_baseline(change_id: &str, project_root: &Path) {
+    let change_dir = project_root.join("cclab/changes").join(change_id);
+    let impl_path = change_dir.join("implementation.md");
+    // Don't overwrite if already has real content
+    if impl_path.exists() {
+        if let Ok(existing) = std::fs::read_to_string(&impl_path) {
+            if existing.contains("## Diff") {
+                return;
+            }
+        }
+    }
+
+    let stat = git_output(&["diff", "--stat", "main"]);
+    let name_status = git_output(&["diff", "--name-status", "main"]);
+    if stat.is_empty() && name_status.is_empty() {
+        return;
+    }
+
+    let mut content = format!(
+        "---\nid: implementation\ntype: change_implementation\nchange_id: {change_id}\n---\n\n\
+         # Implementation\n\n\
+         ## Summary\n\n*(auto-generated baseline from git diff)*\n\n"
+    );
+
+    if !name_status.is_empty() {
+        content.push_str("## Changed Files\n\n```\n");
+        content.push_str(&name_status);
+        content.push_str("\n```\n\n");
+    }
+
+    if !stat.is_empty() {
+        content.push_str("## Diff Statistics\n\n```\n");
+        content.push_str(&stat);
+        content.push_str("\n```\n\n");
+    }
+
+    // Include truncated diff
+    let diff = git_output(&["diff", "main", "--", ".", ":!cclab/"]);
+    if !diff.is_empty() {
+        content.push_str("## Diff\n\n```diff\n");
+        const MAX_LINES: usize = 2000;
+        let lines: Vec<&str> = diff.lines().collect();
+        if lines.len() > MAX_LINES {
+            for line in &lines[..MAX_LINES] {
+                content.push_str(line);
+                content.push('\n');
+            }
+            content.push_str(&format!(
+                "\n... truncated ({} more lines)\n",
+                lines.len() - MAX_LINES
+            ));
+        } else {
+            content.push_str(&diff);
+            content.push('\n');
+        }
+        content.push_str("```\n");
+    }
+
+    let _ = std::fs::write(&impl_path, &content);
+}
+
 fn build_write_diff_prompt(
     change_id: &str,
     project_root: &Path,
 ) -> Result<String> {
+    // Auto-populate baseline so impl is never empty if agent fails
+    auto_populate_impl_baseline(change_id, project_root);
+
     let pp = project_root.display();
 
     let prompt = format!(
@@ -484,4 +571,41 @@ mod tests {
         assert_eq!(next[0]["tool"], "sdd_run_change");
         assert_eq!(next[0]["args"]["advance_to"], "change_merge_created");
     }
+
+    #[test]
+    fn test_terminal_failure_returns_retry_action() {
+        let tmp = setup_change("wf-fail", "change_implementation_reviewed");
+        write_spec(&tmp, "wf-fail", "spec-a", &[]);
+        let change_dir = tmp.path().join("cclab/changes/wf-fail");
+
+        // Set task_revisions to exceed MAX_SPEC_REVISIONS
+        std::fs::write(
+            change_dir.join("STATE.yaml"),
+            format!(
+                "change_id: wf-fail\nphase: change_implementation_reviewed\niteration: 1\ntask_revisions:\n  spec-a: {}\n",
+                MAX_SPEC_REVISIONS
+            ),
+        )
+        .unwrap();
+
+        // Write impl with REVISE verdict to trigger TerminalFailure
+        let mut content = String::from(
+            "---\nid: impl\ntype: change_implementation\n---\n# Implementation\n\n## Diff\n\n```diff\n+code\n```\n\n",
+        );
+        content.push_str("## Review: spec-a\n\nverdict: REVISE\nsummary: needs work\n\n");
+        std::fs::write(change_dir.join("implementation.md"), content).unwrap();
+
+        let args = json!({
+            "project_path": tmp.path().to_str().unwrap(),
+            "change_id": "wf-fail"
+        });
+        let result = execute_workflow(&args, tmp.path()).unwrap();
+        let parsed: Value = serde_json::from_str(&result).unwrap();
+        assert_eq!(parsed["status"], "error");
+        let next = parsed["next_actions"].as_array().unwrap();
+        assert!(!next.is_empty(), "TerminalFailure should provide retry next_actions");
+        assert_eq!(next[0]["tool"], "sdd_run_change");
+        assert_eq!(next[0]["args"]["advance_to"], "change_implementation_created");
+        assert_eq!(next[0]["when"], "optional");
+    }
 }
diff --git a/crates/cclab-sdd/src/mcp/tools/change_spec/common.rs b/crates/cclab-sdd/src/mcp/tools/change_spec/common.rs
index 4d23970..260d6a7 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_spec/common.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_spec/common.rs
@@ -689,6 +689,8 @@ pub fn read_main_spec_ref(content: &str) -> Option<String> {
             if val == "~" || val.is_empty() {
                 return None;
             }
+            // Strip YAML quotes (single or double) from the value
+            let val = val.trim_matches('"').trim_matches('\'');
             return Some(val.to_string());
         }
     }
@@ -877,6 +879,30 @@ Old scenarios.
         assert_eq!(sections, vec!["overview", "requirements"]);
     }
 
+    #[test]
+    fn test_read_main_spec_ref_unquoted() {
+        let content = "---\nid: test\nmain_spec_ref: foo/bar.md\n---\n# Body\n";
+        assert_eq!(read_main_spec_ref(content), Some("foo/bar.md".to_string()));
+    }
+
+    #[test]
+    fn test_read_main_spec_ref_double_quoted() {
+        let content = "---\nid: test\nmain_spec_ref: \"foo/bar.md\"\n---\n# Body\n";
+        assert_eq!(read_main_spec_ref(content), Some("foo/bar.md".to_string()));
+    }
+
+    #[test]
+    fn test_read_main_spec_ref_single_quoted() {
+        let content = "---\nid: test\nmain_spec_ref: 'foo/bar.md'\n---\n# Body\n";
+        assert_eq!(read_main_spec_ref(content), Some("foo/bar.md".to_string()));
+    }
+
+    #[test]
+    fn test_read_main_spec_ref_null() {
+        let content = "---\nid: test\nmain_spec_ref: ~\n---\n# Body\n";
+        assert_eq!(read_main_spec_ref(content), None);
+    }
+
     #[test]
     fn test_is_create_complete() {
         let content = "---\nid: test\ncreate_complete: true\n---\n\n# Body\n";
diff --git a/crates/cclab-sdd/src/workflow/advance.rs b/crates/cclab-sdd/src/workflow/advance.rs
index aad1134..93f02e5 100644
--- a/crates/cclab-sdd/src/workflow/advance.rs
+++ b/crates/cclab-sdd/src/workflow/advance.rs
@@ -59,6 +59,15 @@ pub fn apply_advance_to(
 
     state_manager.save()?;
 
+    // Reset task_revisions when re-entering ChangeImplementationCreated
+    // (allows retry after TerminalFailure)
+    if target_phase == StatePhase::ChangeImplementationCreated
+        && current_phase == StatePhase::ChangeImplementationCreated
+    {
+        state_manager.state_mut().task_revisions.clear();
+        state_manager.save()?;
+    }
+
     // Auto-capture git diff when advancing to ChangeImplementationCreated
     if target_phase == StatePhase::ChangeImplementationCreated {
         capture_impl_diff(change_dir, &state_manager);
```
