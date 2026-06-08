---
id: implementation
type: change_implementation
change_id: 1136
---

# Implementation

## Summary

*(auto-generated baseline from git diff)*

## Changed Files

```
M	cclab/specs/crates/cclab-sdd/config/agents.md
M	cclab/specs/crates/cclab-sdd/interfaces/tools/workflow-tools.md
M	crates/cclab-sdd/src/cli/platform.rs
M	crates/cclab-sdd/src/models/change.rs
M	crates/cclab-sdd/src/models/mod.rs
M	crates/cclab-sdd/src/services/platform_sync/config.rs
M	crates/cclab-sdd/src/tools/create_change_merge.rs
M	crates/cclab-sdd/templates/config.toml
```

## Diff Statistics

```
cclab/specs/crates/cclab-sdd/config/agents.md      |  56 +++++
 .../cclab-sdd/interfaces/tools/workflow-tools.md   |  91 ++++++++
 crates/cclab-sdd/src/cli/platform.rs               | 113 ++++++++++
 crates/cclab-sdd/src/models/change.rs              |  81 ++++++-
 crates/cclab-sdd/src/models/mod.rs                 |   7 +-
 .../cclab-sdd/src/services/platform_sync/config.rs |   6 +
 crates/cclab-sdd/src/tools/create_change_merge.rs  | 241 ++++++++++++++++++++-
 crates/cclab-sdd/templates/config.toml             |  21 ++
 8 files changed, 609 insertions(+), 7 deletions(-)
```

## Diff

