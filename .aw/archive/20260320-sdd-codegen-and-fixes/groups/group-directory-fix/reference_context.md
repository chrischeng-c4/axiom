---
change: sdd-codegen-and-fixes
group: group-directory-fix
date: 2026-03-20
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-sdd/logic/change-spec.md | ? | high | spec files written to groups/{group-id}/specs/, fill prompts written to groups/{group-id}/prompts/, spec preparation uses group-scoped paths |
| cclab-sdd/logic/change-merge.md | ? | high | merge reader iterates groups/*/specs/ per group, backward compat detects both root-level and group-level spec layouts, Q1 requirement |
| cclab-sdd/logic/state-machine.md | ? | high | add active_group field to State schema, document set/clear during spec/impl phases, Q3 requirement: STATE.yaml tracks active group via active_group field |
| cclab-sdd/logic/implement-task.md | ? | high | impl prompts written to groups/{group-id}/prompts/, prompt templates reference groups/{{group_id}}/... paths |
| cclab-sdd/tools/utils/write-artifact.md | ? | medium | change_spec artifact path = groups/{group_id}/specs/{spec_id}.md, payload location convention updated for group-level routing |
| cclab-sdd/README.md | ? | low | background orientation and index only |
| cclab-sdd/sdd-cli.md | ? | medium | STATE.yaml interop (active_group field support), parity with MCP execution (group-scoped paths) |

