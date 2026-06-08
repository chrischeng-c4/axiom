---
id: implementation
type: change_implementation
change_id: sync-adapter
---

# Implementation

## Summary

New `sync_adapter` module in `crates/cclab-agent/src/` providing a generic async `SyncAdapter` trait plus five platform adapters (GitLab, GitHub, Jira, Confluence, Google Docs). Closes #959.

## Changed Files

```
M	crates/cclab-agent/src/error.rs
M	crates/cclab-agent/src/lib.rs
A	crates/cclab-agent/src/sync_adapter/mod.rs
A	crates/cclab-agent/src/sync_adapter/gitlab.rs
A	crates/cclab-agent/src/sync_adapter/github.rs
A	crates/cclab-agent/src/sync_adapter/jira.rs
A	crates/cclab-agent/src/sync_adapter/confluence.rs
A	crates/cclab-agent/src/sync_adapter/gdocs.rs
```

## Diff

```diff
diff --git a/crates/cclab-agent/src/error.rs b/crates/cclab-agent/src/error.rs
--- a/crates/cclab-agent/src/error.rs
+++ b/crates/cclab-agent/src/error.rs
@@ -55,6 +55,9 @@
+    #[error("Operation not supported by this adapter: {0}")]
+    NotSupported(String),
+
     // LLM errors
     #[error("HTTP error: {0}")]
     HttpError(String),

diff --git a/crates/cclab-agent/src/lib.rs b/crates/cclab-agent/src/lib.rs
--- a/crates/cclab-agent/src/lib.rs
+++ b/crates/cclab-agent/src/lib.rs
@@ -72,6 +72,7 @@
+pub mod sync_adapter;
 pub mod tokenizer;
 pub mod tools;

@@ -157,3 +157,10 @@
+
+// Re-export sync adapter types
+pub use sync_adapter::{
+    ConfluenceSyncAdapter, GDocsSyncAdapter, GitHubSyncAdapter, GitLabSyncAdapter,
+    JiraSyncAdapter, SyncAction, SyncAdapter, SyncResult,
+};

diff --git a/crates/cclab-agent/src/sync_adapter/mod.rs b/crates/cclab-agent/src/sync_adapter/mod.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/sync_adapter/mod.rs
@@ -0,0 +1,143 @@
+//! SyncAdapter trait — generic async abstraction for syncing protocol types
+//! to and from third-party platforms.
+//!
+//! | Adapter                   | Domains                   | Priority |
+//! |---------------------------|---------------------------|----------|
+//! | GitLabSyncAdapter         | Issue, Code, Change (MR)  | P1       |
+//! | GitHubSyncAdapter         | Issue, Code, Change (PR)  | P1       |
+//! | JiraSyncAdapter           | Issue only                | P2       |
+//! | ConfluenceSyncAdapter     | Spec only                 | P2       |
+//! | GDocsSyncAdapter          | Spec only                 | P2       |
+
+pub enum SyncAction { Created, Updated, NoChange }
+
+pub struct SyncResult {
+    pub external_id: String,
+    pub url: Option<String>,
+    pub action: SyncAction,
+}
+
+impl SyncResult {
+    pub fn created(external_id, url) -> Self { ... }
+    pub fn updated(external_id, url) -> Self { ... }
+    pub fn no_change(external_id) -> Self { ... }
+}
+
+#[async_trait]
+pub trait SyncAdapter: Send + Sync {
+    async fn push_issue(&self, issue: &IssueProtocol) -> NovaResult<SyncResult>;
+    async fn pull_issue(&self, external_id: &str) -> NovaResult<IssueProtocol>;
+    async fn push_spec(&self, spec: &SpecProtocol) -> NovaResult<SyncResult>;
+    async fn push_change(&self, change: &ChangeProtocol) -> NovaResult<SyncResult>;
+    async fn pull_code_index(&self, path: &str) -> NovaResult<CodeIndexProtocol>;
+}
+// Tests: SyncResult::created, updated, no_change, serde roundtrip (4 tests)

diff --git a/crates/cclab-agent/src/sync_adapter/gitlab.rs b/crates/cclab-agent/src/sync_adapter/gitlab.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/sync_adapter/gitlab.rs
@@ -0,0 +1,273 @@
+//! GitLabSyncAdapter (P1): Issue push+pull, Change via MR, Code index.
+//! push_spec → NotSupported. Auth: PRIVATE-TOKEN PAT.
+
+pub struct GitLabSyncAdapter { client: HttpClient, token: String, base_url: String, project_id: String }
+
+impl GitLabSyncAdapter {
+    pub fn new(token, base_url, project_id) -> NovaResult<Self>
+    // base_url trailing slash stripped; project_id URL-encoded in api_url()
+    fn api_url(&self, path) -> String  // {base_url}/api/v4/projects/{encoded_id}/{path}
+    async fn get<T: Deserialize>(&self, url) -> NovaResult<T>   // PRIVATE-TOKEN header
+    async fn post<T: Deserialize>(&self, url, body: Value) -> NovaResult<T>
+}
+
+impl SyncAdapter for GitLabSyncAdapter {
+    // push_issue  → POST issues {title, description, labels CSV}
+    //               → SyncResult::created(iid, "{base_url}/{project_id}/-/issues/{iid}")
+    // pull_issue  → GET issues/{id} → IssueProtocol (closed→Closed, else Open)
+    // push_spec   → Err(NotSupported("use ConfluenceSyncAdapter"))
+    // push_change → POST merge_requests {title, source_branch, target_branch=main, description}
+    //               requires ChangeProtocol.branch (else InvalidArguments)
+    //               → SyncResult::created(iid, mr.web_url)
+    // pull_code_index → GET repository/files/{encoded_path}/raw?ref=main
+    //                   → deserialise JSON or fallback CodeIndexProtocol::new(path)
+}
+// Tests: new(), api_url(), api_url_trailing_slash(), push_spec=NotSupported, push_change_no_branch=InvalidArguments

diff --git a/crates/cclab-agent/src/sync_adapter/github.rs b/crates/cclab-agent/src/sync_adapter/github.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/sync_adapter/github.rs
@@ -0,0 +1,280 @@
+//! GitHubSyncAdapter (P1): Issue push+pull, Change via PR, Code index.
+//! push_spec → NotSupported. Auth: Bearer PAT.

+pub struct GitHubSyncAdapter { client: HttpClient, token: String, owner: String, repo: String }
+
+impl GitHubSyncAdapter {
+    pub fn new(token, owner, repo) -> NovaResult<Self>
+    fn api_url(&self, path) -> String  // https://api.github.com/repos/{owner}/{repo}/{path}
+    async fn get<T: Deserialize>(&self, url) -> NovaResult<T>
+    // headers: Authorization Bearer, Accept: application/vnd.github.v3+json, User-Agent: cclab-nova
+    async fn post<T: Deserialize>(&self, url, body: Value) -> NovaResult<T>
+}
+
+impl SyncAdapter for GitHubSyncAdapter {
+    // push_issue  → POST issues {title, body, labels}
+    //               → SyncResult::created(number, html_url)
+    // pull_issue  → GET issues/{id} → IssueProtocol (labels[].name, closed→Closed)
+    // push_spec   → Err(NotSupported("use GDocsSyncAdapter"))
+    // push_change → POST pulls {title, head=branch, base=main, body}
+    //               requires ChangeProtocol.branch (else InvalidArguments)
+    //               → SyncResult::created(pr.number, pr.html_url)
+    // pull_code_index → GET contents/{path}
+    //                   → base64-decode file.content (strip newlines), deserialise or fallback
+}
+// Tests: new(), api_url(), push_spec=NotSupported, push_change_no_branch=InvalidArguments

diff --git a/crates/cclab-agent/src/sync_adapter/jira.rs b/crates/cclab-agent/src/sync_adapter/jira.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/sync_adapter/jira.rs
@@ -0,0 +1,275 @@
+//! JiraSyncAdapter (P2): Issue only. Auth: Basic (email:api_token Base64).

+pub struct JiraSyncAdapter { client, base_url, auth_header, project_key, issue_type }
+
+impl JiraSyncAdapter {
+    pub fn new(base_url, email, api_token, project_key, issue_type) -> NovaResult<Self>
+    // auth_header = "Basic " + base64(email + ":" + api_token)
+    fn api_url(&self, path) -> String  // {base_url}/rest/api/3/{path}
+}
+
+impl SyncAdapter for JiraSyncAdapter {
+    // push_issue → POST /rest/api/3/issue
+    //   body: ADF paragraph wrapping issue.description
+    //   → SyncResult::created(key, "{base_url}/browse/{key}")
+    // pull_issue → GET /rest/api/3/issue/{id}
+    //   ADF text extraction: content[0].content[0].content[0].text
+    //   status: Done|Closed|Resolved→Resolved, In Progress→InProgress, else Open
+    // push_spec / push_change / pull_code_index → Err(NotSupported)
+}
+// Tests: new(), api_url(), push_spec=NotSupported, push_change=NotSupported, pull_code_index=NotSupported

diff --git a/crates/cclab-agent/src/sync_adapter/confluence.rs b/crates/cclab-agent/src/sync_adapter/confluence.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/sync_adapter/confluence.rs
@@ -0,0 +1,210 @@
+//! ConfluenceSyncAdapter (P2): Spec only. Auth: Basic (email:api_token Base64).

+pub struct ConfluenceSyncAdapter { client, base_url, auth_header, space_key }

+impl SyncAdapter for ConfluenceSyncAdapter {
+    // push_issue / pull_issue / push_change / pull_code_index → Err(NotSupported)
+    // push_spec → POST /rest/api/content
+    //   type: "page", space: {key}, body.storage.value: "<p>{html_escaped_content}</p>"
+    //   title: last path segment of spec.id, dashes/underscores→spaces
+    //   → SyncResult::created(id, "{base_url}/pages/{id}")
+}
+// Tests: new(), api_url(), push_issue/pull_issue/push_change/pull_code_index=NotSupported

diff --git a/crates/cclab-agent/src/sync_adapter/gdocs.rs b/crates/cclab-agent/src/sync_adapter/gdocs.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-agent/src/sync_adapter/gdocs.rs
@@ -0,0 +1,191 @@
+//! GDocsSyncAdapter (P2): Spec only. Auth: OAuth 2.0 Bearer.

+pub struct GDocsSyncAdapter { client, token, folder_id: Option<String> }

+impl SyncAdapter for GDocsSyncAdapter {
+    // push_issue / pull_issue / push_change / pull_code_index → Err(NotSupported)
+    // push_spec → two-step:
+    //   1. POST https://docs.googleapis.com/v1/documents {title}  → GDocsDocument {document_id}
+    //   2. POST .../documents/{id}:batchUpdate {requests: [{insertText: {location: {index:1}, text}}]}
+    //   title: last path segment, dashes/underscores→spaces
+    //   → SyncResult::created(document_id, "https://docs.google.com/document/d/{id}")
+    //   (folder_id held for caller-side Drive move; not auto-applied)
+}
+// Tests: new(), new_with_folder(), push_issue/pull_issue/push_change/pull_code_index=NotSupported
```

## Acceptance Criteria

- [x] `SyncAdapter` trait defined with all 5 async methods
- [x] `GitLabSyncAdapter` (P1): Issue push+pull, MR push, code index pull
- [x] `GitHubSyncAdapter` (P1): Issue push+pull, PR push, code index pull (base64)
- [x] `JiraSyncAdapter` (P2): Issue push+pull only
- [x] `ConfluenceSyncAdapter` (P2): Spec push only
- [x] `GDocsSyncAdapter` (P2): Spec push only (create + batchUpdate)
- [x] Unsupported methods return `Err(NovaError::NotSupported(...))`
- [x] Auth credentials injected at construction time (no global state)
- [x] `NovaError::NotSupported(String)` variant added to `error.rs`
- [x] All 8 types re-exported from `lib.rs`
- [x] 26 unit tests across all 6 files

## Review: sync-adapter-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sync-adapter

**Summary**: 272 tests pass.

