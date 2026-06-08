---
change: sync-adapter
group: sync-adapter
date: 2026-03-20
---

# Requirements

Add a `SyncAdapter` trait in `crate:agent` that provides a generic async abstraction for syncing protocol types (IssueProtocol, SpecProtocol, ChangeProtocol, CodeIndexProtocol) to and from third-party platforms.

The trait interface:
```rust
#[async_trait]
pub trait SyncAdapter: Send + Sync {
    async fn push_issue(&self, issue: &IssueProtocol) -> NovaResult<SyncResult>;
    async fn pull_issue(&self, external_id: &str) -> NovaResult<IssueProtocol>;
    async fn push_spec(&self, spec: &SpecProtocol) -> NovaResult<SyncResult>;
    async fn push_change(&self, change: &ChangeProtocol) -> NovaResult<SyncResult>;
    async fn pull_code_index(&self, path: &str) -> NovaResult<CodeIndexProtocol>;
}
```

Implement platform adapters:
- P1: `GitLabSyncAdapter` (Issue, Code, Change via MR) and `GitHubSyncAdapter` (Issue, Code, Change via PR)
- P2: `JiraSyncAdapter` (Issue only), `ConfluenceSyncAdapter` (Spec only), `GDocsSyncAdapter` (Spec only)

Depends on #958 — protocol types (`IssueProtocol`, `SpecProtocol`, `ChangeProtocol`, `CodeIndexProtocol`) must already exist.

Acceptance criteria:
- Trait is defined with all five async methods
- P1 adapters compile and pass unit tests with mocked HTTP responses
- Each adapter only implements the methods relevant to its domain (others return `Err(NotSupported)`)
- Auth credentials are injected at construction time (not global state)
