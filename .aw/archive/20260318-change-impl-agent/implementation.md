---
id: implementation
type: change_implementation
change_id: change-impl-agent
---

# Implementation

## Summary

New CodeAgent in crates/cclab-agent across 6 files: (1) agents/code_agent/mod.rs (new, 638 lines): Autonomous code-generation agent implementing the Agent trait; CodeAgentConfig (model, max_tokens, temperature, max_revisions, base_branch, branch_prefix), orchestrates task decomposition → CRR-gated LLM code generation (InlineLLMAgent) → XML parsing → PlatformIntegration branch/commit/PR; CodeAgentBuilder with fluent API requiring provider and platform; 5 unit tests covering success, malformed XML, max-revisions-exceeded, and missing builder params; (2) agents/code_agent/parser.rs (new, 191 lines): parse_file_blocks() extracts <file path="...">...</file> XML blocks from LLM responses; supports single and double quotes; returns MalformedLLMResponse on empty result, missing path attr, unclosed tags; 8 unit tests; (3) agents/code_agent/tasks.rs (new, 344 lines): decompose_spec() parses ## Changes section into ImplementationTask list with topological sort (DataModel < Logic < Integration < Test); TaskCategory and TaskAction enums; infer_category() heuristic on file path; 10 unit tests; (4) agents/mod.rs: added pub mod code_agent and re-exports for CodeAgent, CodeAgentBuilder, CodeAgentConfig, FileBlock, ImplementationTask, TaskAction, TaskCategory; (5) integrations/mod.rs: added CommitFile, CreatedBranch, CreatedCommit, PullRequestParams, CreatedPullRequest source-control types; extended PlatformIntegration trait with default-error create_branch, create_commit, create_pull_request methods; (6) integrations/github.rs & gitlab.rs: implemented create_branch/create_commit/create_pull_request for both platforms using Git Data API (GitHub) and repository commits API (GitLab); added resolve_ref_sha helper and patch() HTTP method on GitHub; added private response structs; (7) error.rs: added MalformedLLMResponse(String) and PlatformError(String) variants; (8) lib.rs: re-exported all new public types at crate root.

## Diff

