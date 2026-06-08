---
change: agent-protocols
group: agent-protocols-module
date: 2026-03-20
---

# Requirements

Add a `src/protocols/` module to `crates/cclab-agent` that defines 5 domain contract types used across all SDD agents. Agents operate on these protocol types; consumers map their ORM models to/from protocols.

Protocol types to define:
- `ProjectProtocol` — id, name, repo_url, platform; used by RestructureCodebaseAgent
- `IssueProtocol` — id, title, description, status, priority, labels, acceptance_criteria; used by RestructureIssueAgent
- `SpecProtocol` — id, path, content, format, version, sync_target; used by ChangeSpecAgent and CodebaseToSpecAgent
- `ChangeProtocol` — id, project_id, issue_ids, spec_ids, branch, status; used by CodeAgent
- `CodeIndexProtocol` — module_path, endpoints, models, dependencies; used by ReferenceCodebaseContextAgent

Consolidation of existing scattered types:
- `Issue`, `IssueState`, `IssueSummary` in `src/integrations/mod.rs` → map to/from `IssueProtocol`
- `SpecExcerpt` in `src/agents/restructure.rs` → superseded by `SpecProtocol`
- `SessionState` in `src/storage/mod.rs` → fields `id`, `issue_ids`, `branch`, `status` partially map to `ChangeProtocol`

Directory structure:
```
src/protocols/
├── mod.rs
├── issue.rs
├── spec.rs
├── change.rs
├── project.rs
└── code_index.rs
```

Key constraint: protocols are pure domain contracts — no ORM, no persistence logic. Plain structs with derive traits (at minimum `Clone`, `Debug`, `Serialize`, `Deserialize`).
