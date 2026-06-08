---
number: 958
title: "feat(agent): Add protocols module — domain contracts for Issue, Spec, Change, Project, CodeIndex"
state: open
labels: [enhancement, crate:agent, P0]
group: "agent-protocols-module"
---

# #958 — feat(agent): Add protocols module — domain contracts for Issue, Spec, Change, Project, CodeIndex

## Summary

Define protocol types (domain contracts) for all 5 SDD domains. Agents operate on these types. Consumers (Conductor, etc.) map their ORM models to/from protocols.

## Protocols

| Protocol | Key Fields | Used By |
|----------|-----------|---------|
| ProjectProtocol | id, name, repo_url, platform | RestructureCodebaseAgent |
| IssueProtocol | id, title, description, status, priority, labels, AC | RestructureIssueAgent |
| SpecProtocol | id, path, content, format, version, sync_target | ChangeSpecAgent, CodebaseToSpecAgent |
| ChangeProtocol | id, project_id, issue_ids, spec_ids, branch, status | CodeAgent |
| CodeIndexProtocol | module_path, endpoints, models, dependencies | ReferenceCodebaseContextAgent |

## Existing Types to Consolidate

- Issue, IssueState, IssueSummary (integrations/mod.rs) → IssueProtocol
- SpecExcerpt (agents/restructure.rs) → part of SpecProtocol
- SessionState (storage/mod.rs) → partially maps to ChangeProtocol

## Structure

```
src/protocols/
├── mod.rs
├── issue.rs
├── spec.rs
├── change.rs
├── project.rs
└── code_index.rs
```

## Key Principle

Protocol = contract. Not ORM, not persistence. Agent reads/writes protocols. Consumer maps to their own storage.
