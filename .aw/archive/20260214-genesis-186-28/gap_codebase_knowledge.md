---
change_id: genesis-186-28
type: gap_codebase_knowledge
created_at: 2026-02-14T03:37:49.862720+00:00
updated_at: 2026-02-14T03:37:49.862720+00:00
---

# Gap Analysis: Codebase vs Knowledge (genesis-186-28)

## Convention Violations

| Code Path | Knowledge Doc Reference | Severity | Description |
|-----------|-------------------------|----------|-------------|
| `crates/cclab-genesis/src/mcp/tools/` | `knowledge:40-mcp/dynamic-config.md` | High | The codebase lacks the documented "Dynamic MCP Configuration" pattern where tools are filtered by stage (decide, plan, implement). All tools appear to be exposed regardless of the current workflow phase. |
| `crates/cclab-genesis/src/models/spec_rules.rs` | `knowledge:spec-to-code/spec-model.md` | Medium | Current spec type rules and `ApiSpecType` lean towards implementation-specific details (OpenAPI/AsyncAPI) instead of the "Agnostic Technical Design" emphasized in the SDD patterns. |

## Pattern Mismatches

| Code Path | Knowledge Doc Reference | Severity | Description |
|-----------|-------------------------|----------|-------------|
| `crates/cclab-genesis/src/services/spec_service.rs` | `knowledge:spec-to-code/code-generator-contract.md` | High | The spec service does not implement the multi-spec contextual inference pattern. It neglects the rich context from Sequence+, Flowchart+, and Requirement+ diagrams during creation/validation, falling into the documented pitfall of only consuming API Specs. |
| `crates/cclab-genesis/src/mcp/tools/run_change/` | `knowledge:40-mcp/index.md` | Medium | The orchestration logic handles phase transitions but does not enforce the dynamic toolset boundaries defined in the MCP architecture. Stage-specific "Tool Sets by Stage" are not structurally implemented. |
| `crates/cclab-genesis/src/mcp/tools/analyze.rs` | `knowledge:30-claude/skills.md` | Low | The code analysis tool operates as a standard MCP tool rather than being integrated into the "Agent Skills Extension" pattern, which would allow it to be triggered by specialized instructions via YAML metadata. |