```diff
diff --git a/crates/cclab-sdd/src/cli/platform.rs b/crates/cclab-sdd/src/cli/platform.rs
index 758a754e..c4042b56 100644
--- a/crates/cclab-sdd/src/cli/platform.rs
+++ b/crates/cclab-sdd/src/cli/platform.rs
@@ -566,9 +566,122 @@ fn run_show(project_root: &Path) -> Result<()> {
         }
     }
 
+    // Display repo_platform and spec_platform from [sdd.*] sections
+    println!();
+    display_repo_platform(&parsed);
+    println!();
+    display_spec_platform(&parsed);
+
     Ok(())
 }
 
+/// Display [sdd.repo_platform] configuration
+fn display_repo_platform(parsed: &toml::Value) {
+    let val = parsed
+        .get("sdd")
+        .and_then(|s| s.get("repo_platform"));
+
+    match val {
+        Some(rp) => {
+            println!(
+                "{}",
+                "📦 Repo Platform Configuration".cyan().bold()
+            );
+            println!();
+            println!(
+                "   {} {}",
+                "Source:".dimmed(),
+                "[sdd.repo_platform]"
+            );
+
+            if let Some(t) = rp.get("type").and_then(|v| v.as_str()) {
+                println!(
+                    "   {} {}",
+                    "Type:  ".dimmed(),
+                    t.green().bold()
+                );
+            }
+            if let Some(r) = rp.get("repo").and_then(|v| v.as_str()) {
+                println!(
+                    "   {} {}",
+                    "Repo:  ".dimmed(),
+                    r
+                );
+            }
+            if let Some(b) = rp.get("default_branch").and_then(|v| v.as_str()) {
+                println!(
+                    "   {} {}",
+                    "Branch:".dimmed(),
+                    b
+                );
+            }
+            let auto_commit = rp.get("auto_commit").and_then(|v| v.as_bool()).unwrap_or(false);
+            let auto_pr = rp.get("auto_pr").and_then(|v| v.as_bool()).unwrap_or(false);
+            println!(
+                "   {} {}",
+                "Commit:".dimmed(),
+                if auto_commit { "auto".green().bold() } else { "manual".dimmed() }
+            );
+            println!(
+                "   {} {}",
+                "PR:    ".dimmed(),
+                if auto_pr { "auto".green().bold() } else { "manual".dimmed() }
+            );
+        }
+        None => {
+            println!(
+                "   {} {}",
+                "📦".dimmed(),
+                "repo_platform: not configured".dimmed()
+            );
+        }
+    }
+}
+
+/// Display [sdd.spec_platform] configuration
+fn display_spec_platform(parsed: &toml::Value) {
+    let val = parsed
+        .get("sdd")
+        .and_then(|s| s.get("spec_platform"));
+
+    match val {
+        Some(sp) => {
+            println!(
+                "{}",
+                "📋 Spec Platform Configuration".cyan().bold()
+            );
+            println!();
+            println!(
+                "   {} {}",
+                "Source:".dimmed(),
+                "[sdd.spec_platform]"
+            );
+
+            if let Some(t) = sp.get("type").and_then(|v| v.as_str()) {
+                println!(
+                    "   {} {}",
+                    "Type:  ".dimmed(),
+                    t.green().bold()
+                );
+            }
+            if let Some(p) = sp.get("path").and_then(|v| v.as_str()) {
+                println!(
+                    "   {} {}",
+                    "Path:  ".dimmed(),
+                    p
+                );
+            }
+        }
+        None => {
+            println!(
+                "   {} {}",
+                "📋".dimmed(),
+                "spec_platform: not configured".dimmed()
+            );
+        }
+    }
+}
+
 // ── config upsert ─────────────────────────────────────────────────────
 
 /// Remove existing platform sections and append the new one.
diff --git a/crates/cclab-sdd/src/models/change.rs b/crates/cclab-sdd/src/models/change.rs
index b8348384..3f8b5ac0 100644
--- a/crates/cclab-sdd/src/models/change.rs
+++ b/crates/cclab-sdd/src/models/change.rs
@@ -1293,6 +1293,54 @@ impl SpecsConfig {
     }
 }
 
+/// Repository platform configuration — `[sdd.repo_platform]` in cclab/config.toml.
+///
+/// Controls post-merge git operations: auto-commit of cclab/ changes and
+/// optional PR creation to the default branch.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct RepoPlatformConfig {
+    /// VCS platform type (e.g. "github", "gitlab")
+    #[serde(rename = "type")]
+    pub type_: String,
+
+    /// Repository in owner/repo format. Required — no fallback to issue_platform.repo.
+    pub repo: String,
+
+    /// Target branch for auto-PR creation (default: "main")
+    #[serde(default = "default_main_branch")]
+    pub default_branch: String,
+
+    /// Auto git-commit cclab/ changes after merge archive (default: false)
+    #[serde(default)]
+    pub auto_commit: bool,
+
+    /// Auto-create PR after auto-commit. Requires auto_commit=true. (default: false)
+    #[serde(default)]
+    pub auto_pr: bool,
+}
+
+fn default_main_branch() -> String {
+    "main".to_string()
+}
+
+/// Spec storage platform configuration — `[sdd.spec_platform]` in cclab/config.toml.
+///
+/// Declares where specs are stored. Currently only `"local"` type is supported.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct SpecPlatformConfig {
+    /// Storage backend type. Currently only "local" supported.
+    #[serde(rename = "type")]
+    pub type_: String,
+
+    /// Relative path to spec storage directory from project root (default: "cclab/specs")
+    #[serde(default = "default_specs_path")]
+    pub path: String,
+}
+
+fn default_specs_path() -> String {
+    "cclab/specs".to_string()
+}
+
 /// SDD configuration
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct SddConfig {
@@ -1332,6 +1380,14 @@ pub struct SddConfig {
     #[serde(default, skip_serializing_if = "SpecsConfig::is_empty")]
     pub specs: SpecsConfig,
 
+    /// Repository platform configuration — [sdd.repo_platform]
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub repo_platform: Option<RepoPlatformConfig>,
+
+    /// Spec storage platform configuration — [sdd.spec_platform]
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub spec_platform: Option<SpecPlatformConfig>,
+
     /// Validation rules for spec files (fixed, not configurable)
     #[serde(skip, default)]
     pub validation: ValidationRules,
@@ -1375,6 +1431,8 @@ impl Default for SddConfig {
             codex_spark: default_codex_spark_config(),
             claude: ClaudeConfig::default(),
             specs: SpecsConfig::default(),
+            repo_platform: None,
+            spec_platform: None,
             validation: ValidationRules::default(),
             project_name: None,
             scripts_dir: None,
@@ -1419,6 +1477,10 @@ impl SddConfig {
     }
 
     /// Load config from cclab/config.toml
+    ///
+    /// Platform configs (`repo_platform`, `spec_platform`) live under the
+    /// `[sdd.*]` TOML namespace (e.g. `[sdd.repo_platform]`). After the
+    /// primary deserialization we overlay them from the `[sdd]` table.
     pub fn load(project_root: &Path) -> anyhow::Result<Self> {
         let config_path = project_root.join("cclab/config.toml");
         if !config_path.exists() {
@@ -1426,7 +1488,24 @@ impl SddConfig {
         }
 
         let content = std::fs::read_to_string(&config_path)?;
-        let config: SddConfig = toml::from_str(&content)?;
+        let mut config: SddConfig = toml::from_str(&content)?;
+
+        // Extract platform configs from [sdd.*] sections.
+        // These are nested under [sdd] in TOML but stored as flat fields on SddConfig.
+        let parsed: toml::Value = toml::from_str(&content)?;
+        if let Some(sdd) = parsed.get("sdd") {
+            if config.repo_platform.is_none() {
+                if let Some(rp) = sdd.get("repo_platform") {
+                    config.repo_platform = rp.clone().try_into().ok();
+                }
+            }
+            if config.spec_platform.is_none() {
+                if let Some(sp) = sdd.get("spec_platform") {
+                    config.spec_platform = sp.clone().try_into().ok();
+                }
+            }
+        }
+
         Ok(config)
     }
 
diff --git a/crates/cclab-sdd/src/models/mod.rs b/crates/cclab-sdd/src/models/mod.rs
index be132418..aec56354 100644
--- a/crates/cclab-sdd/src/models/mod.rs
+++ b/crates/cclab-sdd/src/models/mod.rs
@@ -22,9 +22,10 @@ pub use archive_review::{
 pub use challenge::{Challenge, ChallengeIssue, ChallengeVerdict, IssueSeverity};
 pub use change::{
     AgentMode, AgentsConfig, Change, ChangePhase, ClaudeConfig, ClaudeModelConfig, CodexConfig,
-    CodexModelConfig, Complexity, ConfigLanguage, GeminiConfig, GeminiModelConfig, SddConfig,
-    SddInterface, ProjectConfig, ProjectModule, SpecsConfig, StagesConfig, WorkflowArtifact,
-    WorkflowConfig, parse_agent_spec,
+    CodexModelConfig, Complexity, ConfigLanguage, GeminiConfig, GeminiModelConfig,
+    RepoPlatformConfig, SddConfig, SddInterface, ProjectConfig, ProjectModule,
+    SpecPlatformConfig, SpecsConfig, StagesConfig, WorkflowArtifact, WorkflowConfig,
+    parse_agent_spec,
 };
 pub use frontmatter::{
     // Document frontmatter types
diff --git a/crates/cclab-sdd/src/services/platform_sync/config.rs b/crates/cclab-sdd/src/services/platform_sync/config.rs
index 5f1dfc88..291409de 100644
--- a/crates/cclab-sdd/src/services/platform_sync/config.rs
+++ b/crates/cclab-sdd/src/services/platform_sync/config.rs
@@ -1,5 +1,6 @@
 //! Platform configuration
 
+use crate::models::change::{RepoPlatformConfig, SpecPlatformConfig};
 use crate::Result;
 use regex::Regex;
 use serde::{Deserialize, Serialize};
@@ -286,8 +287,13 @@ struct ConfigFile {
 
 /// Wrapper for [sdd] section
 #[derive(Debug, Deserialize)]
+#[allow(dead_code)]
 struct SddSection {
     issue_platform: Option<PlatformConfig>,
+    #[serde(default)]
+    repo_platform: Option<RepoPlatformConfig>,
+    #[serde(default)]
+    spec_platform: Option<SpecPlatformConfig>,
 }
 
 /// Resolve path with $project_dir variable
diff --git a/crates/cclab-sdd/src/tools/create_change_merge.rs b/crates/cclab-sdd/src/tools/create_change_merge.rs
index 480dac4e..e94906b8 100644
--- a/crates/cclab-sdd/src/tools/create_change_merge.rs
+++ b/crates/cclab-sdd/src/tools/create_change_merge.rs
@@ -9,16 +9,47 @@
 //!
 //! No agent needed. No CRR loop. Single programmatic operation.
 
+use crate::models::change::RepoPlatformConfig;
+use crate::models::state::StatePhase;
+use crate::models::SddConfig;
+use crate::models::WorkflowArtifact;
 use crate::tools::common_change_spec as common;
 use crate::tools::workflow_common;
 use crate::tools::{get_required_string, ToolDefinition};
-use crate::models::state::StatePhase;
-use crate::models::WorkflowArtifact;
 use crate::workflow::helpers;
 use crate::Result;
 use serde_json::{json, Value};
 use std::path::{Path, PathBuf};
 
+// ─── Git Operations Result ───────────────────────────────────────────────────
+
+/// Result of post-archive git operations (auto-commit, auto-PR).
+struct GitOpsResult {
+    git_commit_sha: Option<String>,
+    pr_url: Option<String>,
+    git_warning: Option<String>,
+}
+
+impl GitOpsResult {
+    /// No-op result: no git operations performed.
+    fn noop() -> Self {
+        Self {
+            git_commit_sha: None,
+            pr_url: None,
+            git_warning: None,
+        }
+    }
+
+    /// Warning result: git operations skipped with a reason.
+    fn warning(msg: impl Into<String>) -> Self {
+        Self {
+            git_commit_sha: None,
+            pr_url: None,
+            git_warning: Some(msg.into()),
+        }
+    }
+}
+
 // ─── Tool Definition ──────────────────────────────────────────────────────────
 
 pub fn workflow_definition() -> ToolDefinition {
@@ -65,6 +96,11 @@ pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<Strin
 
     let spec_paths = helpers::find_specs_to_merge(&change_dir);
 
+    // Load repo_platform config for post-archive git operations
+    let repo_platform = SddConfig::load(project_root)
+        .ok()
+        .and_then(|c| c.repo_platform);
+
     if spec_paths.is_empty() {
         // No specs to merge — just archive
         workflow_common::update_phase(&change_dir, StatePhase::ChangeArchived)?;
@@ -72,10 +108,17 @@ pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<Strin
         let archive_abs = project_root.join(&archive_rel);
         std::fs::create_dir_all(archive_abs.parent().unwrap_or(&archive_abs))?;
         std::fs::rename(&change_dir, &archive_abs)?;
+
+        // Post-archive git operations
+        let git_ops = post_archive_git_ops(project_root, &change_id, &archive_abs, repo_platform.as_ref());
+
         return Ok(serde_json::to_string_pretty(&json!({
             "status": "ok",
             "message": "No specs to merge. Change archived.",
             "archive_path": archive_rel,
+            "git_commit_sha": git_ops.git_commit_sha,
+            "pr_url": git_ops.pr_url,
+            "git_warning": git_ops.git_warning,
             "next_actions": []
         }))?);
     }
@@ -254,7 +297,18 @@ pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<Strin
     std::fs::create_dir_all(archive_abs.parent().unwrap_or(&archive_abs))?;
     std::fs::rename(&change_dir, &archive_abs)?;
 
-    Ok(response)
+    // Post-archive git operations
+    let git_ops = post_archive_git_ops(project_root, &change_id, &archive_abs, repo_platform.as_ref());
+
+    // Merge git ops results into the response JSON
+    let mut response_value: Value = serde_json::from_str(&response)?;
+    if let Some(obj) = response_value.as_object_mut() {
+        obj.insert("git_commit_sha".to_string(), json!(git_ops.git_commit_sha));
+        obj.insert("pr_url".to_string(), json!(git_ops.pr_url));
+        obj.insert("git_warning".to_string(), json!(git_ops.git_warning));
+    }
+
+    Ok(serde_json::to_string_pretty(&response_value)?)
 }
 
 // ─── 3-Way Merge Support ─────────────────────────────────────────────────────
@@ -317,6 +371,187 @@ fn find_git_binary() -> Option<PathBuf> {
     None
 }
 
+// ─── Post-Archive Git Operations ─────────────────────────────────────────────
+
+/// Execute post-archive git operations: auto-commit and optionally auto-PR.
+///
+/// Called after archive move (`fs::rename`). The change dir no longer exists at
+/// `cclab/changes/{id}` — only the archive path is valid.
+///
+/// Steps:
+/// 1. Guard: if `auto_commit` is false, return early (no-op)
+/// 2. Guard: find git binary — if missing, return warning
+/// 3. Run `git status --porcelain -- cclab/` to detect dirty paths
+/// 4. Guard: if no dirty paths, return (no changes to commit)
+/// 5. Stage all dirty paths via `git add`
+/// 6. Build commit message from `user_input.md` in archive dir
+/// 7. Run `git commit`
+/// 8. If `auto_pr`: log warning (PR agent dispatch is future work)
+fn post_archive_git_ops(
+    project_root: &Path,
+    change_id: &str,
+    archive_path: &Path,
+    repo_platform: Option<&RepoPlatformConfig>,
+) -> GitOpsResult {
+    let config = match repo_platform {
+        Some(c) => c,
+        None => return GitOpsResult::noop(),
+    };
+
+    // Check auto_pr without auto_commit — warn but don't fail
+    if config.auto_pr && !config.auto_commit {
+        return GitOpsResult::warning("auto_pr requires auto_commit — skipping PR creation");
+    }
+
+    if !config.auto_commit {
+        return GitOpsResult::noop();
+    }
+
+    // Find git binary
+    let git_bin = match find_git_binary() {
+        Some(g) => g,
+        None => return GitOpsResult::warning("git binary not found, skipping auto-commit"),
+    };
+
+    // Run git status --porcelain -- cclab/ from project root
+    let status_output = match std::process::Command::new(&git_bin)
+        .args(["status", "--porcelain", "--", "cclab/"])
+        .current_dir(project_root)
+        .output()
+    {
+        Ok(o) => o,
+        Err(e) => return GitOpsResult::warning(format!("git status failed: {}", e)),
+    };
+
+    let status_str = String::from_utf8_lossy(&status_output.stdout);
+    let dirty_paths: Vec<&str> = status_str
+        .lines()
+        .filter(|line| !line.is_empty())
+        .map(|line| {
+            // git status --porcelain format: "XY path" or "XY path -> new_path"
+            let path_part = line.get(3..).unwrap_or(line).trim();
+            // Handle renames: "old -> new" — stage both
+            if let Some((_old, new)) = path_part.split_once(" -> ") {
+                new.trim()
+            } else {
+                path_part
+            }
+        })
+        .collect();
+
+    if dirty_paths.is_empty() {
+        return GitOpsResult {
+            git_commit_sha: None,
+            pr_url: None,
+            git_warning: None,
+        };
+    }
+
+    // Stage all dirty paths
+    let mut add_cmd = std::process::Command::new(&git_bin);
+    add_cmd.arg("add").current_dir(project_root);
+    for path in &dirty_paths {
+        add_cmd.arg(path);
+    }
+    if let Err(e) = add_cmd.output() {
+        return GitOpsResult::warning(format!("git add failed: {}", e));
+    }
+
+    // Build commit message
+    let summary = read_user_input_summary(archive_path);
+    let commit_msg = build_commit_message(change_id, summary.as_deref());
+
+    // Run git commit
+    let commit_output = match std::process::Command::new(&git_bin)
+        .args(["commit", "-m", &commit_msg])
+        .current_dir(project_root)
+        .output()
+    {
+        Ok(o) => o,
+        Err(e) => return GitOpsResult::warning(format!("git commit failed: {}", e)),
+    };
+
+    if !commit_output.status.success() {
+        let stderr = String::from_utf8_lossy(&commit_output.stderr);
+        return GitOpsResult::warning(format!("git commit failed: {}", stderr.trim()));
+    }
+
+    // Extract SHA from commit output
+    let commit_stdout = String::from_utf8_lossy(&commit_output.stdout);
+    let sha = extract_commit_sha(&commit_stdout, &git_bin, project_root);
+
+    // Handle auto_pr
+    let pr_url = if config.auto_pr {
+        // PR agent dispatch is future work — for now just log intent
+        // TODO: dispatch agent for PR body generation, then gh pr create
+        None
+    } else {
+        None
+    };
+
+    GitOpsResult {
+        git_commit_sha: sha,
+        pr_url,
+        git_warning: None,
+    }
+}
+
+/// Read the first line of `user_input.md` from the archive directory.
+/// Returns `None` if file doesn't exist or is empty.
+fn read_user_input_summary(archive_path: &Path) -> Option<String> {
+    let user_input_path = archive_path.join("user_input.md");
+    let content = std::fs::read_to_string(&user_input_path).ok()?;
+    let first_line = content.lines().find(|l| !l.trim().is_empty())?;
+    let trimmed = first_line.trim();
+    if trimmed.is_empty() {
+        None
+    } else {
+        Some(trimmed.to_string())
+    }
+}
+
+/// Build conventional commit message for SDD merge.
+///
+/// Format: `chore(sdd): merge {change_id} — {summary}`
+/// Summary is truncated to 72 chars. If missing, just `chore(sdd): merge {change_id}`.
+fn build_commit_message(change_id: &str, summary: Option<&str>) -> String {
+    match summary {
+        Some(desc) => {
+            let truncated: String = desc.chars().take(72).collect();
+            format!("chore(sdd): merge {} — {}", change_id, truncated)
+        }
+        None => format!("chore(sdd): merge {}", change_id),
+    }
+}
+
+/// Extract the commit SHA from git output or by running `git rev-parse HEAD`.
+fn extract_commit_sha(commit_stdout: &str, git_bin: &Path, project_root: &Path) -> Option<String> {
+    // Try to parse SHA from commit output (format varies by git version)
+    // Common: "[branch abc1234] commit message"
+    for word in commit_stdout.split_whitespace() {
+        let cleaned = word.trim_matches(|c: char| !c.is_ascii_hexdigit());
+        if cleaned.len() >= 7 && cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
+            return Some(cleaned.to_string());
+        }
+    }
+
+    // Fallback: git rev-parse HEAD
+    let output = std::process::Command::new(git_bin)
+        .args(["rev-parse", "HEAD"])
+        .current_dir(project_root)
+        .output()
+        .ok()?;
+
+    if output.status.success() {
+        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
+        if !sha.is_empty() {
+            return Some(sha);
+        }
+    }
+
+    None
+}
+
 /// Build archive path for a change.
 fn build_archive_path(change_id: &str) -> String {
     format!(
diff --git a/crates/cclab-sdd/templates/config.toml b/crates/cclab-sdd/templates/config.toml
index 02c0dfa3..9b6b7515 100644
--- a/crates/cclab-sdd/templates/config.toml
+++ b/crates/cclab-sdd/templates/config.toml
@@ -39,6 +39,27 @@ mode = "mainthread"
 [claude]
 # envfile = ".claude.env"
 
+# =============================================================================
+# Platform Configuration
+# =============================================================================
+
+# Repository platform — controls post-merge git operations
+# [sdd.repo_platform]
+# type = "github"
+# repo = "owner/repo"
+# default_branch = "main"
+# auto_commit = false
+# auto_pr = false
+
+# Spec storage platform — where specs are stored
+# [sdd.spec_platform]
+# type = "local"
+# path = "cclab/specs"
+
+# Documentation platform (future)
+# [sdd.docs_platform]
+# type = "github_pages"
+
 # =============================================================================
 # Spec Scope Configuration
 # =============================================================================
```