```diff
diff --git a/crates/cclab-agent/src/agents/mod.rs b/crates/cclab-agent/src/agents/mod.rs
index 1d38c4a5..30625130 100644
--- a/crates/cclab-agent/src/agents/mod.rs
+++ b/crates/cclab-agent/src/agents/mod.rs
@@ -2,6 +2,7 @@
 
 mod analyst;
 pub mod change_spec;
+pub mod code_agent;
 mod coding;
 pub mod crr;
 pub mod reference_context;
@@ -10,6 +11,10 @@ mod restructure;
 
 pub use analyst::{AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig};
 pub use change_spec::{ChangeSpecAgent, ChangeSpecAgentBuilder, ChangeSpecAgentConfig, ChangeSpecInput};
+pub use code_agent::{
+    CodeAgent, CodeAgentBuilder, CodeAgentConfig, FileBlock, ImplementationTask, TaskAction,
+    TaskCategory,
+};
 pub use coding::{CodingAgent, CodingAgentBuilder, CodingAgentConfig};
 pub use crr::{CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType};
 pub use reference_context::{
diff --git a/crates/cclab-agent/src/error.rs b/crates/cclab-agent/src/error.rs
index 6591d18c..3e1178a5 100644
--- a/crates/cclab-agent/src/error.rs
+++ b/crates/cclab-agent/src/error.rs
@@ -34,6 +34,12 @@ pub enum NovaError {
     #[error("Maximum revisions exceeded: {0}")]
     MaxRevisionsExceeded(u32),
 
+    #[error("Malformed LLM response: {0}")]
+    MalformedLLMResponse(String),
+
+    #[error("Platform integration error: {0}")]
+    PlatformError(String),
+
     #[error("Context overflow: token budget exceeded")]
     ContextOverflow,
 
diff --git a/crates/cclab-agent/src/integrations/github.rs b/crates/cclab-agent/src/integrations/github.rs
index 2ea3718a..3540a353 100644
--- a/crates/cclab-agent/src/integrations/github.rs
+++ b/crates/cclab-agent/src/integrations/github.rs
@@ -1,7 +1,8 @@
 //! GitHub integration for issue tracking.
 
 use super::{
-    Issue, IssueComment, IssueFilter, IssueSummary, IssueState, PlatformIntegration, PostedComment,
+    CommitFile, CreatedBranch, CreatedCommit, CreatedPullRequest, Issue, IssueComment, IssueFilter,
+    IssueSummary, IssueState, PlatformIntegration, PostedComment, PullRequestParams,
 };
 use crate::error::{NovaError, NovaResult};
 use crate::tools::{Tool, ToolParameter};
@@ -71,6 +72,17 @@ impl GitHubIntegration {
             .map_err(|e| NovaError::ApiError(format!("Failed to parse GitHub response: {}", e)))
     }
 
+    /// Resolve a branch name or SHA to a commit SHA.
+    async fn resolve_ref_sha(&self, git_ref: &str) -> NovaResult<String> {
+        // If it already looks like a SHA (40 hex chars), use it directly.
+        if git_ref.len() == 40 && git_ref.chars().all(|c| c.is_ascii_hexdigit()) {
+            return Ok(git_ref.to_string());
+        }
+        let url = self.api_url(&format!("git/refs/heads/{}", git_ref));
+        let data: GitHubRef = self.get(&url).await?;
+        Ok(data.object.sha)
+    }
+
     /// Make an authenticated POST request.
     async fn post<T: for<'de> Deserialize<'de>>(
         &self,
@@ -101,6 +113,37 @@ impl GitHubIntegration {
             .json_as()
             .map_err(|e| NovaError::ApiError(format!("Failed to parse GitHub response: {}", e)))
     }
+
+    /// Make an authenticated PATCH request (used for updating refs).
+    async fn patch<T: for<'de> Deserialize<'de>>(
+        &self,
+        url: &str,
+        body: serde_json::Value,
+    ) -> NovaResult<T> {
+        let request = RequestBuilder::new(HttpMethod::Patch, url)
+            .header("Authorization", format!("Bearer {}", self.token))
+            .header("Accept", "application/vnd.github.v3+json")
+            .header("User-Agent", "cclab-nova")
+            .json_value(body);
+
+        let response = self
+            .client
+            .execute_builder(request)
+            .await
+            .map_err(|e| NovaError::HttpError(format!("GitHub API request failed: {}", e)))?;
+
+        if !response.is_success() {
+            let body = response.text().unwrap_or_default();
+            return Err(NovaError::ApiError(format!(
+                "GitHub API error {}: {}",
+                response.status_code, body
+            )));
+        }
+
+        response
+            .json_as()
+            .map_err(|e| NovaError::ApiError(format!("Failed to parse GitHub response: {}", e)))
+    }
 }
 
 #[async_trait]
@@ -221,6 +264,110 @@ impl PlatformIntegration for GitHubIntegration {
         })
     }
 
+    async fn create_branch(
+        &self,
+        branch_name: &str,
+        from_ref: &str,
+    ) -> NovaResult<CreatedBranch> {
+        // Resolve from_ref to a SHA.
+        let sha = self.resolve_ref_sha(from_ref).await?;
+
+        let url = self.api_url("git/refs");
+        let body = serde_json::json!({
+            "ref": format!("refs/heads/{}", branch_name),
+            "sha": sha
+        });
+
+        let _: serde_json::Value = self.post(&url, body).await?;
+
+        Ok(CreatedBranch {
+            name: branch_name.to_string(),
+            sha,
+        })
+    }
+
+    async fn create_commit(
+        &self,
+        branch: &str,
+        message: &str,
+        files: &[CommitFile],
+    ) -> NovaResult<CreatedCommit> {
+        // 1. Get the current commit SHA on the branch.
+        let ref_url = self.api_url(&format!("git/refs/heads/{}", branch));
+        let ref_data: GitHubRef = self.get(&ref_url).await?;
+        let parent_sha = ref_data.object.sha.clone();
+
+        // 2. Get the tree SHA of the parent commit.
+        let commit_url = self.api_url(&format!("git/commits/{}", parent_sha));
+        let parent_commit: GitHubCommitObject = self.get(&commit_url).await?;
+        let base_tree_sha = parent_commit.tree.sha;
+
+        // 3. Create blobs for each file.
+        let mut tree_entries: Vec<serde_json::Value> = Vec::new();
+        for file in files {
+            let blob_url = self.api_url("git/blobs");
+            let blob_body = serde_json::json!({
+                "content": file.content,
+                "encoding": "utf-8"
+            });
+            let blob: GitHubBlob = self.post(&blob_url, blob_body).await?;
+            tree_entries.push(serde_json::json!({
+                "path": file.path,
+                "mode": "100644",
+                "type": "blob",
+                "sha": blob.sha
+            }));
+        }
+
+        // 4. Create a new tree.
+        let tree_url = self.api_url("git/trees");
+        let tree_body = serde_json::json!({
+            "base_tree": base_tree_sha,
+            "tree": tree_entries
+        });
+        let new_tree: GitHubTree = self.post(&tree_url, tree_body).await?;
+
+        // 5. Create the commit.
+        let commits_url = self.api_url("git/commits");
+        let commit_body = serde_json::json!({
+            "message": message,
+            "tree": new_tree.sha,
+            "parents": [parent_sha]
+        });
+        let new_commit: GitHubCommitCreated = self.post(&commits_url, commit_body).await?;
+
+        // 6. Update the branch ref.
+        let patch_url = self.api_url(&format!("git/refs/heads/{}", branch));
+        let patch_body = serde_json::json!({ "sha": new_commit.sha });
+        self.patch::<serde_json::Value>(&patch_url, patch_body).await?;
+
+        Ok(CreatedCommit {
+            sha: new_commit.sha,
+            url: new_commit.html_url,
+        })
+    }
+
+    async fn create_pull_request(
+        &self,
+        params: &PullRequestParams,
+    ) -> NovaResult<CreatedPullRequest> {
+        let url = self.api_url("pulls");
+        let body = serde_json::json!({
+            "title": params.title,
+            "body":  params.body,
+            "head":  params.head,
+            "base":  params.base
+        });
+
+        let pr: GitHubPullRequest = self.post(&url, body).await?;
+
+        Ok(CreatedPullRequest {
+            id: pr.id.to_string(),
+            url: pr.html_url,
+            number: pr.number,
+        })
+    }
+
     fn into_tools(self: Box<Self>) -> Vec<Box<dyn Tool>> {
         let integration = Arc::new(*self);
 
@@ -281,6 +428,52 @@ struct GitHubCommentResponse {
     html_url: String,
 }
 
+// Git database / source control response types
+
+#[derive(Debug, Deserialize)]
+struct GitHubRef {
+    object: GitHubRefObject,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitHubRefObject {
+    sha: String,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitHubCommitObject {
+    tree: GitHubTreeRef,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitHubTreeRef {
+    sha: String,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitHubBlob {
+    sha: String,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitHubTree {
+    sha: String,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitHubCommitCreated {
+    sha: String,
+    #[serde(default)]
+    html_url: String,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitHubPullRequest {
+    id: u64,
+    number: u64,
+    html_url: String,
+}
+
 // GitHub tools
 
 struct GetGitHubIssueTool {
diff --git a/crates/cclab-agent/src/integrations/gitlab.rs b/crates/cclab-agent/src/integrations/gitlab.rs
index ac0fa901..c67b24c3 100644
--- a/crates/cclab-agent/src/integrations/gitlab.rs
+++ b/crates/cclab-agent/src/integrations/gitlab.rs
@@ -1,7 +1,8 @@
 //! GitLab integration for issue tracking.
 
 use super::{
-    Issue, IssueComment, IssueFilter, IssueSummary, IssueState, PlatformIntegration, PostedComment,
+    CommitFile, CreatedBranch, CreatedCommit, CreatedPullRequest, Issue, IssueComment, IssueFilter,
+    IssueSummary, IssueState, PlatformIntegration, PostedComment, PullRequestParams,
 };
 use crate::error::{NovaError, NovaResult};
 use crate::tools::{Tool, ToolParameter};
@@ -243,6 +244,84 @@ impl PlatformIntegration for GitLabIntegration {
         })
     }
 
+    async fn create_branch(
+        &self,
+        branch_name: &str,
+        from_ref: &str,
+    ) -> NovaResult<CreatedBranch> {
+        let url = self.api_url("repository/branches");
+        let body = serde_json::json!({
+            "branch": branch_name,
+            "ref": from_ref
+        });
+
+        let branch: GitLabBranch = self.post(&url, body).await?;
+
+        Ok(CreatedBranch {
+            name: branch.name,
+            sha: branch.commit.id,
+        })
+    }
+
+    async fn create_commit(
+        &self,
+        branch: &str,
+        message: &str,
+        files: &[CommitFile],
+    ) -> NovaResult<CreatedCommit> {
+        let url = self.api_url("repository/commits");
+
+        let actions: Vec<serde_json::Value> = files
+            .iter()
+            .map(|f| {
+                serde_json::json!({
+                    "action": "create",
+                    "file_path": f.path,
+                    "content": f.content
+                })
+            })
+            .collect();
+
+        let body = serde_json::json!({
+            "branch": branch,
+            "commit_message": message,
+            "actions": actions
+        });
+
+        let commit: GitLabCommitResponse = self.post(&url, body).await?;
+
+        let commit_url = format!(
+            "{}/{}/-/commit/{}",
+            self.base_url, self.project_id, commit.id
+        );
+
+        Ok(CreatedCommit {
+            sha: commit.id,
+            url: commit_url,
+        })
+    }
+
+    async fn create_pull_request(
+        &self,
+        params: &PullRequestParams,
+    ) -> NovaResult<CreatedPullRequest> {
+        let url = self.api_url("merge_requests");
+        let body = serde_json::json!({
+            "title":         params.title,
+            "description":   params.body,
+            "source_branch": params.head,
+            "target_branch": params.base
+        });
+
+        let mr: GitLabMergeRequest = self.post(&url, body).await?;
+
+        Ok(CreatedPullRequest {
+            id: mr.id.to_string(),
+            url: mr.web_url,
+            number: mr.iid,
+        })
+    }
+
     fn into_tools(self: Box<Self>) -> Vec<Box<dyn Tool>> {
         let integration = Arc::new(*self);
 
@@ -299,6 +378,31 @@ struct GitLabNoteResponse {
     id: u64,
 }
 
+// Source control response types
+
+#[derive(Debug, Deserialize)]
+struct GitLabBranchCommit {
+    id: String,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitLabBranch {
+    name: String,
+    commit: GitLabBranchCommit,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitLabCommitResponse {
+    id: String,
+}
+
+#[derive(Debug, Deserialize)]
+struct GitLabMergeRequest {
+    id: u64,
+    iid: u64,
+    web_url: String,
+}
+
 // GitLab tools
 
 struct GetGitLabIssueTool {
diff --git a/crates/cclab-agent/src/integrations/mod.rs b/crates/cclab-agent/src/integrations/mod.rs
index 83e85ca3..608951c9 100644
--- a/crates/cclab-agent/src/integrations/mod.rs
+++ b/crates/cclab-agent/src/integrations/mod.rs
@@ -11,7 +11,7 @@ pub use github::GitHubIntegration;
 pub use gitlab::GitLabIntegration;
 pub use jira::JiraIntegration;
 
-use crate::error::NovaResult;
+use crate::error::{NovaError, NovaResult};
 use crate::tools::Tool;
 use async_trait::async_trait;
 use chrono::{DateTime, Utc};
@@ -167,6 +167,61 @@ pub struct PostedComment {
     pub url: String,
 }
 
+// ============================================================
+// Source control types
+// ============================================================
+
+/// A file to include in a commit.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct CommitFile {
+    /// Repository-relative path (e.g. `src/lib.rs`).
+    pub path: String,
+    /// Full file content (UTF-8).
+    pub content: String,
+}
+
+/// Result of creating a branch.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct CreatedBranch {
+    /// Branch name.
+    pub name: String,
+    /// Commit SHA the branch points at.
+    pub sha: String,
+}
+
+/// Result of creating a commit.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct CreatedCommit {
+    /// Commit SHA.
+    pub sha: String,
+    /// URL to the commit on the platform.
+    pub url: String,
+}
+
+/// Parameters for opening a pull / merge request.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct PullRequestParams {
+    /// PR/MR title.
+    pub title: String,
+    /// PR/MR body / description.
+    pub body: String,
+    /// Source branch (head).
+    pub head: String,
+    /// Target branch (base).
+    pub base: String,
+}
+
+/// Result of creating a pull / merge request.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct CreatedPullRequest {
+    /// Platform-assigned ID string.
+    pub id: String,
+    /// URL to view the PR/MR.
+    pub url: String,
+    /// PR/MR number on the platform.
+    pub number: u64,
+}
+
 /// Parsed response from a clarification comment.
 #[derive(Debug, Clone, Default, Serialize, Deserialize)]
 pub struct ParsedClarificationResponse {
@@ -273,8 +328,8 @@ pub fn parse_clarification_response(comment_body: &str) -> ParsedClarificationRe
 
 /// Trait for platform integrations.
 ///
-/// Platform integrations provide access to external issue tracking
-/// and project management systems.
+/// Platform integrations provide access to external issue tracking,
+/// project management, and source control systems.
 #[async_trait]
 pub trait PlatformIntegration: Send + Sync {
     /// Get the platform name.
@@ -292,6 +347,51 @@ pub trait PlatformIntegration: Send + Sync {
     /// Post a comment to an issue.
     async fn post_comment(&self, issue_id: &str, body: &str) -> NovaResult<PostedComment>;
 
+    // ---- Source control ----
+
+    /// Create a new branch from `from_ref` (branch name or commit SHA).
+    ///
+    /// Default implementation returns [`NovaError::PlatformError`].
+    /// Override in platforms that support source control.
+    async fn create_branch(
+        &self,
+        _branch_name: &str,
+        _from_ref: &str,
+    ) -> NovaResult<CreatedBranch> {
+        Err(NovaError::PlatformError(format!(
+            "{} does not support source control",
+            self.name()
+        )))
+    }
+
+    /// Create a single commit on `branch` containing all `files`.
+    ///
+    /// Default implementation returns [`NovaError::PlatformError`].
+    async fn create_commit(
+        &self,
+        _branch: &str,
+        _message: &str,
+        _files: &[CommitFile],
+    ) -> NovaResult<CreatedCommit> {
+        Err(NovaError::PlatformError(format!(
+            "{} does not support source control",
+            self.name()
+        )))
+    }
+
+    /// Open a pull request (GitHub) or merge request (GitLab).
+    ///
+    /// Default implementation returns [`NovaError::PlatformError`].
+    async fn create_pull_request(
+        &self,
+        _params: &PullRequestParams,
+    ) -> NovaResult<CreatedPullRequest> {
+        Err(NovaError::PlatformError(format!(
+            "{} does not support pull requests",
+            self.name()
+        )))
+    }
+
     /// Convert this integration into a set of tools.
     fn into_tools(self: Box<Self>) -> Vec<Box<dyn Tool>>;
 }
diff --git a/crates/cclab-agent/src/lib.rs b/crates/cclab-agent/src/lib.rs
index bd6715b4..a99448f4 100644
--- a/crates/cclab-agent/src/lib.rs
+++ b/crates/cclab-agent/src/lib.rs
@@ -82,6 +82,8 @@ pub use agents::{
     Agent, AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig, ApprovalHandler,
     AutoApproveHandler,
     ChangeSpecAgent, ChangeSpecAgentBuilder, ChangeSpecAgentConfig, ChangeSpecInput,
+    CodeAgent, CodeAgentBuilder, CodeAgentConfig, FileBlock, ImplementationTask, TaskAction,
+    TaskCategory,
     CodingAgent, CodingAgentBuilder, CodingAgentConfig,
     CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType,
     Clarification, Contradiction, Question,
@@ -133,6 +135,7 @@ pub use storage::{
 
 // Re-export integration types
 pub use integrations::{
-    GitHubIntegration, GitLabIntegration, Issue, IssueComment, IssueFilter, IssueState,
-    IssueSummary, JiraIntegration, PlatformIntegration,
+    CommitFile, CreatedBranch, CreatedCommit, CreatedPullRequest, GitHubIntegration,
+    GitLabIntegration, Issue, IssueComment, IssueFilter, IssueState, IssueSummary, JiraIntegration,
+    PlatformIntegration, PullRequestParams,
 };
diff --git a/crates/cclab-agent/src/agents/code_agent/mod.rs b/crates/cclab-agent/src/agents/code_agent/mod.rs
new file mode 100644
index 00000000..95eef9af
--- /dev/null
+++ b/crates/cclab-agent/src/agents/code_agent/mod.rs
@@ -0,0 +1,638 @@
+//! CodeAgent — transforms approved specifications into code implementations.
+//!
+//! # Flow
+//!
+//! ```text
+//! execute(spec)
+//!   │
+//!   ├─ decompose_spec → ordered ImplementationTask list
+//!   │
+//!   ├─ CRRCycle(creator=LLM, reviewer=ReviewAgent, reviser=LLM)
+//!   │     └─ artifact = multi-file XML blob
+//!   │
+//!   ├─ parse_file_blocks → Vec<FileBlock>
+//!   │
+//!   ├─ PlatformIntegration::create_branch
+//!   ├─ PlatformIntegration::create_commit
+//!   └─ PlatformIntegration::create_pull_request → PR URL
+//! ```
+
+mod parser;
+mod tasks;
+
+pub use parser::{parse_file_blocks, FileBlock};
+pub use tasks::{decompose_spec, ImplementationTask, TaskAction, TaskCategory};
+
+use crate::agents::crr::CRRCycle;
+use crate::agents::review::{ReviewAgent, ReviewAgentConfig, ReviewType, Reviewer};
+use crate::agents::Agent;
+use crate::error::{NovaError, NovaResult};
+use crate::integrations::{CommitFile, PlatformIntegration, PullRequestParams};
+use crate::llm::{CompletionRequest, LLMProvider};
+use crate::stream::{NoOpHandler, StreamHandler};
+use crate::types::Message;
+use async_trait::async_trait;
+use std::sync::Arc;
+
+// ============================================================
+// Config
+// ============================================================
+
+/// Configuration for [`CodeAgent`].
+#[derive(Debug, Clone)]
+pub struct CodeAgentConfig {
+    /// LLM model used for code generation and revision.
+    pub model: String,
+    /// Maximum tokens per LLM call.
+    pub max_tokens: Option<u32>,
+    /// Sampling temperature (0.0 = deterministic).
+    pub temperature: Option<f32>,
+    /// Maximum CRR revision rounds before [`NovaError::MaxRevisionsExceeded`].
+    pub max_revisions: u32,
+    /// Base branch to branch from and open PRs against.
+    pub base_branch: String,
+    /// Prefix for auto-generated branch names.
+    pub branch_prefix: String,
+}
+
+impl Default for CodeAgentConfig {
+    fn default() -> Self {
+        Self {
+            model: "claude-sonnet-4-20250514".to_string(),
+            max_tokens: Some(8192),
+            temperature: Some(0.0),
+            max_revisions: 3,
+            base_branch: "main".to_string(),
+            branch_prefix: "feature/code-agent-".to_string(),
+        }
+    }
+}
+
+// ============================================================
+// CodeAgent
+// ============================================================
+
+/// Autonomous agent that implements approved specifications end-to-end.
+///
+/// Orchestrates: task decomposition → LLM code generation → CRR review →
+/// platform branch/commit/PR creation.
+pub struct CodeAgent {
+    config: CodeAgentConfig,
+    provider: Arc<dyn LLMProvider>,
+    reviewer: Arc<dyn Reviewer>,
+    platform: Arc<dyn PlatformIntegration>,
+}
+
+impl std::fmt::Debug for CodeAgent {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("CodeAgent")
+            .field("config", &self.config)
+            .finish_non_exhaustive()
+    }
+}
+
+impl CodeAgent {
+    /// Return a builder for this agent.
+    pub fn builder() -> CodeAgentBuilder {
+        CodeAgentBuilder::new()
+    }
+}
+
+#[async_trait]
+impl Agent for CodeAgent {
+    /// Run with `input` as the full specification text.
+    ///
+    /// Returns the URL of the opened pull / merge request on success.
+    async fn run(&self, input: &str) -> NovaResult<String> {
+        let handler = NoOpHandler;
+        self.run_with_handler(input, &handler).await
+    }
+
+    async fn run_with_handler(
+        &self,
+        input: &str,
+        _handler: &dyn StreamHandler,
+    ) -> NovaResult<String> {
+        // 1. Decompose spec into topologically sorted tasks.
+        let tasks = decompose_spec(input);
+
+        // 2. Build the generation prompt from the tasks and spec.
+        let generation_prompt = build_generation_prompt(input, &tasks);
+
+        // 3. Wire up CRR cycle.
+        let creator = Arc::new(InlineLLMAgent {
+            provider: self.provider.clone(),
+            system_prompt: CODE_GENERATION_SYSTEM_PROMPT.to_string(),
+            model: self.config.model.clone(),
+            max_tokens: self.config.max_tokens,
+            temperature: self.config.temperature,
+        });
+
+        let reviser = Arc::new(InlineLLMAgent {
+            provider: self.provider.clone(),
+            system_prompt: CODE_REVISION_SYSTEM_PROMPT.to_string(),
+            model: self.config.model.clone(),
+            max_tokens: self.config.max_tokens,
+            temperature: self.config.temperature,
+        });
+
+        let crr = CRRCycle::new()
+            .creator_arc(creator)
+            .reviewer_arc(self.reviewer.clone())
+            .reviser_arc(reviser)
+            .max_revisions(self.config.max_revisions)
+            .build()?;
+
+        // 4. Run CRR — get approved code artifact (multi-file XML blob).
+        let crr_result = crr.run(&generation_prompt).await?;
+
+        // 5. Parse XML file blocks.
+        let file_blocks = parse_file_blocks(&crr_result.artifact)?;
+
+        // 6. Create a remote branch.
+        let branch_name = format!(
+            "{}{}",
+            self.config.branch_prefix,
+            chrono::Utc::now().timestamp()
+        );
+        self.platform
+            .create_branch(&branch_name, &self.config.base_branch)
+            .await
+            .map_err(|e| NovaError::PlatformError(format!("create_branch failed: {}", e)))?;
+
+        // 7. Commit all generated files in a single commit.
+        let commit_files: Vec<CommitFile> = file_blocks
+            .into_iter()
+            .map(|fb| CommitFile {
+                path: fb.path,
+                content: fb.content,
+            })
+            .collect();
+
+        self.platform
+            .create_commit(
+                &branch_name,
+                "feat: implement spec via CodeAgent",
+                &commit_files,
+            )
+            .await
+            .map_err(|e| NovaError::PlatformError(format!("create_commit failed: {}", e)))?;
+
+        // 8. Open a pull / merge request.
+        let pr = self
+            .platform
+            .create_pull_request(&PullRequestParams {
+                title: "feat: implement spec via CodeAgent".to_string(),
+                body: format!(
+                    "Automated implementation generated by CodeAgent.\n\n\
+                     CRR revisions: {}",
+                    crr_result.revision_count
+                ),
+                head: branch_name,
+                base: self.config.base_branch.clone(),
+            })
+            .await
+            .map_err(|e| NovaError::PlatformError(format!("create_pull_request failed: {}", e)))?;
+
+        Ok(pr.url)
+    }
+}
+
+// ============================================================
+// Prompt construction
+// ============================================================
+
+fn build_generation_prompt(spec: &str, tasks: &[ImplementationTask]) -> String {
+    let mut prompt = String::from(
+        "Implement the following specification. \
+         Output ALL files using the XML format below — one block per file:\n\n\
+         <file path=\"relative/path/to/file.ext\">\n\
+         // full file content here\n\
+         </file>\n\n\
+         Do NOT include any prose outside the <file> blocks.\n\n\
+         ## Specification\n\n",
+    );
+    prompt.push_str(spec);
+
+    if !tasks.is_empty() {
+        prompt.push_str("\n\n## Implementation Order\n\n");
+        for task in tasks {
+            prompt.push_str(&format!(
+                "- [{:?}] {} → `{}`\n",
+                task.action, task.description, task.file_path
+            ));
+        }
+    }
+
+    prompt
+}
+
+const CODE_GENERATION_SYSTEM_PROMPT: &str = r#"You are an expert Rust/software engineer.
+Your task: implement a specification exactly as written.
+
+Rules:
+1. Output ONLY <file path="...">...</file> blocks — no commentary outside them.
+2. Implement every file listed in the ## Changes section.
+3. Follow existing code style and module conventions.
+4. Include #[cfg(test)] unit tests for all public functions.
+5. Handle errors with the existing NovaError type."#;
+
+const CODE_REVISION_SYSTEM_PROMPT: &str = r#"You are an expert Rust/software engineer performing a revision.
+You will receive a previous implementation and a list of review issues.
+
+Rules:
+1. Address every review issue listed.
+2. Output the FULL revised files as <file path="...">...</file> blocks.
+3. Do not omit unchanged files — include all files from the original output.
+4. Do not add commentary outside the <file> blocks."#;
+
+// ============================================================
+// InlineLLMAgent — minimal Agent wrapper around a single LLM call
+// ============================================================
+
+/// A minimal agent that forwards the input directly to the LLM.
+struct InlineLLMAgent {
+    provider: Arc<dyn LLMProvider>,
+    system_prompt: String,
+    model: String,
+    max_tokens: Option<u32>,
+    temperature: Option<f32>,
+}
+
+#[async_trait]
+impl Agent for InlineLLMAgent {
+    async fn run(&self, input: &str) -> NovaResult<String> {
+        let handler = NoOpHandler;
+        self.run_with_handler(input, &handler).await
+    }
+
+    async fn run_with_handler(
+        &self,
+        input: &str,
+        _handler: &dyn StreamHandler,
+    ) -> NovaResult<String> {
+        let messages = vec![
+            Message::system(&self.system_prompt),
+            Message::user(input),
+        ];
+
+        let mut request = CompletionRequest::new(messages, &self.model);
+        if let Some(temp) = self.temperature {
+            request = request.with_temperature(temp);
+        }
+        if let Some(max_tokens) = self.max_tokens {
+            request = request.with_max_tokens(max_tokens);
+        }
+
+        let response = self.provider.complete(request).await?;
+        Ok(response.content)
+    }
+}
+
+// ============================================================
+// Builder
+// ============================================================
+
+/// Builder for [`CodeAgent`].
+pub struct CodeAgentBuilder {
+    config: CodeAgentConfig,
+    provider: Option<Arc<dyn LLMProvider>>,
+    reviewer: Option<Arc<dyn Reviewer>>,
+    platform: Option<Arc<dyn PlatformIntegration>>,
+}
+
+impl CodeAgentBuilder {
+    pub fn new() -> Self {
+        Self {
+            config: CodeAgentConfig::default(),
+            provider: None,
+            reviewer: None,
+            platform: None,
+        }
+    }
+
+    /// Set the LLM provider used for code generation and revision.
+    pub fn with_provider<P: LLMProvider + 'static>(mut self, provider: P) -> Self {
+        self.provider = Some(Arc::new(provider));
+        self
+    }
+
+    /// Set the LLM provider from an `Arc`.
+    pub fn with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self {
+        self.provider = Some(provider);
+        self
+    }
+
+    /// Set a custom reviewer.  Defaults to a [`ReviewAgent`] using `Code` mode.
+    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
+        self.reviewer = Some(Arc::new(reviewer));
+        self
+    }
+
+    /// Set the reviewer from an `Arc`.
+    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
+        self.reviewer = Some(reviewer);
+        self
+    }
+
+    /// Set the platform integration for source control operations.
+    pub fn with_platform<P: PlatformIntegration + 'static>(mut self, platform: P) -> Self {
+        self.platform = Some(Arc::new(platform));
+        self
+    }
+
+    /// Set the platform integration from an `Arc`.
+    pub fn with_platform_arc(mut self, platform: Arc<dyn PlatformIntegration>) -> Self {
+        self.platform = Some(platform);
+        self
+    }
+
+    pub fn with_model(mut self, model: impl Into<String>) -> Self {
+        self.config.model = model.into();
+        self
+    }
+
+    pub fn with_max_revisions(mut self, n: u32) -> Self {
+        self.config.max_revisions = n;
+        self
+    }
+
+    pub fn with_base_branch(mut self, branch: impl Into<String>) -> Self {
+        self.config.base_branch = branch.into();
+        self
+    }
+
+    pub fn with_branch_prefix(mut self, prefix: impl Into<String>) -> Self {
+        self.config.branch_prefix = prefix.into();
+        self
+    }
+
+    /// Build the [`CodeAgent`].
+    ///
+    /// # Errors
+    ///
+    /// Returns [`NovaError::ConfigError`] if `provider` or `platform` is missing.
+    pub fn build(self) -> NovaResult<CodeAgent> {
+        let provider = self
+            .provider
+            .ok_or_else(|| NovaError::ConfigError("CodeAgent: provider is required".to_string()))?;
+
+        let platform = self
+            .platform
+            .ok_or_else(|| NovaError::ConfigError("CodeAgent: platform is required".to_string()))?;
+
+        // Default reviewer: ReviewAgent with Code review type using the same provider.
+        let reviewer: Arc<dyn Reviewer> = match self.reviewer {
+            Some(r) => r,
+            None => {
+                let review_agent = ReviewAgent::builder()
+                    .with_provider_arc(provider.clone())
+                    .with_review_type(ReviewType::Code)
+                    .with_model(self.config.model.clone())
+                    .build()?;
+                Arc::new(review_agent)
+            }
+        };
+
+        Ok(CodeAgent {
+            config: self.config,
+            provider,
+            reviewer,
+            platform,
+        })
+    }
+}
+
+impl Default for CodeAgentBuilder {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+// ============================================================
+// Tests
+// ============================================================
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::agents::review::{ReviewIssue, ReviewVerdict, Reviewer, Severity};
+    use crate::integrations::{
+        CommitFile, CreatedBranch, CreatedCommit, CreatedPullRequest, Issue, IssueComment,
+        IssueFilter, IssueSummary, PostedComment, PullRequestParams,
+    };
+    use crate::llm::{CompletionRequest, CompletionResponse, StreamResponse};
+    use crate::types::TokenUsage;
+    use std::collections::HashMap;
+
+    // ---- mock LLM provider ----
+
+    struct MockProvider {
+        response: String,
+    }
+
+    #[async_trait::async_trait]
+    impl LLMProvider for MockProvider {
+        fn provider_name(&self) -> &str {
+            "mock"
+        }
+        fn supported_models(&self) -> Vec<String> {
+            vec!["mock".to_string()]
+        }
+        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
+            Ok(CompletionResponse {
+                content: self.response.clone(),
+                tool_calls: None,
+                finish_reason: "stop".to_string(),
+                usage: TokenUsage::default(),
+                model: "mock".to_string(),
+                metadata: HashMap::new(),
+            })
+        }
+        async fn complete_stream(&self, _req: CompletionRequest) -> NovaResult<StreamResponse> {
+            unimplemented!()
+        }
+    }
+
+    // ---- mock reviewer that always approves ----
+
+    struct ApproveReviewer;
+
+    #[async_trait::async_trait]
+    impl Reviewer for ApproveReviewer {
+        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
+            Ok(ReviewVerdict::Approved)
+        }
+    }
+
+    // ---- mock platform ----
+
+    struct MockPlatform;
+
+    #[async_trait::async_trait]
+    impl crate::integrations::PlatformIntegration for MockPlatform {
+        fn name(&self) -> &str {
+            "mock"
+        }
+        async fn get_issue(&self, _id: &str) -> NovaResult<Issue> {
+            unimplemented!()
+        }
+        async fn list_issues(&self, _filter: &IssueFilter) -> NovaResult<Vec<IssueSummary>> {
+            unimplemented!()
+        }
+        async fn get_comments(&self, _issue_id: &str) -> NovaResult<Vec<IssueComment>> {
+            unimplemented!()
+        }
+        async fn post_comment(
+            &self,
+            _issue_id: &str,
+            _body: &str,
+        ) -> NovaResult<PostedComment> {
+            unimplemented!()
+        }
+        async fn create_branch(
+            &self,
+            branch_name: &str,
+            _from_ref: &str,
+        ) -> NovaResult<CreatedBranch> {
+            Ok(CreatedBranch {
+                name: branch_name.to_string(),
+                sha: "abc123".to_string(),
+            })
+        }
+        async fn create_commit(
+            &self,
+            _branch: &str,
+            _message: &str,
+            _files: &[CommitFile],
+        ) -> NovaResult<CreatedCommit> {
+            Ok(CreatedCommit {
+                sha: "def456".to_string(),
+                url: "https://example.com/commit/def456".to_string(),
+            })
+        }
+        async fn create_pull_request(
+            &self,
+            params: &PullRequestParams,
+        ) -> NovaResult<CreatedPullRequest> {
+            Ok(CreatedPullRequest {
+                id: "1".to_string(),
+                url: format!("https://example.com/pr/1?branch={}", params.head),
+                number: 1,
+            })
+        }
+        fn into_tools(self: Box<Self>) -> Vec<Box<dyn crate::tools::Tool>> {
+            vec![]
+        }
+    }
+
+    // ---- tests ----
+
+    #[tokio::test]
+    async fn test_code_agent_success() {
+        let xml = "<file path=\"src/lib.rs\">\npub fn hello() {}\n</file>";
+
+        let agent = CodeAgent::builder()
+            .with_provider(MockProvider {
+                response: xml.to_string(),
+            })
+            .with_reviewer(ApproveReviewer)
+            .with_platform(MockPlatform)
+            .build()
+            .unwrap();
+
+        let result = agent
+            .run("## Changes\n- `src/lib.rs`:\n  - **CREATE**: Hello function.\n")
+            .await
+            .unwrap();
+
+        assert!(result.contains("https://example.com/pr/1"));
+    }
+
+    #[tokio::test]
+    async fn test_code_agent_malformed_xml_error() {
+        let agent = CodeAgent::builder()
+            .with_provider(MockProvider {
+                response: "no xml blocks here".to_string(),
+            })
+            .with_reviewer(ApproveReviewer)
+            .with_platform(MockPlatform)
+            .build()
+            .unwrap();
+
+        let err = agent.run("## Changes\n- `src/lib.rs`:\n  - **CREATE**: x\n").await.unwrap_err();
+        assert!(
+            matches!(err, NovaError::MalformedLLMResponse(_)),
+            "expected MalformedLLMResponse, got {:?}",
+            err
+        );
+    }
+
+    #[tokio::test]
+    async fn test_code_agent_max_revisions_exceeded() {
+        use crate::agents::review::ReviewIssue;
+
+        struct AlwaysRejectReviewer;
+
+        #[async_trait::async_trait]
+        impl Reviewer for AlwaysRejectReviewer {
+            async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
+                Ok(ReviewVerdict::NeedsRevision {
+                    issues: vec![ReviewIssue {
+                        severity: Severity::High,
+                        description: "always broken".to_string(),
+                        suggestion: "fix it".to_string(),
+                        location: None,
+                    }],
+                })
+            }
+        }
+
+        let xml = "<file path=\"src/lib.rs\">\npub fn hello() {}\n</file>";
+
+        let agent = CodeAgent::builder()
+            .with_provider(MockProvider {
+                response: xml.to_string(),
+            })
+            .with_reviewer(AlwaysRejectReviewer)
+            .with_platform(MockPlatform)
+            .with_max_revisions(2)
+            .build()
+            .unwrap();
+
+        let err = agent.run("## Changes\n- `src/lib.rs`:\n  - **CREATE**: x\n").await.unwrap_err();
+        assert!(
+            matches!(err, NovaError::MaxRevisionsExceeded(2)),
+            "expected MaxRevisionsExceeded(2), got {:?}",
+            err
+        );
+    }
+
+    #[test]
+    fn test_builder_missing_provider() {
+        let err = CodeAgent::builder()
+            .with_platform(MockPlatform)
+            .build()
+            .unwrap_err();
+        assert!(matches!(err, NovaError::ConfigError(_)));
+    }
+
+    #[test]
+    fn test_builder_missing_platform() {
+        let err = CodeAgent::builder()
+            .with_provider(MockProvider {
+                response: String::new(),
+            })
+            .build()
+            .unwrap_err();
+        assert!(matches!(err, NovaError::ConfigError(_)));
+    }
+
+    #[test]
+    fn test_default_config() {
+        let config = CodeAgentConfig::default();
+        assert_eq!(config.max_revisions, 3);
+        assert_eq!(config.base_branch, "main");
+        assert!(!config.branch_prefix.is_empty());
+    }
+}
diff --git a/crates/cclab-agent/src/agents/code_agent/parser.rs b/crates/cclab-agent/src/agents/code_agent/parser.rs
new file mode 100644
index 00000000..c33acb98
--- /dev/null
+++ b/crates/cclab-agent/src/agents/code_agent/parser.rs
@@ -0,0 +1,191 @@
+//! XML block parser for LLM code generation responses.
+//!
+//! The LLM is instructed to wrap generated files in:
+//!
+//! ```xml
+//! <file path="src/lib.rs">
+//! // file contents here
+//! </file>
+//! ```
+//!
+//! This module extracts those blocks reliably without a full XML parser.
+
+use crate::error::{NovaError, NovaResult};
+
+/// A parsed file block extracted from an LLM response.
+#[derive(Debug, Clone, PartialEq)]
+pub struct FileBlock {
+    /// Repository-relative path declared in the `path` attribute.
+    pub path: String,
+    /// Raw file content between the tags (leading/trailing newlines stripped).
+    pub content: String,
+}
+
+/// Parse all `<file path="...">...</file>` blocks from `input`.
+///
+/// # Errors
+///
+/// Returns [`NovaError::MalformedLLMResponse`] when:
+/// - An opening `<file` tag is missing the `path` attribute.
+/// - An opening tag is never closed with `>`.
+/// - A `</file>` closing tag is missing for an opened block.
+/// - No `<file>` blocks are found at all.
+pub fn parse_file_blocks(input: &str) -> NovaResult<Vec<FileBlock>> {
+    let mut blocks = Vec::new();
+    let mut remaining = input;
+
+    loop {
+        match remaining.find("<file ") {
+            None => break,
+            Some(tag_start) => {
+                let after_open = &remaining[tag_start..];
+
+                // Find closing `>` of the opening tag.
+                let close_bracket = after_open.find('>').ok_or_else(|| {
+                    NovaError::MalformedLLMResponse(
+                        "Unclosed <file> opening tag — missing '>'".to_string(),
+                    )
+                })?;
+
+                let tag_content = &after_open[..close_bracket];
+                let path = extract_path_attr(tag_content)?;
+
+                // Content starts immediately after `>`.
+                let content_area = &after_open[close_bracket + 1..];
+
+                let end_tag = "</file>";
+                let end_pos = content_area.find(end_tag).ok_or_else(|| {
+                    NovaError::MalformedLLMResponse(format!(
+                        "Missing </file> closing tag for path: {}",
+                        path
+                    ))
+                })?;
+
+                let content = content_area[..end_pos]
+                    .trim_start_matches('\n')
+                    .trim_end_matches('\n')
+                    .to_string();
+
+                blocks.push(FileBlock { path, content });
+
+                // Advance past this entire block.
+                let consumed = tag_start + (close_bracket + 1) + end_pos + end_tag.len();
+                remaining = &remaining[consumed..];
+            }
+        }
+    }
+
+    if blocks.is_empty() {
+        return Err(NovaError::MalformedLLMResponse(
+            "No <file path=\"...\">...</file> blocks found in LLM response".to_string(),
+        ));
+    }
+
+    Ok(blocks)
+}
+
+// ---- helpers ----
+
+fn extract_path_attr(tag: &str) -> NovaResult<String> {
+    // Support both double and single quotes: path="..." or path='...'
+    for &quote in &['"', '\''] {
+        let attr = format!("path={}", quote);
+        if let Some(start) = tag.find(&attr) {
+            let after = &tag[start + attr.len()..];
+            if let Some(end) = after.find(quote) {
+                let path = after[..end].to_string();
+                if path.is_empty() {
+                    return Err(NovaError::MalformedLLMResponse(
+                        "Empty path attribute in <file> tag".to_string(),
+                    ));
+                }
+                return Ok(path);
+            }
+        }
+    }
+    Err(NovaError::MalformedLLMResponse(
+        "Missing or malformed 'path' attribute in <file> tag".to_string(),
+    ))
+}
+
+// ============================================================
+// Tests
+// ============================================================
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_parse_single_file_block() {
+        let input = r#"<file path="src/lib.rs">
+fn hello() {}
+</file>"#;
+        let blocks = parse_file_blocks(input).unwrap();
+        assert_eq!(blocks.len(), 1);
+        assert_eq!(blocks[0].path, "src/lib.rs");
+        assert_eq!(blocks[0].content, "fn hello() {}");
+    }
+
+    #[test]
+    fn test_parse_multiple_file_blocks() {
+        let input = r#"Some preamble text.
+<file path="src/main.rs">
+fn main() {}
+</file>
+Some middle text.
+<file path="src/lib.rs">
+pub fn helper() {}
+</file>
+Trailing text."#;
+
+        let blocks = parse_file_blocks(input).unwrap();
+        assert_eq!(blocks.len(), 2);
+        assert_eq!(blocks[0].path, "src/main.rs");
+        assert_eq!(blocks[0].content, "fn main() {}");
+        assert_eq!(blocks[1].path, "src/lib.rs");
+        assert_eq!(blocks[1].content, "pub fn helper() {}");
+    }
+
+    #[test]
+    fn test_parse_single_quotes() {
+        let input = "<file path='src/lib.rs'>\nfn x() {}\n</file>";
+        let blocks = parse_file_blocks(input).unwrap();
+        assert_eq!(blocks[0].path, "src/lib.rs");
+    }
+
+    #[test]
+    fn test_error_no_blocks() {
+        let err = parse_file_blocks("no blocks here").unwrap_err();
+        assert!(matches!(err, NovaError::MalformedLLMResponse(_)));
+        assert!(err.to_string().contains("No <file"));
+    }
+
+    #[test]
+    fn test_error_missing_path_attr() {
+        let err = parse_file_blocks("<file >content</file>").unwrap_err();
+        assert!(matches!(err, NovaError::MalformedLLMResponse(_)));
+    }
+
+    #[test]
+    fn test_error_missing_close_tag() {
+        let err = parse_file_blocks("<file path=\"x.rs\">content without close").unwrap_err();
+        assert!(matches!(err, NovaError::MalformedLLMResponse(_)));
+        assert!(err.to_string().contains("</file>"));
+    }
+
+    #[test]
+    fn test_error_unclosed_open_tag() {
+        let err = parse_file_blocks("<file path=\"x.rs\" content</file>").unwrap_err();
+        assert!(matches!(err, NovaError::MalformedLLMResponse(_)));
+    }
+
+    #[test]
+    fn test_multiline_content_preserved() {
+        let input = "<file path=\"config.toml\">\n[package]\nname = \"foo\"\nversion = \"0.1.0\"\n</file>";
+        let blocks = parse_file_blocks(input).unwrap();
+        let content = &blocks[0].content;
+        assert!(content.contains("[package]"));
+        assert!(content.contains("name = \"foo\""));
+    }
+}
diff --git a/crates/cclab-agent/src/agents/code_agent/tasks.rs b/crates/cclab-agent/src/agents/code_agent/tasks.rs
new file mode 100644
index 00000000..6bd77bfc
--- /dev/null
+++ b/crates/cclab-agent/src/agents/code_agent/tasks.rs
@@ -0,0 +1,344 @@
+//! Task decomposition and topological sorting for CodeAgent.
+//!
+//! Parses the `## Changes` section of a specification and returns
+//! implementation tasks ordered Data → Logic → Integration → Test.
+
+use serde::{Deserialize, Serialize};
+
+// ============================================================
+// Types
+// ============================================================
+
+/// Execution category that controls the topological sort order.
+///
+/// `Ord` is derived so tasks sort Data < Logic < Integration < Test.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
+pub enum TaskCategory {
+    DataModel = 0,
+    Logic = 1,
+    Integration = 2,
+    Test = 3,
+}
+
+/// Whether the target file should be newly created or modified in place.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+pub enum TaskAction {
+    Create,
+    Modify,
+}
+
+/// A single implementation task extracted from a spec.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ImplementationTask {
+    /// Sequential identifier (1-based).
+    pub id: String,
+    /// Short description extracted from the spec bullet.
+    pub description: String,
+    /// Relative file path targeted by this task.
+    pub file_path: String,
+    /// Execution category (determines sort position).
+    pub category: TaskCategory,
+    /// Create or modify.
+    pub action: TaskAction,
+}
+
+// ============================================================
+// Public API
+// ============================================================
+
+/// Parse the `## Changes` section of `spec` into an ordered task list.
+///
+/// Tasks are sorted by category: Data → Logic → Integration → Test.
+/// Within the same category the original spec order is preserved (stable sort).
+pub fn decompose_spec(spec: &str) -> Vec<ImplementationTask> {
+    let changes = extract_changes_section(spec);
+    let mut tasks: Vec<ImplementationTask> = parse_change_entries(&changes)
+        .into_iter()
+        .enumerate()
+        .map(|(idx, entry)| {
+            let category = infer_category(&entry.path);
+            ImplementationTask {
+                id: format!("task-{}", idx + 1),
+                description: entry.description,
+                file_path: entry.path,
+                category,
+                action: entry.action,
+            }
+        })
+        .collect();
+
+    tasks.sort_by_key(|t| t.category);
+    tasks
+}
+
+// ============================================================
+// Private helpers
+// ============================================================
+
+struct ChangeEntry {
+    path: String,
+    description: String,
+    action: TaskAction,
+}
+
+/// Extract everything under `## Changes` up to the next `##` heading.
+fn extract_changes_section(spec: &str) -> String {
+    let marker = "## Changes";
+    if let Some(start) = spec.find(marker) {
+        let rest = &spec[start + marker.len()..];
+        // Stop at the next top-level section (but not sub-sections like ###).
+        if let Some(next) = find_next_h2(rest) {
+            rest[..next].to_string()
+        } else {
+            rest.to_string()
+        }
+    } else {
+        String::new()
+    }
+}
+
+/// Find the position of the next `\n## ` marker in `s`.
+fn find_next_h2(s: &str) -> Option<usize> {
+    s.find("\n## ")
+}
+
+/// Parse change bullets like:
+///
+/// ```text
+/// - `path/to/file.rs`:
+///   - **CREATE**: description
+///   - **MODIFY**: description
+/// ```
+fn parse_change_entries(section: &str) -> Vec<ChangeEntry> {
+    let mut entries = Vec::new();
+    let mut current_path: Option<String> = None;
+
+    for line in section.lines() {
+        let trimmed = line.trim();
+
+        // Detect a path bullet: `- \`path\``
+        if let Some(path) = parse_path_bullet(trimmed) {
+            current_path = Some(path);
+            continue;
+        }
+
+        // Detect action bullets nested under a path: `- **CREATE**:` / `- **MODIFY**:`
+        if let Some(path) = &current_path {
+            if let Some((action, desc)) = parse_action_bullet(trimmed) {
+                entries.push(ChangeEntry {
+                    path: path.clone(),
+                    description: desc,
+                    action,
+                });
+            }
+        }
+    }
+
+    // Fallback: if no nested action bullets were found, treat each path bullet
+    // as a single MODIFY task using the rest of the line as description.
+    if entries.is_empty() {
+        for line in section.lines() {
+            let trimmed = line.trim();
+            if let Some(path) = parse_path_bullet(trimmed) {
+                let desc = trimmed
+                    .trim_start_matches("- ")
+                    .trim_start_matches('`')
+                    .to_string();
+                entries.push(ChangeEntry {
+                    path,
+                    description: desc,
+                    action: TaskAction::Modify,
+                });
+            }
+        }
+    }
+
+    entries
+}
+
+/// Parse `- \`some/path.rs\`...` → returns the path string.
+fn parse_path_bullet(line: &str) -> Option<String> {
+    let line = line.strip_prefix("- `")?;
+    let end = line.find('`')?;
+    let path = line[..end].to_string();
+    if path.is_empty() {
+        None
+    } else {
+        Some(path)
+    }
+}
+
+/// Parse `- **CREATE**: description` or `- **MODIFY**: description`.
+fn parse_action_bullet(line: &str) -> Option<(TaskAction, String)> {
+    let inner = line.strip_prefix("- ")?;
+    if let Some(rest) = inner
+        .strip_prefix("**CREATE**:")
+        .or_else(|| inner.strip_prefix("**Create**:"))
+    {
+        return Some((TaskAction::Create, rest.trim().to_string()));
+    }
+    if let Some(rest) = inner
+        .strip_prefix("**MODIFY**:")
+        .or_else(|| inner.strip_prefix("**Modify**:"))
+        .or_else(|| inner.strip_prefix("**DO**:"))
+        .or_else(|| inner.strip_prefix("**Do**:"))
+    {
+        return Some((TaskAction::Modify, rest.trim().to_string()));
+    }
+    None
+}
+
+/// Infer category from the file path heuristics.
+fn infer_category(path: &str) -> TaskCategory {
+    let p = path.to_lowercase();
+
+    if p.ends_with("_test.rs")
+        || p.ends_with("_spec.rs")
+        || p.contains("/tests/")
+        || p.starts_with("tests/")
+        || p.contains("test_")
+    {
+        return TaskCategory::Test;
+    }
+
+    if p.contains("integrations/")
+        || p.contains("integration/")
+        || p.contains("github")
+        || p.contains("gitlab")
+        || p.contains("jira")
+    {
+        return TaskCategory::Integration;
+    }
+
+    if p.contains("error") || p.contains("types") || p.contains("models") || p.contains("schema")
+    {
+        return TaskCategory::DataModel;
+    }
+
+    TaskCategory::Logic
+}
+
+// ============================================================
+// Tests
+// ============================================================
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    const SAMPLE_SPEC: &str = r#"## Changes
+
+- `crates/cclab-agent/src/error.rs`:
+  - **MODIFY**: Add `MalformedLLMResponse` and `PlatformError` variants.
+
+- `crates/cclab-agent/src/integrations/mod.rs`:
+  - **MODIFY**: Add `create_branch`, `create_commit`, `create_pull_request` to the trait.
+
+- `crates/cclab-agent/src/agents/code_agent/mod.rs`:
+  - **CREATE**: Introduce the `CodeAgent` struct.
+
+- `crates/cclab-agent/tests/code_agent_test.rs`:
+  - **CREATE**: Integration tests for CodeAgent.
+"#;
+
+    #[test]
+    fn test_decompose_returns_tasks() {
+        let tasks = decompose_spec(SAMPLE_SPEC);
+        assert!(!tasks.is_empty(), "should return at least one task");
+    }
+
+    #[test]
+    fn test_topological_order() {
+        let tasks = decompose_spec(SAMPLE_SPEC);
+        // Verify categories are non-decreasing after sort.
+        for w in tasks.windows(2) {
+            assert!(
+                w[0].category <= w[1].category,
+                "tasks out of order: {:?} > {:?}",
+                w[0].category,
+                w[1].category
+            );
+        }
+    }
+
+    #[test]
+    fn test_error_file_is_data_model() {
+        let tasks = decompose_spec(SAMPLE_SPEC);
+        let error_task = tasks
+            .iter()
+            .find(|t| t.file_path.contains("error.rs"))
+            .expect("error.rs task should be present");
+        assert_eq!(error_task.category, TaskCategory::DataModel);
+    }
+
+    #[test]
+    fn test_integration_file_category() {
+        let tasks = decompose_spec(SAMPLE_SPEC);
+        let integration_task = tasks
+            .iter()
+            .find(|t| t.file_path.contains("integrations/mod.rs"))
+            .expect("integrations task should be present");
+        assert_eq!(integration_task.category, TaskCategory::Integration);
+    }
+
+    #[test]
+    fn test_test_file_category() {
+        let tasks = decompose_spec(SAMPLE_SPEC);
+        let test_task = tasks
+            .iter()
+            .find(|t| t.file_path.contains("test"))
+            .expect("test task should be present");
+        assert_eq!(test_task.category, TaskCategory::Test);
+    }
+
+    #[test]
+    fn test_create_action_parsed() {
+        let tasks = decompose_spec(SAMPLE_SPEC);
+        let create_task = tasks
+            .iter()
+            .find(|t| t.file_path.contains("code_agent/mod.rs"))
+            .expect("code_agent task should be present");
+        assert!(matches!(create_task.action, TaskAction::Create));
+    }
+
+    #[test]
+    fn test_modify_action_parsed() {
+        let tasks = decompose_spec(SAMPLE_SPEC);
+        let modify_task = tasks
+            .iter()
+            .find(|t| t.file_path.contains("error.rs"))
+            .expect("error task should be present");
+        assert!(matches!(modify_task.action, TaskAction::Modify));
+    }
+
+    #[test]
+    fn test_empty_spec_returns_empty() {
+        let tasks = decompose_spec("## Overview\nNo changes section.");
+        assert!(tasks.is_empty());
+    }
+
+    #[test]
+    fn test_infer_category_test() {
+        assert_eq!(infer_category("src/foo_test.rs"), TaskCategory::Test);
+        assert_eq!(infer_category("tests/integration.rs"), TaskCategory::Test);
+    }
+
+    #[test]
+    fn test_infer_category_integration() {
+        assert_eq!(
+            infer_category("src/integrations/github.rs"),
+            TaskCategory::Integration
+        );
+    }
+
+    #[test]
+    fn test_infer_category_data_model() {
+        assert_eq!(infer_category("src/error.rs"), TaskCategory::DataModel);
+        assert_eq!(infer_category("src/types.rs"), TaskCategory::DataModel);
+    }
+
+    #[test]
+    fn test_infer_category_logic() {
+        assert_eq!(infer_category("src/agents/code_agent/mod.rs"), TaskCategory::Logic);
+    }
+}

```

## Review: change-impl-agent-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: change-impl-agent

**Summary**: 26 tests pass, compiles clean, 173 total. Includes CodeAgent + parser + task decomposition + GitHub/GitLab integration extensions.

