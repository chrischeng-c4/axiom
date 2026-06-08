---
id: implementation
type: change_implementation
change_id: agent-protocols
---

# Implementation

## Summary

New protocols module in crates/cclab-agent across 7 files: (1) src/protocols/mod.rs (27 lines): module root with crate-doc table mapping each protocol to its owning agent; (2) src/protocols/issue.rs (187 lines): IssueProtocol struct with IssueStatus/IssuePriority enums, From<&Issue> and From<&IssueSummary> conversions, IssueProtocol::new() constructor, 3 unit tests; (3) src/protocols/spec.rs (123 lines): SpecProtocol struct with SpecFormat enum (6 variants: OpenRpc/JsonSchema/Mermaid/Yaml/Markdown/Prose), Display impl, new() constructor, 3 unit tests; (4) src/protocols/change.rs (115 lines): ChangeProtocol struct with ChangeStatus enum (Pending/InProgress/Review/Merged/Cancelled), new() constructor, 3 unit tests; (5) src/protocols/project.rs (71 lines): ProjectProtocol struct with Platform enum (GitHub/GitLab/Jira/Other(String)), Display impl, 2 unit tests; (6) src/protocols/code_index.rs (69 lines): CodeIndexProtocol struct with Vec<String> fields for endpoints/models/dependencies, new() constructor, 2 unit tests; (7) src/lib.rs modified: added pub mod protocols + re-export block for all 10 public protocol types. All types are pure domain contracts (Clone+Debug+Serialize+Deserialize), no ORM or persistence logic. Existing types retained without breaking changes; From conversions bridge old and new.

## Diff

