# Implementation Diff

## Summary

```
crates/cclab-sdd/src/mcp/tools/agent.rs            |  23 +++-
 .../cclab-sdd/src/mcp/tools/change_impl/create.rs  |  72 +++++++++----
 .../cclab-sdd/src/mcp/tools/change_impl/review.rs  |  34 ++++--
 .../cclab-sdd/src/mcp/tools/change_impl/revise.rs  |  19 ++--
 .../cclab-sdd/src/mcp/tools/change_merge/create.rs |  37 ++++---
 .../cclab-sdd/src/mcp/tools/change_spec/create.rs  | 116 +++++++++++++++------
 .../cclab-sdd/src/mcp/tools/change_spec/review.rs  |  38 ++++---
 .../cclab-sdd/src/mcp/tools/change_spec/revise.rs  |  18 ++--
 .../src/mcp/tools/create_pre_clarifications.rs     |  30 ++++--
 .../src/mcp/tools/post_clarifications/create.rs    |  38 ++++---
 .../src/mcp/tools/reference_context/create.rs      |  37 ++++---
 .../src/mcp/tools/reference_context/review.rs      |  38 ++++---
 .../src/mcp/tools/reference_context/revise.rs      |  38 ++++---
 .../cclab-sdd/src/mcp/tools/restructure_input.rs   |  55 +++++-----
 crates/cclab-sdd/src/mcp/tools/workflow_common.rs  |  71 ++++++++++++-
 15 files changed, 463 insertions(+), 201 deletions(-)
```

## Diff

