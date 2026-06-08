# Implementation Diff

## Summary

```
.claude/skills/cclab-codex-review/SKILL.md         |  46 +++++
 .../skills/cclab-gemini-explore-codebase/SKILL.md  |  43 +++++
 .claude/skills/cclab-gemini-explore-specs/SKILL.md |  43 +++++
 Cargo.lock                                         |  70 ++++----
 Cargo.toml                                         |   2 +-
 crates/cclab-sdd-cli/src/commands.rs               | 156 +++++++++++++++++
 crates/cclab-sdd-cli/src/init.rs                   |  50 +++++-
 crates/cclab-sdd-cli/templates/config.toml         | 164 +++---------------
 .../cclab-sdd-cli/templates/mainthread/CLAUDE.md   |   2 +-
 .../skills/cclab-sdd-run-change/SKILL.md           |  39 +++--
 crates/cclab-sdd-mcp/src/tools/agent.rs            |  18 +-
 .../cclab-sdd-mcp/src/tools/change_impl/create.rs  |  66 +++-----
 .../cclab-sdd-mcp/src/tools/change_impl/review.rs  |  19 +--
 .../cclab-sdd-mcp/src/tools/change_impl/revise.rs  |  11 +-
 .../cclab-sdd-mcp/src/tools/change_merge/create.rs |   2 +
 .../cclab-sdd-mcp/src/tools/change_spec/create.rs  |  62 +++----
 .../cclab-sdd-mcp/src/tools/change_spec/review.rs  |  19 +--
 .../cclab-sdd-mcp/src/tools/change_spec/revise.rs  |  19 +--
 .../src/tools/create_pre_clarifications.rs         |  25 +--
 crates/cclab-sdd-mcp/src/tools/init_change.rs      |  15 +-
 .../src/tools/post_clarifications/create.rs        |  26 ++-
 .../src/tools/reference_context/create.rs          |  41 ++---
 .../src/tools/reference_context/review.rs          |  19 +--
 .../src/tools/reference_context/revise.rs          |  15 +-
 .../cclab-sdd-mcp/src/tools/restructure_input.rs   |   2 +
 crates/cclab-sdd-mcp/src/tools/workflow_common.rs  |  38 +++--
 crates/cclab-sdd/src/defaults.rs                   |  15 +-
 crates/cclab-sdd/src/mcp/tools/agent.rs            |  18 +-
 .../cclab-sdd/src/mcp/tools/change_impl/create.rs  |  61 +++----
 .../cclab-sdd/src/mcp/tools/change_impl/review.rs  |  22 ++-
 .../cclab-sdd/src/mcp/tools/change_impl/revise.rs  |  12 +-
 .../cclab-sdd/src/mcp/tools/change_merge/create.rs |   2 +
 .../cclab-sdd/src/mcp/tools/change_spec/create.rs  |  68 ++++----
 .../cclab-sdd/src/mcp/tools/change_spec/review.rs  |  22 ++-
 .../cclab-sdd/src/mcp/tools/change_spec/revise.rs  |  22 ++-
 .../src/mcp/tools/create_pre_clarifications.rs     |  24 +--
 crates/cclab-sdd/src/mcp/tools/init_change.rs      |  11 +-
 .../src/mcp/tools/post_clarifications/create.rs    |  24 ++-
 .../src/mcp/tools/reference_context/create.rs      |  42 ++---
 .../src/mcp/tools/reference_context/review.rs      |  22 ++-
 .../src/mcp/tools/reference_context/revise.rs      |  14 +-
 .../cclab-sdd/src/mcp/tools/restructure_input.rs   |   2 +
 crates/cclab-sdd/src/mcp/tools/workflow_common.rs  |  38 +++--
 crates/cclab-sdd/src/models/change.rs              | 185 +++++++++++++++------
 crates/cclab-sdd/src/models/mod.rs                 |   3 +-
 .../cclab-sdd/src/orchestrator/model_selector.rs   |   4 +-
 crates/cclab-sdd/src/workflow/helpers.rs           |  99 ++++++++++-
 crates/cclab-sdd/src/workflow/mod.rs               |  64 +++----
 .../cclab-sdd/src/workflow/post_clarifications.rs  |  18 +-
 crates/cclab-sdd/templates/config.toml             | 164 +++---------------
 crates/cclab-sdd/templates/mainthread/CLAUDE.md    |   2 +-
 .../skills/cclab-sdd-run-change/SKILL.md           |  39 +++--
 52 files changed, 1109 insertions(+), 900 deletions(-)
```

## Diff

