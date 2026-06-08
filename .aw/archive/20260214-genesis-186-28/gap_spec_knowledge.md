---
change_id: genesis-186-28
type: gap_spec_knowledge
created_at: 2026-02-14T03:38:39.736463+00:00
updated_at: 2026-02-14T03:38:39.736463+00:00
---

# Gap Analysis: Spec vs Knowledge (genesis-186-28)

## Contradictions

| Spec ID | Knowledge Doc Reference | Severity | Description |
|---------|-------------------------|----------|-------------|
| `cclab-genesis/create-spec` | `knowledge:40-mcp/dynamic-config.md` | High | The `create-spec` specification does not account for the Dynamic MCP toolsets. Its prompt templates and enrichment logic assume a flat tool space, potentially leading to the "Exposing too many tools" pitfall documented in the knowledge base. |
| `cclab-genesis/review-spec` | `knowledge:spec-to-code/code-generator-contract.md` | Medium | The review spec focuses on individual spec validation, which contradicts the knowledge base requirement for generators to consume "richer context" from multiple spec types (Sequence+, Flowchart+, etc.). |

## Missing Patterns

| Pattern Name | Knowledge Doc Reference | Severity | Description |
|--------------|-------------------------|----------|-------------|
| **Dynamic MCP Toolsets** | `knowledge:40-mcp/dynamic-config.md` | High | There is no specification defining how the workflow stage (decide, plan, implement) will trigger the specific MCP tool filtering described in the knowledge base. This core infrastructure piece is unmapped. |
| **Agent Skills Extension** | `knowledge:30-claude/skills.md` | Medium | The skill-based triggering pattern (SKILL.md) is not reflected in the `create-spec` or `delegate-agent` specs. The mechanism for exposing SDD capabilities as reusable skills is missing. |
| **Multi-Spec Contextual Inference** | `knowledge:spec-to-code/code-generator-contract.md` | Medium | The pattern of aggregating context from across the spec group (Requirements + Sequence + Flow) for high-fidelity code generation is not specified in the current planning flow. |

## Boundary Misalignments

| Boundary Conflict | Severity | Description |
|-------------------|----------|-------------|
| `review-spec` vs `spec-validator` | Medium | The division of responsibility between the automated completeness check (in `spec-validator`) and the human-in-the-loop review cycle (in `review-spec`) is undefined, particularly regarding "Threshold Escalation" logic. |
| `create-spec` vs `mermaid-plus-conversion` | Low | Blurry boundary on whether "Automated Aurora detection" belongs in the creation orchestration or the diagram conversion service. |
