---
change: sdd-issue-centric
group: issue-centric-workflow
date: 2026-04-09
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| ? | - | high | Phase enum lives in STATE.yaml — must move to issue frontmatter, Phase transitions validated in phase_transition.rs — must read/write issue instead, run-change reads STATE.yaml phase to determine next action — must read issue frontmatter |
| ? | - | high | issue_parser.rs already parses structured sections — extend to read/write phase field, Frontmatter schema needs phase, branch, git_workflow fields |
| ? | - | medium | Issue CRUD operations — score issues list must show phase column, Issue frontmatter schema — add new fields |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| sdd-issue-centric | create | crates/sdd/logic/issue-centric-workflow.md | overview, schema, state-machine, changes |