```diff
diff --git a/crates/cclab-sdd/src/mcp/tools/agent.rs b/crates/cclab-sdd/src/mcp/tools/agent.rs
index 64bfa1a..782ea36 100644
--- a/crates/cclab-sdd/src/mcp/tools/agent.rs
+++ b/crates/cclab-sdd/src/mcp/tools/agent.rs
@@ -31,7 +31,7 @@ pub fn definition() -> ToolDefinition {
             Returns artifact-oriented responses with verification status.".to_string(),
         input_schema: json!({
             "type": "object",
-            "required": ["project_path", "agent", "action", "prompt"],
+            "required": ["project_path", "agent", "action"],
             "properties": {
                 "project_path": {
                     "type": "string",
@@ -54,13 +54,18 @@ pub fn definition() -> ToolDefinition {
                         "create_change_spec", "review_change_spec", "revise_change_spec",
                         "begin_implementation", "implement_spec", "implement_spec_with_codegen", "write_implementation_diff",
                         "review_spec", "revise_spec", "spec_terminal_failure",
-                        "begin_merge", "resume_merge", "review_merge", "fix_merge"
+                        "begin_merge", "resume_merge", "review_merge", "fix_merge",
+                        "restructure_input"
                     ],
                     "description": "Workflow action. 'explore'/'review' use templates; all others pass prompt as-is."
                 },
                 "prompt": {
                     "type": "string",
-                    "description": "The prompt to send to the agent"
+                    "description": "The prompt to send to the agent. Either prompt or prompt_path is required."
+                },
+                "prompt_path": {
+                    "type": "string",
+                    "description": "Relative path to a prompt file (under project root). Read as prompt content. Either prompt or prompt_path is required."
                 }
             }
         }),
@@ -396,9 +401,19 @@ pub async fn execute_streaming(
 ) -> Result<String> {
     let agent = get_required_string(args, "agent")?;
     let action = get_required_string(args, "action")?;
-    let prompt = get_required_string(args, "prompt")?;
     let change_id = get_optional_string(args, "change_id");
 
+    // Resolve prompt: either inline or from file
+    let prompt = if let Some(p) = get_optional_string(args, "prompt") {
+        p
+    } else if let Some(pp) = get_optional_string(args, "prompt_path") {
+        let full_path = project_root.join(&pp);
+        std::fs::read_to_string(&full_path)
+            .map_err(|e| anyhow::anyhow!("Failed to read prompt_path '{}': {}", pp, e))?
+    } else {
+        anyhow::bail!("Either 'prompt' or 'prompt_path' is required");
+    };
+
     // Validate: workflow actions require change_id
     if is_workflow_action(&action) && change_id.is_none() {
         anyhow::bail!(
diff --git a/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs b/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs
index 3bcb5ee..30bdd05 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_impl/create.rs
@@ -6,6 +6,7 @@
 use super::common::{self, ImplSubState, MAX_SPEC_REVISIONS};
 use crate::mcp::tools::workflow_common;
 use crate::mcp::tools::{get_required_string, ToolDefinition};
+use crate::models::WorkflowArtifact;
 use crate::state::StateManager;
 use crate::Result;
 use serde_json::{json, Value};
@@ -267,13 +268,18 @@ fn build_implement_prompt(
         cid = change_id, sid = spec_id
     );
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "spec_id": spec_id,
-        "next_actions": []
-    });
-    Ok(serde_json::to_string_pretty(&result)?)
+    let action = if is_first { "begin_implementation" } else { "implement_spec" };
+    let change_dir = project_root.join("cclab/changes").join(change_id);
+    let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeImplementation);
+
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        action,
+        prompt,
+        executor,
+        json!({ "spec_id": spec_id }),
+    )
 }
 
 fn build_codegen_prompt(
@@ -304,14 +310,17 @@ fn build_codegen_prompt(
          ```"
     );
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "spec_id": spec_id,
-        "codegen": true,
-        "next_actions": []
-    });
-    Ok(serde_json::to_string_pretty(&result)?)
+    let change_dir = project_root.join("cclab/changes").join(change_id);
+    let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeImplementation);
+
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        "implement_spec_with_codegen",
+        prompt,
+        executor,
+        json!({ "spec_id": spec_id, "codegen": true }),
+    )
 }
 
 fn build_write_diff_prompt(
@@ -333,12 +342,17 @@ fn build_write_diff_prompt(
          ```"
     );
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "next_actions": []
-    });
-    Ok(serde_json::to_string_pretty(&result)?)
+    let change_dir = project_root.join("cclab/changes").join(change_id);
+    let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeImplementation);
+
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        "write_implementation_diff",
+        prompt,
+        executor,
+        json!({}),
+    )
 }
 
 // ─── Tests ───────────────────────────────────────────────────────────────────
@@ -376,11 +390,21 @@ mod tests {
         .unwrap();
     }
 
+    fn read_prompt(parsed: &Value, change_dir: &std::path::Path, action: &str) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
+    }
+
     #[test]
     fn test_workflow_begin_implementation() {
         let tmp = setup_change("wf-impl", "change_implementation_created");
         write_spec(&tmp, "wf-impl", "spec-a", &[]);
 
+        let change_dir = tmp.path().join("cclab/changes/wf-impl");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "wf-impl"
@@ -389,7 +413,8 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         assert_eq!(parsed["spec_id"], "spec-a");
-        assert!(parsed["prompt"].as_str().unwrap().contains("Begin Implementation"));
+        let prompt = read_prompt(&parsed, &change_dir, "begin_implementation");
+        assert!(prompt.contains("Begin Implementation"));
     }
 
     #[test]
@@ -411,7 +436,8 @@ mod tests {
         let result = execute_workflow(&args, tmp.path()).unwrap();
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
-        assert!(parsed["prompt"].as_str().unwrap().contains("Write Implementation Diff"));
+        let prompt = read_prompt(&parsed, &change_dir, "write_implementation_diff");
+        assert!(prompt.contains("Write Implementation Diff"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/change_impl/review.rs b/crates/cclab-sdd/src/mcp/tools/change_impl/review.rs
index 84ca4b9..bd4e705 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_impl/review.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_impl/review.rs
@@ -325,16 +325,17 @@ mcp__cclab-mcp__sdd_artifact_review_change_implementation(project_path="{pp}", c
 ```"#,
     );
 
+    let change_dir = project_root.join("cclab/changes").join(change_id);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewChangeImplementation);
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "executor": executor,
-        "spec_id": spec_id,
-        "next_actions": []
-    });
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        &format!("review_impl_{}", spec_id),
+        prompt,
+        executor,
+        json!({ "spec_id": spec_id }),
+    )
 }
 
 // ─── Tests ───────────────────────────────────────────────────────────────────
@@ -371,9 +372,23 @@ mod tests {
         tmp
     }
 
