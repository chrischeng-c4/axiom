---
id: fillback-main-specs-workflow
type: proposal
version: 1
created_at: 2026-02-06T04:28:32.453117+00:00
updated_at: 2026-02-06T04:28:32.453117+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add /cclab:genesis:fillback-main-specs skill for generating rich specs from existing code"
history:
  - timestamp: 2026-02-06T04:28:32.453117+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 4
  new_files: 2
affected_specs:
  - id: fillback-skill-definition
    path: specs/fillback-skill-definition.md
    depends: []
---

<proposal>

# Change: fillback-main-specs-workflow

## Summary

Add /cclab:genesis:fillback-main-specs skill for generating rich specs from existing code

## Why

The Genesis SDD workflow assumes specs exist before implementation, but existing codebases lack formal specifications. The current fillback engine (CodeStrategy) only generates basic symbol tables and dependency lists — not the rich spec format needed for spec-to-code roundtripping (requirements, scenarios, Mermaid diagrams, OpenAPI/OpenRPC/AsyncAPI/JSON Schema). A dedicated workflow skill is needed that orchestrates existing MCP tools (analyze_code_for_spec, write_main_spec, Aurora generators, Prism analysis) to produce Genesis-compatible main specs directly from code.

## What Changes

- Create new skill definition at .claude/skills/cclab-genesis-fillback-main-specs/SKILL.md with full workflow orchestration logic
- Create matching template at crates/cclab-genesis/templates/mainthread/skills/cclab-genesis-fillback-main-specs/SKILL.md for cclab init distribution
- Update CLAUDE.md (both project and template) to register the new skill in the workflow table
- Skill implements two modes: mono-repo (one component at a time via workspace detection) and non-mono-repo (AI-assessed dynamic chunking by functional domain)
- Per-component pipeline: AST scan → Prism analysis → AI enrichment (LLM generates requirements, scenarios, diagrams, API specs) → write_main_spec → optional validation

## Impact

- **Scope**: minor
- **Affected Files**: ~4
- **New Files**: ~2
- Affected specs:
  - `fillback-skill-definition` (no dependencies)
- Affected code: `.claude/skills/cclab-genesis-fillback-main-specs/SKILL.md`, `crates/cclab-genesis/templates/mainthread/skills/cclab-genesis-fillback-main-specs/SKILL.md`, `CLAUDE.md`, `crates/cclab-genesis/templates/mainthread/CLAUDE.md`

</proposal>