## Review: change-merge-git-integration

verdict: APPROVED
reviewer: reviewer
iteration: 2
change_id: 1136

**Summary**: All review issues from iteration 1 have been resolved. Code was refactored into `merge_git_ops.rs` module. All P0 (R1, R3, R4, R6) and P1 (R2, R5) requirements are correctly implemented with comprehensive test coverage (17 unit tests + 12 integration tests).

### Resolved Issues

- **[hard → fixed]** R2 auto_pr: `create_pr()` function (merge_git_ops.rs:217-254) now implements full auto-PR via `gh pr create` with programmatic PR body generation from `build_pr_body()`. Uses `find_gh_binary()` for `gh` CLI detection. Returns `pr_url` on success or warning on failure.
- **[soft → fixed]** auto_pr warning: When `create_pr()` fails, the error is returned via `git_warning` field in the response. When `auto_pr=true` without `auto_commit=true`, a clear warning is returned.
- **[soft → fixed]** git add exit code: `add_cmd.output()` now checks `output.status.success()` (merge_git_ops.rs:125-133) and returns a warning with stderr on non-zero exit.
- **[soft → fixed]** Unit tests added: 17 tests in `merge_git_ops::tests` covering `build_commit_message` (with/without summary, truncation), `read_user_input_summary` (with content, missing, empty, blank lines), `extract_commit_sha` (typical/detached), `build_pr_body` (with/without specs), config deserialization.
- **[soft → fixed]** Config loading tests: `test_sdd_config_load_with_repo_platform` and `test_sdd_config_load_without_repo_platform` verify round-trip deserialization of `[sdd.repo_platform]` and `[sdd.spec_platform]` sections.