+    /// Read prompt content from either inline response or prompt file.
+    fn read_prompt(parsed: &Value, change_dir: &std::path::Path) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        let action = parsed["next_actions"][0]["args"]["action"]
+            .as_str()
+            .unwrap_or("unknown");
+        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
+    }
+
     #[test]
     fn test_review_workflow_returns_prompt() {
         let tmp = setup_review_change("rev-wf");
+        let change_dir = tmp.path().join("cclab/changes/rev-wf");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "rev-wf"
@@ -382,7 +397,8 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         assert_eq!(parsed["spec_id"], "spec-a");
-        assert!(parsed["prompt"].as_str().unwrap().contains("Review Implementation"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Review Implementation"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/change_impl/revise.rs b/crates/cclab-sdd/src/mcp/tools/change_impl/revise.rs
index c43e832..04d49e0 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_impl/revise.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_impl/revise.rs
@@ -7,6 +7,7 @@ use super::common;
 use super::create;
 use crate::mcp::tools::workflow_common;
 use crate::mcp::tools::{get_required_string, ToolDefinition};
+use crate::models::WorkflowArtifact;
 use crate::Result;
 use serde_json::{json, Value};
 use std::path::Path;
@@ -137,11 +138,15 @@ fn build_revise_prompt(
          ```"
     );
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "spec_id": spec_id,
-        "next_actions": []
-    });
-    Ok(serde_json::to_string_pretty(&result)?)
+    let change_dir = project_root.join("cclab/changes").join(change_id);
+    let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseChangeImplementation);
+
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        "revise_change_implementation",
+        prompt,
+        executor,
+        json!({ "spec_id": spec_id }),
+    )
 }
diff --git a/crates/cclab-sdd/src/mcp/tools/change_merge/create.rs b/crates/cclab-sdd/src/mcp/tools/change_merge/create.rs
index 3e280c1..4c1ee51 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_merge/create.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_merge/create.rs
@@ -13,6 +13,7 @@ use crate::mcp::tools::change_spec::common;
 use crate::mcp::tools::workflow_common;
 use crate::mcp::tools::{get_required_string, ToolDefinition};
 use crate::models::state::StatePhase;
+use crate::models::WorkflowArtifact;
 use crate::workflow::helpers;
 use crate::Result;
 use serde_json::{json, Value};
@@ -125,19 +126,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
 
     let archive_path = build_archive_path(&change_id);
 
-    let mut result = json!({
-        "status": "ok",
-        "message": format!("Merged {} spec(s) to main. Change archived.", merged.len()),
-        "specs_merged": merged,
-        "archive_path": &archive_path,
-    });
-
-    if !errors.is_empty() {
-        result["warnings"] = json!(errors);
-    }
-
-    // Return archive instructions
-    result["prompt"] = json!(format!(
+    let prompt = format!(
         "# Merge Complete for Change '{}'\n\n\
          {} spec(s) merged to main specs directory.\n\n\
          Archive the change directory:\n\
@@ -151,10 +140,26 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
         archive_path,
         change_id,
         archive_path,
-    ));
-    result["next_actions"] = json!([]);
+    );
 
-    Ok(serde_json::to_string_pretty(&result)?)
+    let mut extra_fields = json!({
+        "specs_merged": merged,
+        "archive_path": &archive_path,
+    });
+    if !errors.is_empty() {
+        extra_fields["warnings"] = json!(errors);
+    }
+
+    let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeMerge);
+
+    workflow_common::build_workflow_response(
+        &change_dir,
+        &change_id,
+        "create_change_merge",
+        prompt,
+        executor,
+        extra_fields,
+    )
 }
 
 /// Build archive path for a change.
diff --git a/crates/cclab-sdd/src/mcp/tools/change_spec/create.rs b/crates/cclab-sdd/src/mcp/tools/change_spec/create.rs
index 8ad2d8f..d78f308 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_spec/create.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_spec/create.rs
@@ -9,7 +9,7 @@
 use super::common::{self, SpecSubState};
 use crate::mcp::tools::review_helpers;
 use crate::mcp::tools::workflow_common;
-use crate::mcp::tools::{get_required_string, ToolDefinition};
+use crate::mcp::tools::{get_optional_string, get_required_string, ToolDefinition};
 use crate::models::state::StatePhase;
 use crate::models::WorkflowArtifact;
 use crate::workflow::helpers;
@@ -76,6 +76,20 @@ pub fn artifact_definition() -> ToolDefinition {
                 "content": {
                     "type": "string",
                     "description": "Markdown content for this section (everything after the H2 heading)"
+                },
+                "fill_sections": {
+                    "type": "array",
+                    "items": { "type": "string" },
+                    "description": "Sections to fill (set during analyze). Persisted to frontmatter."
+                },
+                "main_spec_ref": {
+                    "type": "string",
+                    "description": "Target path in cclab/specs/ for merge (e.g. cclab-sdd/tools/foo.md)"
+                },
+                "merge_strategy": {
+                    "type": "string",
+                    "enum": ["new", "append", "replace"],
+                    "description": "How to merge into main spec. Default: new"
                 }
             }
         }),
@@ -314,9 +328,32 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
         filled.push(section.clone());
     }
     let filled_str = format!("[{}]", filled.join(", "));
-    let final_content =
+    let mut final_content =
         review_helpers::upsert_frontmatter_field(&updated, "filled_sections", &filled_str);
 
+    // Persist fill_sections if provided (from analyze step)
+    if let Some(fs) = args.get("fill_sections").and_then(|v| v.as_array()) {
+        let fs_list: Vec<String> = fs.iter().filter_map(|v| v.as_str().map(String::from)).collect();
+        if !fs_list.is_empty() {
+            let fs_str = format!("[{}]", fs_list.join(", "));
+            final_content = review_helpers::upsert_frontmatter_field(&final_content, "fill_sections", &fs_str);
+        }
+    }
+
+    // Persist main_spec_ref if provided
+    if let Some(ref_path) = get_optional_string(args, "main_spec_ref") {
+        final_content = review_helpers::upsert_frontmatter_field(
+            &final_content, "main_spec_ref", &format!("\"{}\"", ref_path),
+        );
+    }
+
+    // Persist merge_strategy if provided
+    if let Some(strategy) = get_optional_string(args, "merge_strategy") {
+        final_content = review_helpers::upsert_frontmatter_field(
+            &final_content, "merge_strategy", &strategy,
+        );
+    }
+
     std::fs::write(&spec_path, &final_content)?;
 
     let artifacts_written = vec![format!("specs/{}.md", spec_id)];
@@ -367,10 +404,11 @@ A skeleton has been generated at `specs/{spec_id}.md`.
    - `sdd_read_artifact(scope="read_path:changes/{change_id}/reference_context.md")` if no proposal
 2. Read the skeleton: `sdd_read_artifact(scope="read_path:changes/{change_id}/specs/{spec_id}.md")`
 3. **IMPORTANT — `main_spec_ref`**: Check the spec frontmatter. If `main_spec_ref` is `~` (null),
-   you MUST set it to the target path in `cclab/specs/` where this spec will be merged.
+   you MUST determine the target path in `cclab/specs/` where this spec will be merged.
    Format: `<scope>/<category>/<spec-id>.md` (e.g., `cclab-sdd/tools/new-feature.md`).
    Use `sdd_read_artifact(scope="read_path:specs")` to see existing spec groups.
-   Update `main_spec_ref` in the frontmatter via the artifact tool.
+   Pass it as the `main_spec_ref` parameter when calling the artifact tool.
+   Also pass `merge_strategy` ("new", "append", or "replace") as a parameter.
 4. Decide which sections to fill based on the nature of the change:
    - **overview** — always fill
    - **requirements** — always fill
@@ -384,11 +422,12 @@ A skeleton has been generated at `specs/{spec_id}.md`.
 
 ## Expected Action
 
-Call the artifact tool to write the **overview** section first. The overview should include
-your analysis of which sections need filling. Include a `fill_sections` list in the frontmatter
-by writing it into the overview content — the system will track it.
+Call the artifact tool to write the **overview** section first. Pass the `fill_sections`
+array as a parameter (e.g., `fill_sections=["overview", "requirements", "scenarios"]`).
+Also pass `main_spec_ref` and `merge_strategy` as parameters if determined above.
+The system persists these to frontmatter automatically.
 
-Actually, call the artifact tool for each section in sequence. Start with `overview`.
+Then call the artifact tool for each remaining section in sequence.
 
 ## MCP Tools
 
@@ -396,22 +435,22 @@ Actually, call the artifact tool for each section in sequence. Start with `overv
 mcp__cclab-mcp__sdd_read_artifact(project_path="{pp}", scope="read_path:changes/{change_id}/proposal.md")
 mcp__cclab-mcp__sdd_read_artifact(project_path="{pp}", scope="read_path:changes/{change_id}/specs/{spec_id}.md")
 mcp__cclab-mcp__sdd_read_artifact(project_path="{pp}", scope="read_path:specs")
-mcp__cclab-mcp__sdd_artifact_create_change_spec(project_path="{pp}", change_id="{change_id}", spec_id="{spec_id}", section="overview", content="...")
+mcp__cclab-mcp__sdd_artifact_create_change_spec(project_path="{pp}", change_id="{change_id}", spec_id="{spec_id}", section="overview", content="...", fill_sections=["overview", "requirements", "scenarios"], main_spec_ref="cclab-sdd/tools/example.md", merge_strategy="new")
 ```
 {depends_note}"#,
     );
 
+    let change_dir = project_root.join("cclab/changes").join(change_id);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeSpec);
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "executor": executor,
-        "spec_id": spec_id,
-        "next_actions": []
-    });
-
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        &format!("analyze_spec_{}", spec_id),
+        prompt,
+        executor,
+        json!({ "spec_id": spec_id }),
+    )
 }
 
 /// Build FILL prompt for a specific section.
@@ -467,17 +506,17 @@ mcp__cclab-mcp__sdd_artifact_create_change_spec(project_path="{pp}", change_id="
 ```"#,
     );
 
+    let change_dir = project_root.join("cclab/changes").join(change_id);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeSpec);
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "executor": executor,
-        "spec_id": spec_id,
-        "next_actions": []
-    });
-
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        &format!("fill_spec_{}_{}", spec_id, section),
+        prompt,
+        executor,
+        json!({ "spec_id": spec_id }),
+    )
 }
 
 #[cfg(test)]