```diff
diff --git a/crates/cclab-agent/src/lib.rs b/crates/cclab-agent/src/lib.rs
--- a/crates/cclab-agent/src/lib.rs
+++ b/crates/cclab-agent/src/lib.rs
@@ -67,6 +67,7 @@
 pub mod llm;
+pub mod protocols;
 pub mod security;
 pub mod storage;
 pub mod stream;
@@ -147,3 +148,12 @@
 pub use integrations::{
     CommitFile, CreatedBranch, CreatedCommit, CreatedPullRequest, GitHubIntegration,
     GitLabIntegration, Issue, IssueComment, IssueFilter, IssueState, IssueSummary, JiraIntegration,
     PlatformIntegration, PullRequestParams,
 };
+
+// Re-export protocol types
+pub use protocols::{
+    ChangeProtocol, ChangeStatus,
+    CodeIndexProtocol,
+    IssueProtocol, IssuePriority, IssueStatus,
+    Platform, ProjectProtocol,
+    SpecFormat, SpecProtocol,
+};
diff --git a/crates/cclab-agent/src/protocols/mod.rs b/crates/cclab-agent/src/protocols/mod.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/protocols/mod.rs
@@ -0,0 +1,27 @@
+//! Protocols module — domain contracts for all 5 SDD domains.
+//!
+//! Protocol types are pure domain contracts: no ORM, no persistence logic.
+//! Agents read and write protocol types; consumers (Conductor, etc.) map
+//! their storage models to/from protocols.
+//!
+//! # Protocols
+//!
+//! | Protocol | Key Fields | Agent |
+//! |----------|-----------|-------|
+//! | [`ProjectProtocol`] | id, name, repo_url, platform | RestructureCodebaseAgent |
+//! | [`IssueProtocol`] | id, title, description, status, priority, labels, acceptance_criteria | RestructureIssueAgent |
+//! | [`SpecProtocol`] | id, path, content, format, version, sync_target | ChangeSpecAgent, CodebaseToSpecAgent |
+//! | [`ChangeProtocol`] | id, project_id, issue_ids, spec_ids, branch, status | CodeAgent |
+//! | [`CodeIndexProtocol`] | module_path, endpoints, models, dependencies | ReferenceCodebaseContextAgent |
+
+mod change;
+mod code_index;
+mod issue;
+mod project;
+mod spec;
+
+pub use change::{ChangeProtocol, ChangeStatus};
+pub use code_index::CodeIndexProtocol;
+pub use issue::{IssueProtocol, IssuePriority, IssueStatus};
+pub use project::{Platform, ProjectProtocol};
+pub use spec::{SpecFormat, SpecProtocol};
diff --git a/crates/cclab-agent/src/protocols/issue.rs b/crates/cclab-agent/src/protocols/issue.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/protocols/issue.rs
@@ -0,0 +1,187 @@
+//! IssueProtocol — domain contract for an issue/ticket.
+//!
+//! Consolidates `Issue`, `IssueState`, and `IssueSummary` from
+//! `integrations/mod.rs` into a single, agent-facing type.
+
+use crate::integrations::{Issue, IssueState, IssueSummary};
+use serde::{Deserialize, Serialize};
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum IssueStatus {
+    Open,
+    Closed,
+    InProgress,
+    Resolved,
+    Wontfix,
+}
+
+impl Default for IssueStatus {
+    fn default() -> Self { Self::Open }
+}
+
+impl From<IssueState> for IssueStatus {
+    fn from(state: IssueState) -> Self {
+        match state {
+            IssueState::Open => IssueStatus::Open,
+            IssueState::Closed => IssueStatus::Closed,
+            IssueState::InProgress => IssueStatus::InProgress,
+            IssueState::Resolved => IssueStatus::Resolved,
+            IssueState::Wontfix => IssueStatus::Wontfix,
+        }
+    }
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum IssuePriority { Low, Medium, High, Critical }
+
+impl Default for IssuePriority {
+    fn default() -> Self { Self::Medium }
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct IssueProtocol {
+    pub id: String,
+    pub title: String,
+    pub description: String,
+    pub status: IssueStatus,
+    pub priority: IssuePriority,
+    pub labels: Vec<String>,
+    pub acceptance_criteria: Vec<String>,
+}
+
+impl IssueProtocol {
+    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
+        Self {
+            id: id.into(), title: title.into(),
+            description: String::new(),
+            status: IssueStatus::default(),
+            priority: IssuePriority::default(),
+            labels: Vec::new(),
+            acceptance_criteria: Vec::new(),
+        }
+    }
+}
+
+impl From<&Issue> for IssueProtocol {
+    fn from(issue: &Issue) -> Self {
+        Self {
+            id: issue.id.clone(),
+            title: issue.title.clone(),
+            description: issue.body.clone(),
+            status: IssueStatus::from(issue.state),
+            priority: IssuePriority::default(),
+            labels: issue.labels.clone(),
+            acceptance_criteria: Vec::new(),
+        }
+    }
+}
+
+impl From<&IssueSummary> for IssueProtocol {
+    fn from(summary: &IssueSummary) -> Self {
+        Self {
+            id: summary.id.clone(),
+            title: summary.title.clone(),
+            description: String::new(),
+            status: IssueStatus::from(summary.state),
+            priority: IssuePriority::default(),
+            labels: summary.labels.clone(),
+            acceptance_criteria: Vec::new(),
+        }
+    }
+}
diff --git a/crates/cclab-agent/src/protocols/spec.rs b/crates/cclab-agent/src/protocols/spec.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/protocols/spec.rs
@@ -0,0 +1,123 @@
+//! SpecProtocol — domain contract for a specification file.
+
+use serde::{Deserialize, Serialize};
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum SpecFormat { OpenRpc, JsonSchema, Mermaid, Yaml, Markdown, Prose }
+
+impl Default for SpecFormat {
+    fn default() -> Self { Self::Markdown }
+}
+
+impl std::fmt::Display for SpecFormat {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        let s = match self {
+            SpecFormat::OpenRpc => "open_rpc",
+            SpecFormat::JsonSchema => "json_schema",
+            SpecFormat::Mermaid => "mermaid",
+            SpecFormat::Yaml => "yaml",
+            SpecFormat::Markdown => "markdown",
+            SpecFormat::Prose => "prose",
+        };
+        write!(f, "{}", s)
+    }
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct SpecProtocol {
+    pub id: String,
+    pub path: String,
+    pub content: String,
+    pub format: SpecFormat,
+    pub version: String,
+    pub sync_target: Option<String>,
+}
+
+impl SpecProtocol {
+    pub fn new(id: impl Into<String>, path: impl Into<String>, content: impl Into<String>) -> Self {
+        Self {
+            id: id.into(), path: path.into(), content: content.into(),
+            format: SpecFormat::default(),
+            version: String::new(),
+            sync_target: None,
+        }
+    }
+}
diff --git a/crates/cclab-agent/src/protocols/change.rs b/crates/cclab-agent/src/protocols/change.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/protocols/change.rs
@@ -0,0 +1,115 @@
+//! ChangeProtocol — domain contract for a code change (issue -> spec -> branch -> PR).
+
+use serde::{Deserialize, Serialize};
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum ChangeStatus { Pending, InProgress, Review, Merged, Cancelled }
+
+impl Default for ChangeStatus {
+    fn default() -> Self { Self::Pending }
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ChangeProtocol {
+    pub id: String,
+    pub project_id: String,
+    pub issue_ids: Vec<String>,
+    pub spec_ids: Vec<String>,
+    pub branch: Option<String>,
+    pub status: ChangeStatus,
+}
+
+impl ChangeProtocol {
+    pub fn new(id: impl Into<String>, project_id: impl Into<String>, issue_ids: Vec<String>) -> Self {
+        Self {
+            id: id.into(), project_id: project_id.into(),
+            issue_ids,
+            spec_ids: Vec::new(),
+            branch: None,
+            status: ChangeStatus::default(),
+        }
+    }
+}
diff --git a/crates/cclab-agent/src/protocols/project.rs b/crates/cclab-agent/src/protocols/project.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/protocols/project.rs
@@ -0,0 +1,71 @@
+//! ProjectProtocol — domain contract for a project/repository.
+
+use serde::{Deserialize, Serialize};
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum Platform { GitHub, GitLab, Jira, #[serde(untagged)] Other(String) }
+
+impl std::fmt::Display for Platform {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        match self {
+            Platform::GitHub => write!(f, "github"),
+            Platform::GitLab => write!(f, "gitlab"),
+            Platform::Jira => write!(f, "jira"),
+            Platform::Other(s) => write!(f, "{}", s),
+        }
+    }
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ProjectProtocol {
+    pub id: String,
+    pub name: String,
+    pub repo_url: String,
+    pub platform: Platform,
+}
diff --git a/crates/cclab-agent/src/protocols/code_index.rs b/crates/cclab-agent/src/protocols/code_index.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/protocols/code_index.rs
@@ -0,0 +1,69 @@
+//! CodeIndexProtocol — domain contract for a codebase module index entry.
+
+use serde::{Deserialize, Serialize};
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct CodeIndexProtocol {
+    pub module_path: String,
+    pub endpoints: Vec<String>,
+    pub models: Vec<String>,
+    pub dependencies: Vec<String>,
+}
+
+impl CodeIndexProtocol {
+    pub fn new(module_path: impl Into<String>) -> Self {
+        Self {
+            module_path: module_path.into(),
+            endpoints: Vec::new(),
+            models: Vec::new(),
+            dependencies: Vec::new(),
+        }
+    }
+}
```

## Review: agent-protocols-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: agent-protocols

**Summary**: 242 tests pass.