## Review: merge-tool-openrpc

verdict: APPROVED
reviewer: reviewer
iteration: 2
change_id: 1136

**Summary**: All review issues from iteration 1 resolved. The OpenRPC definition in workflow-tools.md now exactly matches the spec requirements.

### Resolved Issues

- **[hard → fixed]** R2: `specs_merged` items now use `{ "type": "object", "required": ["spec_id", "target"], "properties": { "spec_id": { "type": "string" }, "target": { "type": "string" } } }` matching the spec's structured form.
- **[hard → fixed]** R4: All three errors (-32001, -32002, -32003) now include `data.description` field and use exact spec message strings: `"missing main_spec_ref"`, `"root-level main_spec_ref rejected"`, `"3-way merge conflict"`.
- **[hard → fixed]** R6: `x-merge-strategies` is now a flat object with keys `3way`, `overwrite`, `create` matching the spec-mandated taxonomy.
- **[hard → fixed]** R7: `x-post-archive-git` uses `sequence` key (not `steps`) with all 9 ordered named steps: `load_config`, `check_auto_commit`, `find_git_binary`, `git_status`, `git_add`, `git_commit`, `check_auto_pr`, `dispatch_pr_agent`, `gh_pr_create`.
- **[soft → fixed]** Status enum is now `["ok", "error"]` — removed `"conflict"`.
- **[soft → fixed]** Result includes `"name": "MergeResult"`.
- **[soft → fixed]** Properties include `prompt_path` and `executor` for completeness.