@@ -486,6 +525,19 @@ mod tests {
     use crate::state::StateManager;
     use tempfile::TempDir;
 
+    /// Read prompt content from either inline response or prompt file.
+    fn read_prompt(parsed: &Value, change_dir: &std::path::Path) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        let action = parsed["next_actions"][0]["args"]["action"]
+            .as_str()
+            .unwrap_or("unknown");
+        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
+    }
+
     fn setup_change(change_id: &str) -> TempDir {
         let tmp = TempDir::new().unwrap();
         let change_dir = tmp.path().join("cclab/changes").join(change_id);
@@ -502,6 +554,7 @@ mod tests {
     #[test]
     fn test_workflow_creates_skeleton() {
         let tmp = setup_change("skel-test");
+        let change_dir = tmp.path().join("cclab/changes/skel-test");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "skel-test"
@@ -510,7 +563,8 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         assert_eq!(parsed["spec_id"], "skel-test-spec");
-        assert!(parsed["prompt"].as_str().unwrap().contains("Analyze Spec"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Analyze Spec"));
 
         // Verify skeleton file created
         let spec_path = tmp
@@ -550,10 +604,8 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         // Should return fill prompt for first section (overview)
-        assert!(parsed["prompt"]
-            .as_str()
-            .unwrap()
-            .contains("Fill Section 'overview'"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Fill Section 'overview'"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/change_spec/review.rs b/crates/cclab-sdd/src/mcp/tools/change_spec/review.rs
index 0123d8e..2e42880 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_spec/review.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_spec/review.rs
@@ -332,17 +332,17 @@ mcp__cclab-mcp__sdd_artifact_review_change_spec(project_path="{pp}", change_id="
 ```"#,
     );
 
+    let change_dir = project_root.join("cclab/changes").join(change_id);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewChangeSpec);
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "executor": executor,
-        "spec_id": spec_id,
-        "next_actions": []
-    });
-
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        &format!("review_spec_{}", spec_id),
+        prompt,
+        executor,
+        json!({ "spec_id": spec_id }),
+    )
 }
 
 #[cfg(test)]
@@ -386,9 +386,23 @@ mod tests {
         tmp
     }
 
+    /// Read prompt content from either inline response or prompt file.
+    fn read_prompt(parsed: &Value, change_dir: &std::path::Path) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        let action = parsed["next_actions"][0]["args"]["action"]
+            .as_str()
+            .unwrap_or("unknown");
+        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
+    }
+
     #[test]
     fn test_review_workflow_returns_prompt() {
         let tmp = setup_review_change("rev-wf");
+        let change_dir = tmp.path().join("cclab/changes/rev-wf");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "rev-wf"
@@ -397,10 +411,8 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         assert_eq!(parsed["spec_id"], "rev-wf-spec");
-        assert!(parsed["prompt"]
-            .as_str()
-            .unwrap()
-            .contains("Review Spec"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Review Spec"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/change_spec/revise.rs b/crates/cclab-sdd/src/mcp/tools/change_spec/revise.rs
index f9190cb..7e559a7 100644
--- a/crates/cclab-sdd/src/mcp/tools/change_spec/revise.rs
+++ b/crates/cclab-sdd/src/mcp/tools/change_spec/revise.rs
@@ -261,15 +261,15 @@ mcp__cclab-mcp__sdd_artifact_revise_change_spec(project_path="{pp}", change_id="
 ```"#,
     );
 
+    let change_dir = project_root.join("cclab/changes").join(change_id);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseChangeSpec);
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "executor": executor,
-        "spec_id": spec_id,
-        "next_actions": []
-    });
-
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        &format!("revise_spec_{}", spec_id),
+        prompt,
+        executor,
+        json!({ "spec_id": spec_id }),
+    )
 }
diff --git a/crates/cclab-sdd/src/mcp/tools/create_pre_clarifications.rs b/crates/cclab-sdd/src/mcp/tools/create_pre_clarifications.rs
index d6cd386..a2370a8 100644
--- a/crates/cclab-sdd/src/mcp/tools/create_pre_clarifications.rs
+++ b/crates/cclab-sdd/src/mcp/tools/create_pre_clarifications.rs
@@ -1,6 +1,7 @@
 use super::{get_required_array, get_required_string, ToolDefinition};
 use crate::mcp::tools::workflow_common;
 use crate::models::state::StatePhase;
+use crate::models::WorkflowArtifact;
 use crate::services::pre_clarifications_service::{
     append_clarifications as service_append, AppendClarificationsInput, QuestionAnswer,
 };
@@ -419,14 +420,16 @@ mcp__cclab-mcp__sdd_artifact_create_pre_clarifications(project_path="{project_pa
         },
     );
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "group_id": group_id,
-        "next_actions": []
-    });
+    let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreatePreClarifications);
 
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        &change_dir,
+        &change_id,
+        "create_pre_clarifications",
+        prompt,
+        executor,
+        json!({ "group_id": group_id }),
+    )
 }
 
 /// Execute sdd_artifact_create_pre_clarifications.
