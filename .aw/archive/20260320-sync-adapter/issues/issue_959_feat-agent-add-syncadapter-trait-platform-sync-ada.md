---
number: 959
title: "feat(agent): Add SyncAdapter trait + platform sync adapters"
state: open
labels: [enhancement, crate:agent, P1]
group: "sync-adapter"
---

# #959 — feat(agent): Add SyncAdapter trait + platform sync adapters

**Depends**: #958 (protocols)

## Summary

Generic sync abstraction between protocol types and third-party platforms.

## Interface

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

## Adapters

| Adapter | Domains | Priority |
|---------|---------|----------|
| GitLabSyncAdapter | Issue, Code, Change (MR) | P1 |
| GitHubSyncAdapter | Issue, Code, Change (PR) | P1 |
| JiraSyncAdapter | Issue only | P2 |
| ConfluenceSyncAdapter | Spec only | P2 |
| GDocsSyncAdapter | Spec only | P2 |
