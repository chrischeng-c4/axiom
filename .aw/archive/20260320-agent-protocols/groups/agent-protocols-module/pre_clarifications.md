---
change: agent-protocols
group: agent-protocols-module
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Replace or keep existing types?
- **Answer**: Keep existing types alongside. Add From/Into conversions. Migration later. No breaking changes now.

### Q2: General
- **Question**: ID type?
- **Answer**: All protocols use id: String. Consumers convert. String is the universal denominator — works for GitHub numbers, GitLab IIDs, Jira keys, UUIDs.

### Q3: General
- **Question**: ChangeProtocol vs SessionState?
- **Answer**: ChangeProtocol is a pure contract (no methods). SessionState keeps its own storage logic. Add From<SessionState> for ChangeProtocol. No replacement.

### Q4: General
- **Question**: CodeIndex fields?
- **Answer**: Use Vec<String> for now (free-form). Keep it simple. Structured types can come later when we have concrete use cases.

### Q5: General
- **Question**: sync_target type?
- **Answer**: Option<SyncTarget> where SyncTarget is an enum: Local, Confluence { page_id }, GoogleDocs { doc_id }, Custom { url }. None means not synced.