@@ -714,9 +717,19 @@ mod tests {
         tmp
     }
 
+    fn read_prompt(parsed: &Value, change_dir: &std::path::Path) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        let prompt_path = change_dir.join("prompts/create_pre_clarifications.md");
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
+    }
+
     #[test]
     fn test_workflow_pre_clarifications_returns_prompt() {
         let tmp = setup_group_change("wf-test");
+        let change_dir = tmp.path().join("cclab/changes/wf-test");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "wf-test"
@@ -725,7 +738,8 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         assert_eq!(parsed["group_id"], "my-group");
-        assert!(parsed["prompt"].as_str().unwrap().contains("Clarify Group"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Clarify Group"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/post_clarifications/create.rs b/crates/cclab-sdd/src/mcp/tools/post_clarifications/create.rs
index 5ee2c75..6734fa0 100644
--- a/crates/cclab-sdd/src/mcp/tools/post_clarifications/create.rs
+++ b/crates/cclab-sdd/src/mcp/tools/post_clarifications/create.rs
@@ -5,6 +5,7 @@
 
 use crate::mcp::tools::workflow_common;
 use crate::mcp::tools::{get_required_string, ToolDefinition};
+use crate::models::WorkflowArtifact;
 use crate::state::StateManager;
 use crate::workflow::scope;
 use crate::Result;
@@ -338,14 +339,16 @@ mcp__cclab-mcp__sdd_artifact_create_post_clarifications(project_path="{project_p
         },
     );
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "group_id": group_id,
-        "next_actions": []
-    });
+    let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreatePostClarifications);
 
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        change_dir,
+        change_id,
+        "create_post_clarifications",
+        prompt,
+        executor,
+        json!({ "group_id": group_id }),
+    )
 }
 
 // ─── Rendering ───────────────────────────────────────────────────────────────
