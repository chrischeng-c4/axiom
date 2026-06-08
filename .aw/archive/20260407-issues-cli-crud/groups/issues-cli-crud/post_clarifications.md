---
change: issues-cli-crud
group: issues-cli-crud
date: 2026-04-06
status: skipped
---

# Post-Clarifications

## Scope Summary

### Problem
score issues CLI has list/show/sync (read path) but no write verbs (create/update/close) or search. → requirements.md § R1-R4

### Success Criteria
All IssueBackend CRUD verbs uniform across Local + GitHub + GitLab. Cross-artifact references (related:/implements:) in frontmatter. Agent-first --json + structured errors. → requirements.md § R5-R8

### Boundary
In scope: R1-R8 as specified. Out: Jira backend, bidirectional sync, authoring agent prompt.

