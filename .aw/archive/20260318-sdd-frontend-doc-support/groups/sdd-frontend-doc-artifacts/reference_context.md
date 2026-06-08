---
change: sdd-frontend-doc-support
group: sdd-frontend-doc-artifacts
date: 2026-03-18
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-sdd/logic/change-spec.md | cclab-sdd | high | Section type system — 17 types including wireframe, component, design-token, overview, Section fill order and CLI-driven generation with per-type flags, Spec preparation from spec_plan (sections, main_spec_ref), Cross-reference system with content-level id and $ref, Section selection rule engine (keyword matching) |
| cclab-sdd/logic/reference-context.md | cclab-sdd | high | spec_plan array with sections field, Section selection via rule engine, Review checklist includes spec_plan validation |
| cclab-sdd/logic/implement-task.md | cclab-sdd | high | Group-scoped spec paths: groups/{group_id}/specs/, Per-spec implementation loop — needs extension for doc_target writes, Codegen routing (has_json_schema or has_api_spec) |
| cclab-sdd/logic/change-merge.md | cclab-sdd | high | Merge from groups/*/specs/ to cclab/specs/, Merge strategy: new | update — needs extension for doc append|replace, Frontmatter stripping (fill_sections, filled_sections, etc.) |
| cclab-sdd/generate/spec-model.md | cclab-sdd | low | Background: spec-to-code model for Mermaid Plus diagram types |
| cclab-sdd/interfaces/cli/commands.md | cclab-sdd | medium | CLI command tree — artifact subcommand structure, CLI-to-logic mapping for artifact commands |