@@ -454,9 +457,19 @@ mod tests {
         tmp
     }
 
+    fn read_prompt(parsed: &Value, change_dir: &std::path::Path) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        let prompt_path = change_dir.join("prompts/create_post_clarifications.md");
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
+    }
+
     #[test]
     fn test_workflow_returns_create_prompt() {
         let tmp = setup_group_change("post-test");
+        let change_dir = tmp.path().join("cclab/changes/post-test");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "post-test"
@@ -465,14 +478,9 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         assert_eq!(parsed["group_id"], "my-group");
-        assert!(parsed["prompt"]
-            .as_str()
-            .unwrap()
-            .contains("Post-Clarification"));
-        assert!(parsed["prompt"]
-            .as_str()
-            .unwrap()
-            .contains("Contradiction Mining"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Post-Clarification"));
+        assert!(prompt.contains("Contradiction Mining"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/reference_context/create.rs b/crates/cclab-sdd/src/mcp/tools/reference_context/create.rs
index 95907e2..d685a75 100644
--- a/crates/cclab-sdd/src/mcp/tools/reference_context/create.rs
+++ b/crates/cclab-sdd/src/mcp/tools/reference_context/create.rs
@@ -290,15 +290,14 @@ mcp__cclab-mcp__sdd_artifact_create_reference_context(project_path="{project_pat
 
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateReferenceContext);
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "executor": executor,
-        "group_id": group_id,
-        "next_actions": []
-    });
-
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        change_dir,
+        change_id,
+        "create_reference_context",
+        prompt,
+        executor,
+        json!({ "group_id": group_id }),
+    )
 }
 
 #[cfg(test)]
@@ -331,9 +330,23 @@ mod tests {
         tmp
     }
 
+    /// Read prompt content from either inline response or prompt file.
+    fn read_prompt(parsed: &Value, change_dir: &Path) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        let action = parsed["next_actions"][0]["args"]["action"]
+            .as_str()
+            .unwrap_or("create_reference_context");
+        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No inline prompt and no prompt file at {:?}", prompt_path))
+    }
+
     #[test]
     fn test_workflow_returns_create_prompt() {
         let tmp = setup_group_change("ref-test");
+        let change_dir = tmp.path().join("cclab/changes/ref-test");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "ref-test"
@@ -342,10 +355,8 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         assert_eq!(parsed["group_id"], "my-group");
-        assert!(parsed["prompt"]
-            .as_str()
-            .unwrap()
-            .contains("Gather Reference Context"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Gather Reference Context"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/reference_context/review.rs b/crates/cclab-sdd/src/mcp/tools/reference_context/review.rs
index 921b500..60f0478 100644
--- a/crates/cclab-sdd/src/mcp/tools/reference_context/review.rs
+++ b/crates/cclab-sdd/src/mcp/tools/reference_context/review.rs
@@ -277,17 +277,17 @@ mcp__cclab-mcp__sdd_artifact_review_reference_context(project_path="{project_pat
 ```"#,
     );
 
+    let change_dir = project_root.join("cclab/changes").join(change_id);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewReferenceContext);
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "executor": executor,
-        "group_id": group_id,
-        "next_actions": []
-    });
-
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        "review_reference_context",
+        prompt,
+        executor,
+        json!({ "group_id": group_id }),
+    )
 }
 
 #[cfg(test)]
@@ -318,9 +318,23 @@ mod tests {
         tmp
     }
 
+    /// Read prompt content from either inline response or prompt file.
+    fn read_prompt(parsed: &Value, change_dir: &std::path::Path) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        let action = parsed["next_actions"][0]["args"]["action"]
+            .as_str()
+            .unwrap_or("review_reference_context");
+        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
+    }
+
     #[test]
     fn test_review_workflow_returns_prompt() {
         let tmp = setup_review_change("rev-wf");
+        let change_dir = tmp.path().join("cclab/changes/rev-wf");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "rev-wf"
@@ -329,10 +343,8 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         assert_eq!(parsed["group_id"], "my-group");
-        assert!(parsed["prompt"]
-            .as_str()
-            .unwrap()
-            .contains("Review Reference Context"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Review Reference Context"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/reference_context/revise.rs b/crates/cclab-sdd/src/mcp/tools/reference_context/revise.rs
index b749aad..f3f321c 100644
--- a/crates/cclab-sdd/src/mcp/tools/reference_context/revise.rs
+++ b/crates/cclab-sdd/src/mcp/tools/reference_context/revise.rs
@@ -171,17 +171,17 @@ mcp__cclab-mcp__sdd_artifact_revise_reference_context(project_path="{project_pat
 ```"#,
     );
 
+    let change_dir = project_root.join("cclab/changes").join(change_id);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseReferenceContext);
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "executor": executor,
-        "group_id": group_id,
-        "next_actions": []
-    });
-
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        &change_dir,
+        change_id,
+        "revise_reference_context",
+        prompt,
+        executor,
+        json!({ "group_id": group_id }),
+    )
 }
 
 #[cfg(test)]