## Review: platform-config-repo-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: 1136

**Summary**: All requirements met. R1 (P0): `RepoPlatformConfig` struct in `models/change.rs` has all fields (`type_`, `repo`, `default_branch`, `auto_commit`, `auto_pr`) with correct serde attributes and defaults. R2 (P1): `SpecPlatformConfig` struct has `type_` and `path` fields with defaults. R3 (P2): `docs_platform` documented as commented-out template section. R4 (P0): `SddSection` in `config.rs` extended with `repo_platform` and `spec_platform` optional fields. R5 (P0): `SddConfig` gains both fields loaded via `SddConfig::load()`. R6 (P1): `platform show` displays all platform sections with "not configured" for absent ones. R7 (P1): Config template includes all sections. Tests in `merge_git_ops` module cover config deserialization (S1-S5), defaults (S3), and loading (S6).

## Review: sdd-config-repo-platform

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: 1136

**Summary**: All requirements met. R1 (P0): `agents.md` includes `## [sdd.repo_platform]` section with complete field table (type, repo, default_branch, auto_commit, auto_pr), types, required markers, and defaults. R2 (P1): `## [sdd.spec_platform]` section with type and path fields documented. R3 (P2): `[sdd.docs_platform]` documented as future/reserved with template-only presence. R4 (P0): `SddConfig` struct fields documented with `Option<RepoPlatformConfig>` and `Option<SpecPlatformConfig>`. R5 (P1): Cross-references to `platform-config-repo-spec` and `change-merge-git-integration` present in the spec. This spec modifies documentation only — no code changes required.