```diff
diff --git a/.claude/skills/cclab-codex-review/SKILL.md b/.claude/skills/cclab-codex-review/SKILL.md
new file mode 100644
index 0000000..6fda9f5
--- /dev/null
+++ b/.claude/skills/cclab-codex-review/SKILL.md
@@ -0,0 +1,46 @@
+---
+name: cclab:codex:review
+description: Run Codex headless to review code or changes
+user-invocable: true
+---
+
+# /cclab:codex:review
+
+Dispatches a review prompt to Codex CLI headlessly. Use this when you need a second opinion on code quality, security, correctness, or spec compliance.
+
+## Usage
+
+```
+/cclab:codex:review "<prompt>"
+```
+
+## Instructions
+
+1. Parse the user's prompt. If empty, default to reviewing staged git changes:
+   - Run `git diff --cached --stat` to check for staged changes
+   - If staged changes exist, use: `"Review the staged changes for correctness, security, and code quality."`
+   - If no staged changes, use: `"Review the recent changes in this repository for correctness, security, and code quality."`
+
+2. Run Codex CLI via Bash (read-only review mode — no file writes):
+
+```bash
+codex review -c model=gpt-5.4 -c reasoning=medium "<prompt>"
+```
+
+3. Present the review findings to the user.
+
+## Examples
+
+```
+# Review staged changes (default)
+/cclab:codex:review
+
+# Review specific file
+/cclab:codex:review "Review src/auth.rs for security vulnerabilities"
+
+# Review architecture
+/cclab:codex:review "Review the crate dependency graph for circular dependencies"
+
+# Review a PR
+/cclab:codex:review "Review the diff between main and HEAD for breaking changes"
+```
diff --git a/.claude/skills/cclab-gemini-explore-codebase/SKILL.md b/.claude/skills/cclab-gemini-explore-codebase/SKILL.md
new file mode 100644
index 0000000..ba46d53
--- /dev/null
+++ b/.claude/skills/cclab-gemini-explore-codebase/SKILL.md
@@ -0,0 +1,43 @@
+---
+name: cclab:gemini:explore:codebase
+description: Run Gemini headless to explore and analyze the codebase
+user-invocable: true
+---
+
+# /cclab:gemini:explore:codebase
+
+Dispatches Gemini CLI headlessly to explore source code — architecture, dependencies, patterns, and implementations.
+
+## Usage
+
+```
+/cclab:gemini:explore:codebase "<prompt>"
+```
+
+## Instructions
+
+1. Parse the user's prompt. If empty, ask the user what they want to explore.
+
+2. Run Gemini CLI headlessly via Bash:
+
+```bash
+gemini -m gemini-3-flash-preview --output-format stream-json -p "Focus on source code under crates/ and src/. <prompt>"
+```
+
+3. Parse the streamed JSON output and present the findings to the user.
+
+## Examples
+
+```
+# Map APIs
+/cclab:gemini:explore:codebase "Map all public APIs exposed by the cclab-sdd crate"
+
+# Trace callers
+/cclab:gemini:explore:codebase "Find all callers of StateManager::load across the workspace"
+
+# Architecture analysis
+/cclab:gemini:explore:codebase "Analyze the data flow from MCP tool call to state file update"
+
+# Find patterns
+/cclab:gemini:explore:codebase "Find all files that use distributed_slice for CLI registration"
+```
diff --git a/.claude/skills/cclab-gemini-explore-specs/SKILL.md b/.claude/skills/cclab-gemini-explore-specs/SKILL.md
new file mode 100644
index 0000000..4bd810d
--- /dev/null
+++ b/.claude/skills/cclab-gemini-explore-specs/SKILL.md
@@ -0,0 +1,43 @@
+---
+name: cclab:gemini:explore:specs
+description: Run Gemini headless to explore project specs and knowledge base
+user-invocable: true
+---
+
+# /cclab:gemini:explore:specs
+
+Dispatches Gemini CLI headlessly to explore `cclab/specs/`, `cclab/knowledge/`, and `cclab/changes/` — SDD specs, knowledge base, and change artifacts.
+
+## Usage
+
+```
+/cclab:gemini:explore:specs "<prompt>"
+```
+
+## Instructions
+
+1. Parse the user's prompt. If empty, ask the user what they want to explore.
+
+2. Run Gemini CLI headlessly via Bash:
+
+```bash
+gemini -m gemini-3-flash-preview --output-format stream-json -p "Focus on files under cclab/specs/, cclab/knowledge/, and cclab/changes/. <prompt>"
+```
+
+3. Parse the streamed JSON output and present the findings to the user.
+
+## Examples
+
+```
+# Find specs related to a topic
+/cclab:gemini:explore:specs "Find all specs related to the SDD workflow state machine"
+
+# Understand a change
+/cclab:gemini:explore:specs "Summarize the sdd-crate-split change — what was decided and why"
+
+# Cross-reference specs
+/cclab:gemini:explore:specs "Which specs reference StatePhase and what do they say about valid transitions?"
+
+# Knowledge base search
+/cclab:gemini:explore:specs "What conventions does the knowledge base define for crate splitting?"
+```
diff --git a/Cargo.lock b/Cargo.lock
index 5ddb419..1f1658c 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1158,7 +1158,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1169,7 +1169,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "cclab-api",
@@ -1202,7 +1202,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli-registry"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "clap",
@@ -1211,7 +1211,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "pyo3",
@@ -1220,7 +1220,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-core"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "bson",
@@ -1238,7 +1238,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "aes-gcm",
  "argon2",
@@ -1265,7 +1265,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1292,7 +1292,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1305,7 +1305,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-core"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "bitvec",
  "regex",
@@ -1332,7 +1332,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-formula"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "cclab-grid-core",
  "nom 7.1.3",
@@ -1342,7 +1342,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-history"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1350,7 +1350,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-server"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "axum 0.7.9",
@@ -1374,7 +1374,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-wasm"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1392,7 +1392,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1431,7 +1431,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "async-trait",
  "bincode",
@@ -1460,7 +1460,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1472,7 +1472,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "base64 0.22.1",
@@ -1500,7 +1500,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "image",
  "pyo3",
@@ -1511,7 +1511,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1533,7 +1533,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-nucleus"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "bson",
  "cclab-agent",
@@ -1564,7 +1564,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1594,7 +1594,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "pyo3",
  "serde",
@@ -1604,7 +1604,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-prism"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1633,7 +1633,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1665,7 +1665,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "async-nats",
  "async-trait",
@@ -1706,7 +1706,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1732,7 +1732,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "bson",
  "dotenvy",
@@ -1747,7 +1747,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "cclab-array",
  "cclab-frame",
@@ -1760,7 +1760,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1808,7 +1808,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd-cli"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "cclab-cli-registry",
@@ -1834,7 +1834,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd-mcp"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1875,7 +1875,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-server"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "anyhow",
  "async-stream",
@@ -1901,7 +1901,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1913,14 +1913,14 @@ dependencies = [
 
 [[package]]
 name = "cclab-util"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "pyo3",
 ]
 
 [[package]]
 name = "cclab-vortex"
-version = "0.3.29"
+version = "0.3.30"
 dependencies = [
  "bytemuck",
  "env_logger",
diff --git a/Cargo.toml b/Cargo.toml
index 9b936b0..971c17e 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -43,7 +43,7 @@ members = [
 resolver = "2"
 
 [workspace.package]
-version = "0.3.29"
+version = "0.3.30"
 authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
 edition = "2021"
 license = "MIT"
diff --git a/crates/cclab-sdd-cli/src/commands.rs b/crates/cclab-sdd-cli/src/commands.rs
index 21f13ab..3d07ecb 100644
--- a/crates/cclab-sdd-cli/src/commands.rs
+++ b/crates/cclab-sdd-cli/src/commands.rs
@@ -116,6 +116,73 @@ pub enum Commands {
     /// Issue platform configuration (GitHub/GitLab/Jira)
     #[command(subcommand)]
     Platform(platform::PlatformCommands),
+
+    /// Run the SDD workflow state machine (CLI equivalent of sdd_run_change MCP tool)
+    RunChange {
+        /// Change ID
+        #[arg(long)]
+        change_id: String,
+
+        /// Description (for new changes)
+        #[arg(long)]
+        description: Option<String>,
+
+        /// Issue references (e.g., --issue "#123" --issue "#456")
+        #[arg(long = "issue")]
+        issues: Vec<String>,
+
+        /// Git workflow: new_branch or in_place
+        #[arg(long)]
+        git_workflow: Option<String>,
+
+        /// Phase to advance to before routing
+        #[arg(long)]
+        advance_to: Option<String>,
+    },
+
+    /// Execute workflow actions (CLI equivalent of sdd_workflow_* MCP tools)
+    Workflow {
+        /// Action name (e.g., create-change-spec, review-change-spec, delegate-agent)
+        action: String,
+
+        /// Change ID
+        change_id: String,
+
+        /// Agent name (for delegate-agent)
+        #[arg(long)]
+        agent: Option<String>,
+
+        /// Action label (for delegate-agent)
+        #[arg(long)]
+        wf_action: Option<String>,
+
+        /// Prompt file path (for delegate-agent)
+        #[arg(long)]
+        prompt_path: Option<String>,
+    },
+
+    /// Execute artifact actions (CLI equivalent of sdd_artifact_* MCP tools)
+    Artifact {
+        /// Action name (e.g., create-change-spec, create-reference-context)
+        action: String,
+
+        /// Change ID
+        change_id: String,
+
+        /// Extra arguments as JSON (e.g., '{"spec_id":"auth","section":"overview","content":"..."}')
+        #[arg(default_value = "{}")]
+        extra_args: String,
+    },
+
+    /// Call any SDD MCP tool by name (debugging fallback)
+    ToolCall {
+        /// Tool name (e.g., sdd_workflow_create_change_spec)
+        name: String,
+
+        /// JSON arguments (default: {})
+        #[arg(default_value = "{}")]
+        args: String,
+    },
 }
 
 /// Run a genesis CLI command
@@ -194,6 +261,95 @@ pub async fn run_command(cmd: Commands) -> Result<()> {
         Commands::Platform(cmd) => {
             platform::run(cmd)?;
         }
+
+        Commands::RunChange {
+            change_id,
+            description,
+            issues,
+            git_workflow,
+            advance_to,
+        } => {
+            let project_root = std::env::current_dir()?;
+            let mut args = serde_json::json!({
+                "project_path": project_root.display().to_string(),
+                "change_id": change_id,
+            });
+            if let Some(desc) = description {
+                args["description"] = serde_json::json!(desc);
+            }
+            if !issues.is_empty() {
+                args["issues"] = serde_json::json!(issues);
+            }
+            if let Some(wf) = git_workflow {
+                args["git_workflow"] = serde_json::json!(wf);
+            }
+            if let Some(advance) = advance_to {
+                args["advance_to"] = serde_json::json!(advance);
+            }
+            let result = cclab_sdd::workflow::execute(&args, &project_root).await?;
+            println!("{}", result);
+        }
+
+        Commands::Workflow {
+            action,
+            change_id,
+            agent,
+            wf_action,
+            prompt_path,
+        } => {
+            let project_root = std::env::current_dir()?;
+            let tool_name = if action == "delegate-agent" {
+                "sdd_delegate_agent".to_string()
+            } else {
+                format!("sdd_workflow_{}", action.replace('-', "_"))
+            };
+            let mut args = serde_json::json!({
+                "project_path": project_root.display().to_string(),
+                "change_id": change_id,
+            });
+            if let Some(a) = agent {
+                args["agent"] = serde_json::json!(a);
+            }
+            if let Some(act) = wf_action {
+                args["action"] = serde_json::json!(act);
+            }
+            if let Some(pp) = prompt_path {
+                args["prompt_path"] = serde_json::json!(pp);
+            }
+            let registry = cclab_sdd_mcp::tools::ToolRegistry::new();
+            let result = registry.call_tool(&tool_name, &args).await?;
+            println!("{}", result);
+        }
+
+        Commands::Artifact {
+            action,
+            change_id,
+            extra_args,
+        } => {
+            let project_root = std::env::current_dir()?;
+            let tool_name = format!("sdd_artifact_{}", action.replace('-', "_"));
+            let mut args: serde_json::Value =
+                serde_json::from_str(&extra_args).unwrap_or_else(|_| serde_json::json!({}));
+            args["project_path"] = serde_json::json!(project_root.display().to_string());
+            args["change_id"] = serde_json::json!(change_id);
+            let registry = cclab_sdd_mcp::tools::ToolRegistry::new();
+            let result = registry.call_tool(&tool_name, &args).await?;
+            println!("{}", result);
+        }
+
+        Commands::ToolCall { name, args } => {
+            let project_root = std::env::current_dir()?;
+            let mut parsed_args: serde_json::Value =
+                serde_json::from_str(&args).unwrap_or_else(|_| serde_json::json!({}));
+            // Inject project_path if missing
+            if parsed_args.get("project_path").is_none() {
+                parsed_args["project_path"] =
+                    serde_json::json!(project_root.display().to_string());
+            }
+            let registry = cclab_sdd_mcp::tools::ToolRegistry::new();
+            let result = registry.call_tool(&name, &parsed_args).await?;
+            println!("{}", result);
+        }
     }
 
     Ok(())
diff --git a/crates/cclab-sdd-cli/src/init.rs b/crates/cclab-sdd-cli/src/init.rs
index 960b943..bc247ca 100644
--- a/crates/cclab-sdd-cli/src/init.rs
+++ b/crates/cclab-sdd-cli/src/init.rs
@@ -1,6 +1,6 @@
 use cclab_sdd_mcp::config::{ensure_claude_mcp_json, ensure_claude_settings, ensure_codex_mcp_config, ensure_gemini_mcp_config};
 use cclab_sdd_mcp::Registry;
-use cclab_sdd::models::{AgentMode, SddConfig};
+use cclab_sdd::models::{AgentMode, SddConfig, SddInterface};
 use cclab_sdd::Result;
 use colored::Colorize;
 use std::env;
@@ -88,7 +88,8 @@ pub async fn run(name: Option<&str>, _force: bool, agent_mode: Option<&str>) ->
         println!();
         run_update(name, &project_root, &sdd_dir, &claude_dir, parsed_mode)?;
     } else {
-        // Fresh install - determine agent mode and platform
+        // Fresh install - determine interface, agent mode, and platform
+        let interface = determine_interface()?;
         let mode = determine_agent_mode(agent_mode)?;
         let platform_toml = determine_platform(&project_root)?;
 
@@ -96,9 +97,10 @@ pub async fn run(name: Option<&str>, _force: bool, agent_mode: Option<&str>) ->
             "{}",
             format!("🎭 Initializing SDD v{}...", SDD_VERSION).cyan().bold()
         );
+        println!("   Interface: {}", interface.name().green());
         println!("   Agent mode: {}", mode.name().green());
         println!();
-        run_fresh_install(name, &project_root, &sdd_dir, &claude_dir, mode, platform_toml)?;
+        run_fresh_install(name, &project_root, &sdd_dir, &claude_dir, interface, mode, platform_toml)?;
     }
 
     Ok(())
@@ -395,6 +397,41 @@ fn ensure_gitignore_entry(project_root: &Path, entry: &str) -> Result<()> {
 // Agent mode selection
 // ---------------------------------------------------------------------------
 
+/// Determine SDD interface mode (CLI or MCP)
+fn determine_interface() -> Result<SddInterface> {
+    println!("{}", "🔧 Select SDD interface:".cyan().bold());
+    println!();
+    println!(
+        "   {} {} {}",
+        "1.".green().bold(),
+        "CLI".white().bold(),
+        "(Recommended)".green()
+    );
+    println!(
+        "      {}",
+        SddInterface::Cli.description().dimmed()
+    );
+    println!();
+    println!("   {} {}", "2.".green().bold(), "MCP".white().bold());
+    println!(
+        "      {}",
+        SddInterface::Mcp.description().dimmed()
+    );
+    println!();
+
+    print!("   Enter choice [1/2] (default: 1): ");
+    io::stdout().flush()?;
+
+    let mut input = String::new();
+    io::stdin().read_line(&mut input)?;
+    let interface = match input.trim() {
+        "2" | "mcp" => SddInterface::Mcp,
+        _ => SddInterface::Cli,
+    };
+    println!();
+    Ok(interface)
+}
+
 /// Determine agent mode from CLI argument or interactive prompt
 fn determine_agent_mode(agent_mode: Option<&str>) -> Result<AgentMode> {
     // If provided via CLI, use it
@@ -438,6 +475,7 @@ fn run_fresh_install(
     project_root: &Path,
     sdd_dir: &Path,
     claude_dir: &Path,
+    interface: SddInterface,
     agent_mode: AgentMode,
     platform_toml: Option<String>,
 ) -> Result<()> {
@@ -452,9 +490,9 @@ fn run_fresh_install(
     let skills_dir = claude_dir.join("skills");
     std::fs::create_dir_all(&skills_dir)?;
 
-    // Create config with selected agent mode
+    // Create config with selected interface and agent mode
     let _ = name; // name parameter is deprecated and ignored
-    let mut config = SddConfig::with_agent_mode(agent_mode);
+    let mut config = SddConfig::with_interface_and_agent_mode(interface, agent_mode);
     config.set_version(SDD_VERSION);
     config.save(project_root)?;
 
@@ -467,7 +505,7 @@ fn run_fresh_install(
     }
 
     let platform_info = if platform_toml.is_some() { " + platform" } else { "" };
-    println!("   ✓ cclab/config.toml (mode: {}{})", agent_mode.name(), platform_info);
+    println!("   ✓ cclab/config.toml (interface: {}, mode: {}{})", interface.name(), agent_mode.name(), platform_info);
 
     // Install system files
     install_system_files(project_root, sdd_dir, claude_dir)?;
diff --git a/crates/cclab-sdd-cli/templates/config.toml b/crates/cclab-sdd-cli/templates/config.toml
index db2bd5c..d0cfe90 100644
--- a/crates/cclab-sdd-cli/templates/config.toml
+++ b/crates/cclab-sdd-cli/templates/config.toml
@@ -1,161 +1,51 @@
 # Genesis Configuration
 # Copy to cclab/config.toml and customize as needed
 
-# =============================================================================
-# Project Configuration (monorepo-aware)
-# =============================================================================
-
-# Map directory paths to languages. Used by Genesis task generator and Prism.
-# [[project.modules]]
-# path = "."           # project root (single-language project)
-# language = "rust"    # rust, python, typescript, javascript, go
-# framework = "axum"   # optional: axum, react, django, etc.
-
-# Example monorepo:
-# [[project.modules]]
-# path = "api/"
-# language = "python"
-# framework = "django"
-#
-# [[project.modules]]
-# path = "frontend/"
-# language = "typescript"
-# framework = "react"
-
 # =============================================================================
 # Workflow Configuration
 # =============================================================================
 
-# Per-artifact agent configuration
-# Format: "agent" or "agent:model" (e.g., "gemini:pro", "codex:deep")
+# Per-artifact agent configuration — format: {agent}:{model}[ {reasoning}]
 # Always include "mainthread" as final fallback
-# Available agents: gemini, codex, claude, mainthread
+# Available agents:
+#   gemini:      gemini-3-flash-preview, gemini-3.1-pro-preview
+#   codex:       gpt-5.4 low, gpt-5.4 medium, gpt-5.4 high, gpt-5.4 xhigh
+#   codex-spark: gpt-5.4-mini low, gpt-5.4-mini medium, gpt-5.4-mini high, gpt-5.4-mini xhigh
+#   claude:      claude-haiku-4-5, claude-sonnet-4-6, claude-opus-4-6
+#   mainthread   (no model needed)
 
 # Global envfile - loaded for all agents (relative to project root or absolute)
 # envfile = ".env"
 
+# How the SKILL template interacts with SDD tools:
+# - "cli" (default): runs `cclab sdd workflow/artifact` commands (no MCP server needed)
+# - "mcp": calls MCP tools via server (requires `cclab server start`)
+# interface = "cli"
+
 [workflow.agents]
-# Plan phase - specs
-create_spec = ["gemini:pro", "mainthread"]
-review_spec = ["codex:max", "mainthread"]
-revise_spec = ["gemini:pro", "mainthread"]
-
-# Impl phase
-implement = ["mainthread"]
-review_impl = ["codex:balanced", "mainthread"]
-revise_impl = ["mainthread"]
-
-# Merge phase
-merge_specs = ["gemini:flash", "mainthread"]
-review_merge = ["codex:balanced", "mainthread"]
-revise_merge = ["gemini:flash", "mainthread"]
+restructure_input = ["gemini:gemini-3-flash-preview", "mainthread"]
+create_pre_clarifications = ["mainthread"]
+create_post_clarifications = ["mainthread"]
+create_reference_context = ["gemini:gemini-3-flash-preview", "mainthread"]
+review_reference_context = ["codex:gpt-5.4 medium", "mainthread"]
+revise_reference_context = ["gemini:gemini-3-flash-preview", "mainthread"]
+create_change_spec = ["gemini:gemini-3.1-pro-preview", "mainthread"]
+review_change_spec = ["codex:gpt-5.4 xhigh", "mainthread"]
+revise_change_spec = ["gemini:gemini-3.1-pro-preview", "mainthread"]
+create_change_implementation = ["mainthread"]
+review_change_implementation = ["codex:gpt-5.4 medium", "mainthread"]
+revise_change_implementation = ["mainthread"]
+create_change_merge = ["gemini:gemini-3-flash-preview", "mainthread"]
 
 # =============================================================================
-# Agent Configurations
+# Agent Configurations (envfile only — models use built-in defaults)
 # =============================================================================
 
 [gemini]
-command = "gemini"
-default = "flash"
-# envfile = ".gemini.env"  # provider-specific env (overrides global)
-
-[[gemini.models]]
-id = "flash"
-model = "gemini-3-flash-preview"
-complexity = "medium"
-cost_per_1m_input = 0.1
-cost_per_1m_output = 0.4
-
-[[gemini.models]]
-id = "pro"
-model = "gemini-3-pro-preview"
-complexity = "critical"
-cost_per_1m_input = 1.25
-cost_per_1m_output = 10.0
+# envfile = ".gemini.env"
 
 [codex]
-command = "codex"
-default = "balanced"
 # envfile = ".codex.env"
 
-[[codex.models]]
-id = "fast"
-model = "gpt-5.3-codex"
-reasoning = "low"
-complexity = "low"
-cost_per_1m_input = 2.0
-cost_per_1m_output = 8.0
-
-[[codex.models]]
-id = "balanced"
-model = "gpt-5.3-codex"
-reasoning = "medium"
-complexity = "medium"
-cost_per_1m_input = 2.0
-cost_per_1m_output = 8.0
-
-[[codex.models]]
-id = "deep"
-model = "gpt-5.3-codex"
-reasoning = "high"
-complexity = "high"
-cost_per_1m_input = 2.0
-cost_per_1m_output = 8.0
-
-[[codex.models]]
-id = "max"
-model = "gpt-5.3-codex"
-reasoning = "xhigh"
-complexity = "critical"
-cost_per_1m_input = 2.0
-cost_per_1m_output = 8.0
-
 [claude]
-command = "claude"
-default = "balanced"
 # envfile = ".claude.env"
-
-[[claude.models]]
-id = "fast"
-model = "haiku"
-complexity = "low"
-cost_per_1m_input = 0.8
-cost_per_1m_output = 4.0
-
-[[claude.models]]
-id = "balanced"
-model = "sonnet"
-complexity = "medium"
-cost_per_1m_input = 3.0
-cost_per_1m_output = 15.0
-
-[[claude.models]]
-id = "deep"
-model = "opus"
-complexity = "critical"
-cost_per_1m_input = 15.0
-cost_per_1m_output = 75.0
-
-# =============================================================================
-# Validation Rules
-# =============================================================================
-
-[validation]
-required_headings = [
-    "Overview",
-    "Acceptance Criteria",
-]
-requirement_pattern = ""
-scenario_pattern = '(?m)^###\s*Scenario:|^-\s*WHEN[^\n]*THEN'
-scenario_min_count = 1
-require_when_then = true
-when_pattern = '\*\*WHEN\*\*|WHEN'
-then_pattern = '\*\*THEN\*\*|THEN'
-
-[validation.severity_map]
-missing_heading = "High"
-invalid_requirement_format = "High"
-missing_scenario = "High"
-missing_when_then = "High"
-duplicate_requirement = "High"
-broken_reference = "Medium"
diff --git a/crates/cclab-sdd-cli/templates/mainthread/CLAUDE.md b/crates/cclab-sdd-cli/templates/mainthread/CLAUDE.md
index fd583e4..b8eaf6a 100644
--- a/crates/cclab-sdd-cli/templates/mainthread/CLAUDE.md
+++ b/crates/cclab-sdd-cli/templates/mainthread/CLAUDE.md
@@ -29,7 +29,7 @@ MCP tools: `sdd_read_artifact(scope="list:main_specs")` / `sdd_read_artifact(sco
 | `/cclab:gemini:explore:specs` | Run Gemini headless to explore specs and knowledge base |
 | `/cclab:gemini:explore:codebase` | Run Gemini headless to explore source code |
 
-The skill is **state-aware** — calls `sdd_run_change` MCP tool internally and resumes from the current phase.
+The skill is **state-aware** — resumes from the current phase. Check `cclab/config.toml` for `interface` (cli or mcp) to determine tool dispatch mode.
 
 **CRITICAL**: One change-id handles ALL referenced issues. NEVER split multiple issues (e.g. `"#272 #273"`) into separate changes. Pass the full description to the tool — it manages multi-issue routing internally.
 
diff --git a/crates/cclab-sdd-cli/templates/mainthread/skills/cclab-sdd-run-change/SKILL.md b/crates/cclab-sdd-cli/templates/mainthread/skills/cclab-sdd-run-change/SKILL.md
index fb6267c..10c52b3 100644
--- a/crates/cclab-sdd-cli/templates/mainthread/skills/cclab-sdd-run-change/SKILL.md
+++ b/crates/cclab-sdd-cli/templates/mainthread/skills/cclab-sdd-run-change/SKILL.md
@@ -6,11 +6,17 @@ user-invocable: true
 
 # /cclab:sdd:run-change
 
-Calls `sdd_run_change` MCP tool in a loop. Each call returns the next workflow tool to call. Mainthread drives the loop and dispatches to agents when the workflow tool's `executor` field indicates a non-mainthread executor.
+Runs the SDD workflow loop. Each iteration returns the next action. Mainthread drives the loop and dispatches to agents when needed.
+
+## Interface Mode
+
+Check `cclab/config.toml` for `interface`:
+- `interface = "cli"` (default) → use CLI commands (Bash tool)
+- `interface = "mcp"` → use MCP tool calls
 
 ## Entry Point
 
-Before calling `sdd_run_change`, resolve the change-id, extract issue references, and set up git workflow.
+Before starting the loop, resolve the change-id, extract issue references, and set up git workflow.
 
 ### 1. Change-ID Resolution
 
@@ -44,28 +50,39 @@ Skip this step if resuming an existing change (no description).
    - `new_branch` → `git checkout -b cclab/{change-id}`
    - `in_place` → no action
 4. Record `branch` = current branch name (after checkout if `new_branch`)
-5. Pass `git_workflow` and `branch` to `sdd_run_change`
+5. Pass `git_workflow` and `branch` to the first workflow call
 
 ## Rules
 
 1. **One change-id = one change.** NEVER split multiple issues into separate changes. Pass the full description as-is.
-2. **Do NOT plan, interpret, or act on the description yourself.** Call `sdd_run_change` immediately after entry point resolution. The tool decides what to do.
+2. **Do NOT plan, interpret, or act on the description yourself.** Start the loop immediately after entry point resolution. The tool decides what to do.
 3. **Follow the response literally.** Read `action`, `prompt`, and `executor` from the response:
    - `action: "complete"` → workflow done
    - Other actions → check `executor` to decide who handles it (see Loop below)
-4. **MCP tools only.** Do NOT use CLI commands.
 
 ## Loop
 
+The response format is interface-aware — only ONE of `tool` or `cli` will be present in `next_actions` entries.
+
+### MCP Mode (`interface = "mcp"`)
+
 1. Call `sdd_run_change(project_path=$PWD, change_id=<id>, description=<desc>, issues=<issues>, git_workflow=<workflow>, branch=<branch>)`
    - `description`, `issues`, `git_workflow`, `branch` only on first call (new change)
-   - `init_change` is handled internally — no separate step needed
-2. Read `next[0].tool` from the response → call the workflow tool (e.g. `sdd_workflow_create_reference_context`)
+2. Read `next_actions[0].tool` + `next_actions[0].args` → call that MCP tool
 3. Check the workflow tool response `executor` field:
-   - `executor[0]` is **not** `"mainthread"` → delegate to agent: call `sdd_delegate_agent(agent=executor[0], action=<artifact_name>, prompt=response.prompt, change_id=<id>)`
-   - `executor[0]` is `"mainthread"` → follow `prompt` + `next_actions` directly
-4. Call `sdd_run_change` again
-5. Repeat until `action: "complete"`
+   - `executor[0]` is **not** `"mainthread"` → the response already has `next_actions` with `sdd_delegate_agent` — call it
+   - `executor[0]` is `"mainthread"` → follow `prompt_path` (read the file) and `next_actions`
+4. Call `sdd_run_change` again → repeat until `action: "complete"`
+
+### CLI Mode (`interface = "cli"`)
+
+1. Run: `cclab sdd run-change --change-id <id> --description "<desc>" --issue "#123" --git-workflow <wf>`
+   - `--description`, `--issue`, `--git-workflow` only on first call (new change)
+2. Parse JSON output. Read `next_actions[0].cli` → run that CLI command via Bash tool
+3. Parse the CLI output (JSON). Check `executor` field:
+   - `executor[0]` is **not** `"mainthread"` → the response already has `next_actions` with a `cli` command to delegate — run it
+   - `executor[0]` is `"mainthread"` → follow `prompt_path` (read the file) and `next_actions`
+4. Run `cclab sdd run-change --change-id <id>` again → repeat until `action: "complete"`
 
 ## Usage
 
diff --git a/crates/cclab-sdd-mcp/src/tools/agent.rs b/crates/cclab-sdd-mcp/src/tools/agent.rs
index 493f5e0..3b7f971 100644
--- a/crates/cclab-sdd-mcp/src/tools/agent.rs
+++ b/crates/cclab-sdd-mcp/src/tools/agent.rs
@@ -77,7 +77,7 @@ fn parse_agent_spec(agent: &str) -> Result<(LlmProvider, String)> {
     let parts: Vec<&str> = agent.splitn(2, ':').collect();
     if parts.len() != 2 {
         anyhow::bail!(
-            "Invalid agent spec '{}'. Expected format: provider:model_id (e.g. 'gemini:flash', 'codex:balanced', 'claude:fast')",
+            "Invalid agent spec '{}'. Expected format: provider:model (e.g. 'gemini:gemini-3-flash-preview', 'codex:gpt-5.4 medium', 'claude:claude-sonnet-4-6')",
             agent
         );
     }
@@ -504,23 +504,23 @@ mod tests {
 
     #[test]
     fn test_parse_agent_spec_gemini() {
-        let (provider, model_id) = parse_agent_spec("gemini:flash").unwrap();
+        let (provider, model_id) = parse_agent_spec("gemini:gemini-3-flash-preview").unwrap();
         assert_eq!(provider, LlmProvider::Gemini);
-        assert_eq!(model_id, "flash");
+        assert_eq!(model_id, "gemini-3-flash-preview");
     }
 
     #[test]
     fn test_parse_agent_spec_codex() {
-        let (provider, model_id) = parse_agent_spec("codex:balanced").unwrap();
+        let (provider, model_id) = parse_agent_spec("codex:gpt-5.4 medium").unwrap();
         assert_eq!(provider, LlmProvider::Codex);
-        assert_eq!(model_id, "balanced");
+        assert_eq!(model_id, "gpt-5.4 medium");
     }
 
     #[test]
     fn test_parse_agent_spec_claude() {
-        let (provider, model_id) = parse_agent_spec("claude:fast").unwrap();
+        let (provider, model_id) = parse_agent_spec("claude:claude-sonnet-4-6").unwrap();
         assert_eq!(provider, LlmProvider::Claude);
-        assert_eq!(model_id, "fast");
+        assert_eq!(model_id, "claude-sonnet-4-6");
     }
 
     #[test]
@@ -606,12 +606,12 @@ mod tests {
     #[test]
     fn test_build_provider_args_claude() {
         let selected = SelectedModel::Claude {
-            model: "haiku".to_string(),
+            model: cclab_sdd::defaults::CLAUDE_BALANCED_MODEL.to_string(),
             command: "".to_string(),
         };
         let args = build_provider_args(LlmProvider::Claude, &selected, "test prompt");
         assert!(args.contains(&"--model".to_string()));
-        assert!(args.contains(&"haiku".to_string()));
+        assert!(args.contains(&cclab_sdd::defaults::CLAUDE_BALANCED_MODEL.to_string()));
         assert!(args.contains(&"-p".to_string()));
         assert!(args.contains(&"test prompt".to_string()));
         // Verify --system-prompt "" overrides CLAUDE.md
diff --git a/crates/cclab-sdd-mcp/src/tools/change_impl/create.rs b/crates/cclab-sdd-mcp/src/tools/change_impl/create.rs
index a90151a..4533d3b 100644
--- a/crates/cclab-sdd-mcp/src/tools/change_impl/create.rs
+++ b/crates/cclab-sdd-mcp/src/tools/change_impl/create.rs
@@ -100,6 +100,8 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
         }
     }
 
+    let interface = workflow_common::load_interface(project_root);
+
     match sub_state {
         ImplSubState::NoSpecs => {
             let result = json!({
@@ -127,12 +129,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "prompt": format!("Spec '{}' ready for review. Redirecting to review workflow.", spec_id),
-                "next_actions": [{
-                    "tool": "sdd_workflow_review_change_implementation",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_review_change_implementation", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -141,12 +138,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "spec_id": spec_id,
-                "next_actions": [{
-                    "tool": "sdd_workflow_revise_change_implementation",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_revise_change_implementation", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -160,16 +152,10 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
                     spec_id, revisions, MAX_SPEC_REVISIONS
                 ),
                 "spec_id": spec_id,
-                "next_actions": [{
-                    "tool": "sdd_run_change",
-                    "args": {
-                        "change_id": change_id,
-                        "advance_to": "change_implementation_created"
-                    },
-                    "executor": "mainthread",
-                    "when": "optional",
-                    "description": "Reset revision counter and retry implementation"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_run_change", json!({
+                    "change_id": change_id,
+                    "advance_to": "change_implementation_created"
+                }))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -178,15 +164,10 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "message": "All specs implemented and approved! Advancing to merge phase.",
-                "next_actions": [{
-                    "tool": "sdd_run_change",
-                    "args": {
-                        "change_id": change_id,
-                        "advance_to": "change_merge_created"
-                    },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_run_change", json!({
+                    "change_id": change_id,
+                    "advance_to": "change_merge_created"
+                }))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -218,15 +199,11 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
 
     std::fs::write(&impl_path, &content)?;
 
+    let interface = workflow_common::load_interface(project_root);
     let result = json!({
         "status": "ok",
         "artifacts_written": ["implementation.md"],
-        "next_actions": [{
-            "tool": "sdd_workflow_create_change_implementation",
-            "args": { "change_id": change_id },
-            "when": "immediate",
-            "executor": "mainthread"
-        }]
+        "next_actions": [workflow_common::next_action(interface, "sdd_workflow_create_change_implementation", json!({"change_id": change_id}))]
     });
     Ok(serde_json::to_string_pretty(&result)?)
 }
@@ -280,6 +257,7 @@ fn build_implement_prompt(
 
     let action = if is_first { "begin_implementation" } else { "implement_spec" };
     let change_dir = project_root.join("cclab/changes").join(change_id);
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeImplementation);
 
     workflow_common::build_workflow_response(
@@ -289,6 +267,7 @@ fn build_implement_prompt(
         prompt,
         executor,
         json!({ "spec_id": spec_id }),
+        interface,
     )
 }
 
@@ -321,6 +300,7 @@ fn build_codegen_prompt(
     );
 
     let change_dir = project_root.join("cclab/changes").join(change_id);
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeImplementation);
 
     workflow_common::build_workflow_response(
@@ -330,6 +310,7 @@ fn build_codegen_prompt(
         prompt,
         executor,
         json!({ "spec_id": spec_id, "codegen": true }),
+        interface,
     )
 }
 
@@ -430,6 +411,7 @@ fn build_write_diff_prompt(
     );
 
     let change_dir = project_root.join("cclab/changes").join(change_id);
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeImplementation);
 
     workflow_common::build_workflow_response(
@@ -439,6 +421,7 @@ fn build_write_diff_prompt(
         prompt,
         executor,
         json!({}),
+        interface,
     )
 }
 
@@ -568,7 +551,9 @@ mod tests {
         assert_eq!(parsed["status"], "ok");
         assert!(parsed["message"].as_str().unwrap().contains("Advancing to merge"));
         let next = parsed["next_actions"].as_array().unwrap();
-        assert_eq!(next[0]["tool"], "sdd_run_change");
+        // next_action may use "tool" (MCP) or "cli" (CLI) depending on interface config
+        assert!(next[0].get("tool").is_some() || next[0].get("cli").is_some());
+        assert_eq!(next[0]["args"]["change_id"], "wf-merge");
         assert_eq!(next[0]["args"]["advance_to"], "change_merge_created");
     }
 
@@ -604,8 +589,9 @@ mod tests {
         assert_eq!(parsed["status"], "error");
         let next = parsed["next_actions"].as_array().unwrap();
         assert!(!next.is_empty(), "TerminalFailure should provide retry next_actions");
-        assert_eq!(next[0]["tool"], "sdd_run_change");
+        // next_action may use "tool" (MCP) or "cli" (CLI) depending on interface config
+        assert!(next[0].get("tool").is_some() || next[0].get("cli").is_some());
+        assert_eq!(next[0]["args"]["change_id"], "wf-fail");
         assert_eq!(next[0]["args"]["advance_to"], "change_implementation_created");
-        assert_eq!(next[0]["when"], "optional");
     }
 }
diff --git a/crates/cclab-sdd-mcp/src/tools/change_impl/review.rs b/crates/cclab-sdd-mcp/src/tools/change_impl/review.rs
index 430ec5f..254b639 100644
--- a/crates/cclab-sdd-mcp/src/tools/change_impl/review.rs
+++ b/crates/cclab-sdd-mcp/src/tools/change_impl/review.rs
@@ -115,6 +115,8 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
 
     let (sub_state, _, _) = common::resolve_next_impl(&change_dir, &change_id)?;
 
+    let interface = workflow_common::load_interface(project_root);
+
     match sub_state {
         ImplSubState::ReviewSpec { spec_id } => {
             build_review_prompt(&change_id, &spec_id, project_root)
@@ -124,12 +126,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "prompt": "Not in review sub-state. Redirecting to create router.",
-                "next_actions": [{
-                    "tool": "sdd_workflow_create_change_implementation",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_create_change_implementation", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -196,15 +193,11 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
     // Update phase
     workflow_common::update_phase(&change_dir, StatePhase::ChangeImplementationReviewed)?;
 
+    let interface = workflow_common::load_interface(project_root);
     let result = json!({
         "status": "ok",
         "artifacts_written": ["implementation.md"],
-        "next_actions": [{
-            "tool": "sdd_workflow_create_change_implementation",
-            "args": { "change_id": change_id },
-            "when": "immediate",
-            "executor": "mainthread"
-        }]
+        "next_actions": [workflow_common::next_action(interface, "sdd_workflow_create_change_implementation", json!({"change_id": change_id}))]
     });
     Ok(serde_json::to_string_pretty(&result)?)
 }
@@ -326,6 +319,7 @@ mcp__cclab-mcp__sdd_artifact_review_change_implementation(project_path="{pp}", c
     );
 
     let change_dir = project_root.join("cclab/changes").join(change_id);
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewChangeImplementation);
 
     workflow_common::build_workflow_response(
@@ -335,6 +329,7 @@ mcp__cclab-mcp__sdd_artifact_review_change_implementation(project_path="{pp}", c
         prompt,
         executor,
         json!({ "spec_id": spec_id }),
+        interface,
     )
 }
 
diff --git a/crates/cclab-sdd-mcp/src/tools/change_impl/revise.rs b/crates/cclab-sdd-mcp/src/tools/change_impl/revise.rs
index c965a7b..1ef1911 100644
--- a/crates/cclab-sdd-mcp/src/tools/change_impl/revise.rs
+++ b/crates/cclab-sdd-mcp/src/tools/change_impl/revise.rs
@@ -83,6 +83,8 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
 
     let (sub_state, _, _) = common::resolve_next_impl(&change_dir, &change_id)?;
 
+    let interface = workflow_common::load_interface(project_root);
+
     match sub_state {
         common::ImplSubState::ReviseSpec { spec_id } => {
             build_revise_prompt(&change_id, &spec_id, project_root)
@@ -92,12 +94,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "message": "Implementation is not in Revise sub-state. Redirecting to router.",
-                "next_actions": [{
-                    "tool": "sdd_workflow_create_change_implementation",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_create_change_implementation", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -139,6 +136,7 @@ fn build_revise_prompt(
     );
 
     let change_dir = project_root.join("cclab/changes").join(change_id);
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseChangeImplementation);
 
     workflow_common::build_workflow_response(
@@ -148,5 +146,6 @@ fn build_revise_prompt(
         prompt,
         executor,
         json!({ "spec_id": spec_id }),
+        interface,
     )
 }
diff --git a/crates/cclab-sdd-mcp/src/tools/change_merge/create.rs b/crates/cclab-sdd-mcp/src/tools/change_merge/create.rs
index c017da4..0a0cb9b 100644
--- a/crates/cclab-sdd-mcp/src/tools/change_merge/create.rs
+++ b/crates/cclab-sdd-mcp/src/tools/change_merge/create.rs
@@ -150,6 +150,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
         extra_fields["warnings"] = json!(errors);
     }
 
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeMerge);
 
     workflow_common::build_workflow_response(
@@ -159,6 +160,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
         prompt,
         executor,
         extra_fields,
+        interface,
     )
 }
 
diff --git a/crates/cclab-sdd-mcp/src/tools/change_spec/create.rs b/crates/cclab-sdd-mcp/src/tools/change_spec/create.rs
index cfdfb0f..24626ea 100644
--- a/crates/cclab-sdd-mcp/src/tools/change_spec/create.rs
+++ b/crates/cclab-sdd-mcp/src/tools/change_spec/create.rs
@@ -111,6 +111,8 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
     let change_dir = project_root.join("cclab/changes").join(&change_id);
     workflow_common::validate_change_dir(&change_dir, project_root)?;
 
+    let interface = workflow_common::load_interface(project_root);
+
     match common::resolve_next_spec(&change_dir, &change_id)? {
         SpecSubState::Create { spec_id, depends } => {
             handle_create_sub_state(
@@ -125,12 +127,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "spec_id": spec_id,
-                "next_actions": [{
-                    "tool": "sdd_workflow_review_change_spec",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_review_change_spec", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -138,12 +135,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "spec_id": spec_id,
-                "next_actions": [{
-                    "tool": "sdd_workflow_revise_change_spec",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_revise_change_spec", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -155,6 +147,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
                 &phase,
                 &format!("spec:{}", spec_id),
                 &format!("review_spec_{}", spec_id),
+                interface,
             );
             Ok(serde_json::to_string_pretty(&resp)?)
         }
@@ -162,15 +155,10 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "phase_complete",
                 "prompt": "All specs created and reviewed. Advancing to implementation.",
-                "next_actions": [{
-                    "tool": helpers::RUN_CHANGE_TOOL,
-                    "args": {
-                        "change_id": change_id,
-                        "advance_to": "change_implementation_created",
-                    },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, helpers::RUN_CHANGE_TOOL, json!({
+                    "change_id": change_id,
+                    "advance_to": "change_implementation_created",
+                }))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -226,15 +214,11 @@ fn handle_create_sub_state(
 
     if common::is_create_complete(&content) {
         // Create done — redirect to review
+        let interface = workflow_common::load_interface(project_root);
         let result = json!({
             "status": "ok",
             "spec_id": spec_id,
-            "next_actions": [{
-                "tool": "sdd_workflow_review_change_spec",
-                "args": { "change_id": change_id },
-                "when": "immediate",
-                "executor": "mainthread"
-            }]
+            "next_actions": [workflow_common::next_action(interface, "sdd_workflow_review_change_spec", json!({"change_id": change_id}))]
         });
         return Ok(serde_json::to_string_pretty(&result)?);
     }
@@ -266,16 +250,12 @@ fn handle_create_sub_state(
         std::fs::write(&spec_path, &marked)?;
 
         // Redirect to review
+        let interface = workflow_common::load_interface(project_root);
         let result = json!({
             "status": "ok",
             "spec_id": spec_id,
             "message": "Spec creation complete. All sections filled and pruned.",
-            "next_actions": [{
-                "tool": "sdd_workflow_review_change_spec",
-                "args": { "change_id": change_id },
-                "when": "immediate",
-                "executor": "mainthread"
-            }]
+            "next_actions": [workflow_common::next_action(interface, "sdd_workflow_review_change_spec", json!({"change_id": change_id}))]
         });
         Ok(serde_json::to_string_pretty(&result)?)
     }
@@ -358,15 +338,11 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
 
     let artifacts_written = vec![format!("specs/{}.md", spec_id)];
 
+    let interface = workflow_common::load_interface(project_root);
     let result = json!({
         "status": "ok",
         "artifacts_written": artifacts_written,
-        "next_actions": [{
-            "tool": "sdd_workflow_create_change_spec",
-            "args": { "change_id": change_id },
-            "when": "immediate",
-            "executor": "mainthread"
-        }]
+        "next_actions": [workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))]
     });
 
     Ok(serde_json::to_string_pretty(&result)?)
@@ -441,6 +417,7 @@ mcp__cclab-mcp__sdd_artifact_create_change_spec(project_path="{pp}", change_id="
     );
 
     let change_dir = project_root.join("cclab/changes").join(change_id);
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeSpec);
 
     workflow_common::build_workflow_response(
@@ -450,6 +427,7 @@ mcp__cclab-mcp__sdd_artifact_create_change_spec(project_path="{pp}", change_id="
         prompt,
         executor,
         json!({ "spec_id": spec_id }),
+        interface,
     )
 }
 
@@ -507,6 +485,7 @@ mcp__cclab-mcp__sdd_artifact_create_change_spec(project_path="{pp}", change_id="
     );
 
     let change_dir = project_root.join("cclab/changes").join(change_id);
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeSpec);
 
     workflow_common::build_workflow_response(
@@ -516,6 +495,7 @@ mcp__cclab-mcp__sdd_artifact_create_change_spec(project_path="{pp}", change_id="
         prompt,
         executor,
         json!({ "spec_id": spec_id }),
+        interface,
     )
 }
 
@@ -635,7 +615,9 @@ mod tests {
 
         // Should redirect to review (create complete)
         let next = &parsed["next_actions"][0];
-        assert_eq!(next["tool"], "sdd_workflow_review_change_spec");
+        // next_action may use "tool" (MCP) or "cli" (CLI) depending on interface config
+        assert!(next.get("tool").is_some() || next.get("cli").is_some());
+        assert_eq!(next["args"]["change_id"], "prune-test");
 
         // Verify file was pruned
         let spec_path = change_dir.join("specs/prune-test-spec.md");
diff --git a/crates/cclab-sdd-mcp/src/tools/change_spec/review.rs b/crates/cclab-sdd-mcp/src/tools/change_spec/review.rs
index d22d240..e667d82 100644
--- a/crates/cclab-sdd-mcp/src/tools/change_spec/review.rs
+++ b/crates/cclab-sdd-mcp/src/tools/change_spec/review.rs
@@ -125,6 +125,8 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
     let change_dir = project_root.join("cclab/changes").join(&change_id);
     workflow_common::validate_change_dir(&change_dir, project_root)?;
 
+    let interface = workflow_common::load_interface(project_root);
+
     match common::resolve_next_spec(&change_dir, &change_id)? {
         SpecSubState::Review { spec_id } => {
             build_review_prompt(&change_id, &spec_id, project_root)
@@ -134,12 +136,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "prompt": "Spec is not in Review sub-state. Redirecting to router.",
-                "next_actions": [{
-                    "tool": "sdd_workflow_create_change_spec",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -272,15 +269,11 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
 
     let artifacts_written = vec![format!("specs/{}.md", spec_id)];
 
+    let interface = workflow_common::load_interface(project_root);
     let result = json!({
         "status": "ok",
         "artifacts_written": artifacts_written,
-        "next_actions": [{
-            "tool": "sdd_workflow_create_change_spec",
-            "args": { "change_id": change_id },
-            "when": "immediate",
-            "executor": "mainthread"
-        }]
+        "next_actions": [workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))]
     });
 
     Ok(serde_json::to_string_pretty(&result)?)
@@ -333,6 +326,7 @@ mcp__cclab-mcp__sdd_artifact_review_change_spec(project_path="{pp}", change_id="
     );
 
     let change_dir = project_root.join("cclab/changes").join(change_id);
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviewChangeSpec);
 
     workflow_common::build_workflow_response(
@@ -342,6 +336,7 @@ mcp__cclab-mcp__sdd_artifact_review_change_spec(project_path="{pp}", change_id="
         prompt,
         executor,
         json!({ "spec_id": spec_id }),
+        interface,
     )
 }
 
diff --git a/crates/cclab-sdd-mcp/src/tools/change_spec/revise.rs b/crates/cclab-sdd-mcp/src/tools/change_spec/revise.rs
index 712affa..e7d400c 100644
--- a/crates/cclab-sdd-mcp/src/tools/change_spec/revise.rs
+++ b/crates/cclab-sdd-mcp/src/tools/change_spec/revise.rs
@@ -89,6 +89,8 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
     let change_dir = project_root.join("cclab/changes").join(&change_id);
     workflow_common::validate_change_dir(&change_dir, project_root)?;
 
+    let interface = workflow_common::load_interface(project_root);
+
     match common::resolve_next_spec(&change_dir, &change_id)? {
         common::SpecSubState::Revise { spec_id } => {
             handle_revise_sub_state(&change_id, &spec_id, &change_dir, project_root)
@@ -98,12 +100,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "message": "Spec is not in Revise sub-state. Redirecting to router.",
-                "next_actions": [{
-                    "tool": "sdd_workflow_create_change_spec",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -172,16 +169,12 @@ fn handle_revise_sub_state(
         workflow_common::update_phase(change_dir, StatePhase::ChangeSpecRevised)?;
 
         // Redirect back to workflow router (will go to review)
+        let interface = workflow_common::load_interface(project_root);
         let result = json!({
             "status": "ok",
             "spec_id": spec_id,
             "message": "Revision complete. Flagged sections re-filled.",
-            "next_actions": [{
-                "tool": "sdd_workflow_create_change_spec",
-                "args": { "change_id": change_id },
-                "when": "immediate",
-                "executor": "mainthread"
-            }]
+            "next_actions": [workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))]
         });
         Ok(serde_json::to_string_pretty(&result)?)
     }
@@ -262,6 +255,7 @@ mcp__cclab-mcp__sdd_artifact_revise_change_spec(project_path="{pp}", change_id="
     );
 
     let change_dir = project_root.join("cclab/changes").join(change_id);
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::ReviseChangeSpec);
 
     workflow_common::build_workflow_response(
@@ -271,5 +265,6 @@ mcp__cclab-mcp__sdd_artifact_revise_change_spec(project_path="{pp}", change_id="
         prompt,
         executor,
         json!({ "spec_id": spec_id }),
+        interface,
     )
 }
diff --git a/crates/cclab-sdd-mcp/src/tools/create_pre_clarifications.rs b/crates/cclab-sdd-mcp/src/tools/create_pre_clarifications.rs
index 5597c89..a6501bf 100644
--- a/crates/cclab-sdd-mcp/src/tools/create_pre_clarifications.rs
+++ b/crates/cclab-sdd-mcp/src/tools/create_pre_clarifications.rs
@@ -355,17 +355,13 @@ pub fn execute_workflow_pre_clarifications(
         sm.set_phase(StatePhase::PreClarificationsCreated)?;
         sm.save()?;
 
+        let interface = workflow_common::load_interface(project_root);
         let result = json!({
             "status": "phase_complete",
             "prompt": "All groups have been clarified. Phase advanced to PreClarificationsCreated.",
             "group_id": null,
             "next_actions": [
-                {
-                    "tool": "sdd_run_change",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }
+                workflow_common::next_action(interface, "sdd_run_change", json!({"change_id": change_id}))
             ]
         });
         return Ok(serde_json::to_string_pretty(&result)?);
@@ -420,6 +416,7 @@ mcp__cclab-mcp__sdd_artifact_create_pre_clarifications(project_path="{project_pa
         },
     );
 
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreatePreClarifications);
 
     workflow_common::build_workflow_response(
@@ -429,6 +426,7 @@ mcp__cclab-mcp__sdd_artifact_create_pre_clarifications(project_path="{project_pa
         prompt,
         executor,
         json!({ "group_id": group_id }),
+        interface,
     )
 }
 
@@ -529,26 +527,17 @@ pub fn execute_artifact_pre_clarifications(
         "STATE.yaml".into(),
     ];
 
+    let interface = workflow_common::load_interface(project_root);
     let next_actions = if all_done {
         // All groups done → advance phase, next is sdd_run_change
         let mut sm = StateManager::load(&change_dir)?;
         sm.set_phase(StatePhase::PreClarificationsCreated)?;
         sm.save()?;
 
-        json!([{
-            "tool": "sdd_run_change",
-            "args": { "change_id": change_id },
-            "when": "immediate",
-            "executor": "mainthread"
-        }])
+        json!([workflow_common::next_action(interface, "sdd_run_change", json!({"change_id": change_id}))])
     } else {
         // More groups to clarify → loop back
-        json!([{
-            "tool": "sdd_workflow_create_pre_clarifications",
-            "args": { "change_id": change_id },
-            "when": "immediate",
-            "executor": "mainthread"
-        }])
+        json!([workflow_common::next_action(interface, "sdd_workflow_create_pre_clarifications", json!({"change_id": change_id}))])
     };
 
     let result = json!({
diff --git a/crates/cclab-sdd-mcp/src/tools/init_change.rs b/crates/cclab-sdd-mcp/src/tools/init_change.rs
index 7e7122e..da54d30 100644
--- a/crates/cclab-sdd-mcp/src/tools/init_change.rs
+++ b/crates/cclab-sdd-mcp/src/tools/init_change.rs
@@ -15,6 +15,8 @@ use cclab_sdd::Result;
 use serde_json::{json, Value};
 use std::path::Path;
 
+use super::workflow_common;
+
 /// MCP tool definition for sdd_init_change
 pub fn definition() -> ToolDefinition {
     ToolDefinition {
@@ -111,16 +113,12 @@ pub fn execute_standalone(args: &Value, project_root: &Path) -> Result<String> {
     }
     let artifacts_written = json!(written);
 
+    let interface = workflow_common::load_interface(project_root);
     let result = json!({
         "status": "ok",
         "artifacts_written": artifacts_written,
         "next_actions": [
-            {
-                "tool": next_tool,
-                "args": { "change_id": change_id },
-                "when": "immediate",
-                "executor": "mainthread"
-            }
+            workflow_common::next_action(interface, next_tool, json!({"change_id": change_id}))
         ]
     });
 
@@ -350,10 +348,9 @@ mod tests {
         assert!(parsed["next_actions"].is_array());
         let next = &parsed["next_actions"][0];
         // Both paths → routes to sdd_workflow_restructure_input
-        assert_eq!(next["tool"], "sdd_workflow_restructure_input");
+        // next_action may use "tool" (MCP) or "cli" (CLI) depending on interface config
+        assert!(next.get("tool").is_some() || next.get("cli").is_some());
         assert_eq!(next["args"]["change_id"], "my-change");
-        assert_eq!(next["when"], "immediate");
-        assert_eq!(next["executor"], "mainthread");
 
         // Verify files
         let change_dir = tmp.path().join("cclab/changes/my-change");
diff --git a/crates/cclab-sdd-mcp/src/tools/post_clarifications/create.rs b/crates/cclab-sdd-mcp/src/tools/post_clarifications/create.rs
index a2ce866..3012536 100644
--- a/crates/cclab-sdd-mcp/src/tools/post_clarifications/create.rs
+++ b/crates/cclab-sdd-mcp/src/tools/post_clarifications/create.rs
@@ -158,16 +158,12 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
     if remaining.is_empty() {
         // All groups done — don't advance phase (already PostClarificationsCreated),
         // just return next_actions: [sdd_run_change] which will route to spec
+        let interface = workflow_common::load_interface(project_root);
         let result = json!({
             "status": "phase_complete",
             "prompt": "All groups have post-clarifications. Proceeding to spec creation.",
             "group_id": null,
-            "next_actions": [{
-                "tool": "sdd_run_change",
-                "args": { "change_id": change_id },
-                "when": "immediate",
-                "executor": "mainthread"
-            }]
+            "next_actions": [workflow_common::next_action(interface, "sdd_run_change", json!({"change_id": change_id}))]
         });
         return Ok(serde_json::to_string_pretty(&result)?);
     }
@@ -239,12 +235,8 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
         "STATE.yaml".into(),
     ];
 
-    let next_actions = json!([{
-        "tool": "sdd_workflow_create_post_clarifications",
-        "args": { "change_id": change_id },
-        "when": "immediate",
-        "executor": "mainthread"
-    }]);
+    let interface = workflow_common::load_interface(project_root);
+    let next_actions = json!([workflow_common::next_action(interface, "sdd_workflow_create_post_clarifications", json!({"change_id": change_id}))]);
 
     let result = json!({
         "status": "ok",
@@ -339,6 +331,7 @@ mcp__cclab-mcp__sdd_artifact_create_post_clarifications(project_path="{project_p
         },
     );
 
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreatePostClarifications);
 
     workflow_common::build_workflow_response(
@@ -348,6 +341,7 @@ mcp__cclab-mcp__sdd_artifact_create_post_clarifications(project_path="{project_p
         prompt,
         executor,
         json!({ "group_id": group_id }),
+        interface,
     )
 }
 
@@ -506,7 +500,9 @@ mod tests {
         let parsed: Value = serde_json::from_str(&result).unwrap();
         assert_eq!(parsed["status"], "phase_complete");
         let next = &parsed["next_actions"][0];
-        assert_eq!(next["tool"], "sdd_run_change");
+        // next_action may use "tool" (MCP) or "cli" (CLI) depending on interface config
+        assert!(next.get("tool").is_some() || next.get("cli").is_some());
+        assert_eq!(next["args"]["change_id"], "all-done");
     }
 
     #[test]
@@ -593,7 +589,9 @@ mod tests {
         let result = execute_artifact(&args, tmp.path()).unwrap();
         let parsed: Value = serde_json::from_str(&result).unwrap();
         let next = &parsed["next_actions"][0];
-        assert_eq!(next["tool"], "sdd_workflow_create_post_clarifications");
+        // next_action may use "tool" (MCP) or "cli" (CLI) depending on interface config
+        assert!(next.get("tool").is_some() || next.get("cli").is_some());
+        assert_eq!(next["args"]["change_id"], "loop-test");
     }
 
     #[test]
diff --git a/crates/cclab-sdd-mcp/src/tools/reference_context/create.rs b/crates/cclab-sdd-mcp/src/tools/reference_context/create.rs
index 4d16294..637ad93 100644
--- a/crates/cclab-sdd-mcp/src/tools/reference_context/create.rs
+++ b/crates/cclab-sdd-mcp/src/tools/reference_context/create.rs
@@ -110,6 +110,8 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
     let change_dir = project_root.join("cclab/changes").join(&change_id);
     workflow_common::validate_change_dir(&change_dir, project_root)?;
 
+    let interface = workflow_common::load_interface(project_root);
+
     match common::resolve_next_group(&change_dir)? {
         GroupSubState::Create { group_id } => {
             build_create_prompt(&change_id, &group_id, &change_dir, project_root)
@@ -118,12 +120,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "group_id": group_id,
-                "next_actions": [{
-                    "tool": "sdd_workflow_review_reference_context",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_review_reference_context", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -131,12 +128,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
             let result = json!({
                 "status": "ok",
                 "group_id": group_id,
-                "next_actions": [{
-                    "tool": "sdd_workflow_revise_reference_context",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_workflow_revise_reference_context", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -149,12 +141,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
                 "status": "phase_complete",
                 "prompt": "All groups have reference context approved. Phase advanced to PostClarificationsCreated.",
                 "group_id": null,
-                "next_actions": [{
-                    "tool": "sdd_run_change",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",
-                    "executor": "mainthread"
-                }]
+                "next_actions": [workflow_common::next_action(interface, "sdd_run_change", json!({"change_id": change_id}))]
             });
             Ok(serde_json::to_string_pretty(&result)?)
         }
@@ -209,12 +196,8 @@ pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
 
     let artifacts_written = vec![format!("groups/{}/reference_context.md", group_id)];
 
-    let next_actions = json!([{
-        "tool": "sdd_workflow_create_reference_context",
-        "args": { "change_id": change_id },
-        "when": "immediate",
-        "executor": "mainthread"
-    }]);
+    let interface = workflow_common::load_interface(project_root);
+    let next_actions = json!([workflow_common::next_action(interface, "sdd_workflow_create_reference_context", json!({"change_id": change_id}))]);
 
     let result = json!({
         "status": "ok",
@@ -288,6 +271,7 @@ mcp__cclab-mcp__sdd_artifact_create_reference_context(project_path="{project_pat
         },
     );
 
+    let interface = workflow_common::load_interface(project_root);
     let executor = workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateReferenceContext);
 
     workflow_common::build_workflow_response(
@@ -297,6 +281,7 @@ mcp__cclab-mcp__sdd_artifact_create_reference_context(project_path="{project_pat
         prompt,
         executor,
         json!({ "group_id": group_id }),
+        interface,
     )
 }
 
@@ -382,7 +367,9 @@ mod tests {
         assert_eq!(parsed["status"], "ok");
         // Should redirect to review workflow
         let next = &parsed["next_actions"][0];
-        assert_eq!(next["tool"], "sdd_workflow_review_reference_context");
+        // next_action may use "tool" (MCP) or "cli" (CLI) depending on interface config
+        assert!(next.get("tool").is_some() || next.get("cli").is_some());
+        assert_eq!(next["args"]["change_id"], "rev-test");
     }
 
     #[test]
@@ -408,7 +395,9 @@ mod tests {
         assert_eq!(parsed["status"], "ok");
         // Should redirect to revise workflow
         let next = &parsed["next_actions"][0];
-        assert_eq!(next["tool"], "sdd_workflow_revise_reference_context");
+        // next_action may use "tool" (MCP) or "cli" (CLI) depending on interface config
+        assert!(next.get("tool").is_some() || next.get("cli").is_some());
+        assert_eq!(next["args"]["change_id"], "revise-test");
     }
 
     #[test]
diff --git a/crates/cclab-sdd-mcp/src/tools/reference_context/review.rs b/crates/cclab-sdd-mcp/src/tools/reference_context/review.rs
index 56aa3e0..ec8b04d 100644
--- a/crates/cclab-sdd-mcp/src/tools/reference_context/review.rs
+++ b/crates/cclab-sdd-mcp/src/tools/reference_context/review.rs
@@ -115,6 +115,8 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
     let change_dir = project_root.join("cclab/changes").join(&change_id);
     workflow_common::validate_change_dir(&change_dir, project_root)?;
 
+    let interface = workflow_common::load_interface(project_root);
+
     // Resolve current group — should be in Review sub-state
     match common::resolve_next_group(&change_dir)? {
         common::GroupSubState::Review { group_id } => {
@@ -126,12 +128,7 @@ pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
                 "status": "ok",
                 "prompt": "Group is not in Review sub-state. Redirecting to router.",
                 "group_id": null,
-                "next_actions": [{
-                    "tool": "sdd_workflow_create_reference_context",
-                    "args": { "change_id": change_id },
-                    "when": "immediate",

... truncated (2458 more lines)
```