@@ -212,9 +212,23 @@ mod tests {
         tmp
     }
 
+    /// Read prompt content from either inline response or prompt file.
+    fn read_prompt(parsed: &Value, change_dir: &std::path::Path) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        let action = parsed["next_actions"][0]["args"]["action"]
+            .as_str()
+            .unwrap_or("revise_reference_context");
+        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
+    }
+
     #[test]
     fn test_revise_workflow_returns_prompt() {
         let tmp = setup_revise_change("rev-wf");
+        let change_dir = tmp.path().join("cclab/changes/rev-wf");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "rev-wf"
@@ -223,10 +237,8 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
         assert_eq!(parsed["group_id"], "my-group");
-        assert!(parsed["prompt"]
-            .as_str()
-            .unwrap()
-            .contains("Revise Reference Context"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Revise Reference Context"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/restructure_input.rs b/crates/cclab-sdd/src/mcp/tools/restructure_input.rs
index 20a9f58..0daaf61 100644
--- a/crates/cclab-sdd/src/mcp/tools/restructure_input.rs
+++ b/crates/cclab-sdd/src/mcp/tools/restructure_input.rs
@@ -226,27 +226,14 @@ mcp__cclab-mcp__sdd_artifact_restructure_input(project_path="{project_path}", ch
 
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::RestructureInput);
 
-    let result = json!({
-        "status": "ok",
-        "prompt": prompt,
-        "executor": executor,
-        "next_actions": [
-            {
-                "action": "read",
-                "files": read_files
-            },
-            {
-                "tool": "sdd_artifact_restructure_input",
-                "args": { "change_id": &change_id }
-            },
-            {
-                "tool": "sdd_workflow_create_pre_clarifications",
-                "args": { "change_id": &change_id }
-            }
-        ]
-    });
-
-    Ok(serde_json::to_string_pretty(&result)?)
+    workflow_common::build_workflow_response(
+        &change_dir,
+        &change_id,
+        "restructure_input",
+        prompt,
+        executor,
+        json!({}),
+    )
 }
 
 /// Execute sdd_artifact_restructure_input — writes groups + requirements + question stubs.
@@ -352,9 +339,24 @@ mod tests {
         tmp
     }
 
+    /// Read prompt content from either inline response or prompt file.
+    fn read_prompt(parsed: &Value, change_dir: &Path) -> String {
+        if let Some(p) = parsed["prompt"].as_str() {
+            return p.to_string();
+        }
+        // Prompt written to file — read from prompts/ dir
+        let action = parsed["next_actions"][0]["args"]["action"]
+            .as_str()
+            .unwrap_or("restructure_input");
+        let prompt_path = change_dir.join("prompts").join(format!("{}.md", action));
+        std::fs::read_to_string(&prompt_path)
+            .unwrap_or_else(|_| panic!("No inline prompt and no prompt file at {:?}", prompt_path))
+    }
+
     #[test]
     fn test_workflow_returns_prompt_with_issues() {
         let tmp = setup_project_with_issues("test-restructure");
+        let change_dir = tmp.path().join("cclab/changes/test-restructure");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "test-restructure"
@@ -362,13 +364,15 @@ mod tests {
         let result = execute_workflow(&args, tmp.path()).unwrap();
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
-        assert!(parsed["prompt"].as_str().unwrap().contains("Restructure Input"));
-        assert!(parsed["prompt"].as_str().unwrap().contains("Read Issues"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Restructure Input"));
+        assert!(prompt.contains("Read Issues"));
     }
 
     #[test]
     fn test_workflow_returns_prompt_no_issues() {
         let tmp = setup_project_no_issues("test-restructure-no");
+        let change_dir = tmp.path().join("cclab/changes/test-restructure-no");
         let args = json!({
             "project_path": tmp.path().to_str().unwrap(),
             "change_id": "test-restructure-no"
@@ -376,8 +380,9 @@ mod tests {
         let result = execute_workflow(&args, tmp.path()).unwrap();
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "ok");
-        assert!(parsed["prompt"].as_str().unwrap().contains("Restructure Input"));
-        assert!(parsed["prompt"].as_str().unwrap().contains("Read Input"));
+        let prompt = read_prompt(&parsed, &change_dir);
+        assert!(prompt.contains("Restructure Input"));
+        assert!(prompt.contains("Read Input"));
     }
 
     #[test]
diff --git a/crates/cclab-sdd/src/mcp/tools/workflow_common.rs b/crates/cclab-sdd/src/mcp/tools/workflow_common.rs
index 96e1747..006cfab 100644
--- a/crates/cclab-sdd/src/mcp/tools/workflow_common.rs
+++ b/crates/cclab-sdd/src/mcp/tools/workflow_common.rs
@@ -7,7 +7,8 @@ use crate::models::state::StatePhase;
 use crate::models::{SddConfig, WorkflowArtifact};
 use crate::state::StateManager;
 use crate::Result;
-use std::path::Path;
+use serde_json::{json, Value};
+use std::path::{Path, PathBuf};
 
 /// Validate change_id format (security: prevent directory traversal)
 pub fn validate_change_id(change_id: &str) -> Result<()> {
@@ -82,6 +83,74 @@ pub fn list_group_ids(groups_dir: &Path) -> Result<Vec<String>> {
     Ok(ids)
 }
 
+/// Write a prompt to a file in the change's `prompts/` directory.
+///
+/// Returns the path to the written file.
+pub fn write_prompt_file(change_dir: &Path, action: &str, prompt: &str) -> Result<PathBuf> {
+    let prompts_dir = change_dir.join("prompts");
+    std::fs::create_dir_all(&prompts_dir)?;
+    let path = prompts_dir.join(format!("{}.md", action));
+    std::fs::write(&path, prompt)?;
+    Ok(path)
+}
+
+/// Build a workflow response. Always writes prompt to file for clean tool responses.
+///
+/// - Prompt is always written to `prompts/{action}.md` (no inline prompt in response).
+/// - If executor is `["mainthread"]`: returns `prompt_path` + `executor` for mainthread to read.
+/// - Otherwise: returns `next_actions` with `sdd_delegate_agent` + `prompt_path`.
+///
+/// `extra_fields` are merged into the top-level response (e.g., `spec_id`, `group_id`).
+pub fn build_workflow_response(
+    change_dir: &Path,
+    change_id: &str,
+    action: &str,
+    prompt: String,
+    executor: Vec<String>,
+    extra_fields: Value,
+) -> Result<String> {
+    // Always write prompt to file
+    write_prompt_file(change_dir, action, &prompt)?;
+    let rel_path = format!("cclab/changes/{}/prompts/{}.md", change_id, action);
+
+    let is_mainthread_only = executor.len() == 1 && executor[0] == "mainthread";
+
+    let mut result = if is_mainthread_only {
+        json!({
+            "status": "ok",
+            "prompt_path": rel_path,
+            "executor": executor,
+            "next_actions": []
+        })
+    } else {
+        let agent = executor.first().cloned().unwrap_or_else(|| "mainthread".to_string());
+        json!({
+            "status": "ok",
+            "prompt_path": rel_path,
+            "next_actions": [{
+                "tool": "sdd_delegate_agent",
+                "args": {
+                    "change_id": change_id,
+                    "agent": agent,
+                    "action": action,
+                    "prompt_path": rel_path
+                },
+                "when": "immediate",
+                "executor": "mainthread"
+            }]
+        })
+    };
+
+    // Merge extra fields into top-level
+    if let Some(obj) = extra_fields.as_object() {
+        for (k, v) in obj {
+            result[k] = v.clone();
+        }
+    }
+
+    Ok(serde_json::to_string_pretty(&result)?)
+}
+
 /// Build a hint string listing issues belonging to a group (from frontmatter).
 pub fn build_group_issues_hint(change_dir: &Path, group_id: &str) -> String {
     let issues_dir = change_dir.join("issues");
```
