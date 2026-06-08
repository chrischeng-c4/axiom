---
change: sync-adapter
group: sync-adapter
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Bidirectional
- **Answer**: Bidirectional is composed at caller level (push + pull). No explicit method needed. The trait stays simple.

### Q2: General
- **Question**: SyncResult
- **Answer**: SyncResult { external_id: String, external_url: Option<String>, status: SyncStatus }. SyncStatus enum: Synced, Failed { error: String }, Skipped { reason: String }.

### Q3: General
- **Question**: P2 adapters scope
- **Answer**: Stub only for P2. Jira/Confluence/GDocs return Err(NovaError::NotSupported) for now. GitLab and GitHub fully implemented.

### Q4: General
- **Question**: Auth
- **Answer**: Constructor injection. Pass config struct (api_key, base_url, etc.) when building the adapter. Same pattern as LLMProvider.

### Q5: General
- **Question**: Partial domain support
- **Answer**: Split into domain-specific sub-traits: IssueSyncAdapter, SpecSyncAdapter, CodeSyncAdapter, ChangeSyncAdapter. Each platform implements only what it supports. Cleaner than NotSupported errors.

