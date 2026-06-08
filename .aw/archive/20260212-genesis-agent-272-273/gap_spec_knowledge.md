---
change_id: genesis-agent-272-273
type: gap_spec_knowledge
created_at: 2026-02-12T11:30:19.778597+00:00
updated_at: 2026-02-12T11:30:19.778597+00:00
---

# Gap Analysis: Spec vs Knowledge

## Spec-Knowledge Misalignments

### GAP-SK1: Dynamic MCP tool filtering not in delegate-agent spec (LOW)
- **Knowledge**: `40-mcp/index.md` documents per-stage tool filtering (plan=22 tools, implement=4-5, review=3-4)
- **Spec**: `delegate-agent.md` doesn't address which MCP tools sub-agents should have access to per action type
- **Impact**: Low — sub-agents get all tools currently; filtering is a separate concern from delegate_agent itself

### GAP-SK2: Spec references old tool name (LOW)
- **Knowledge**: Knowledge docs reference `genesis_agent` (recursion prevention pattern)
- **Spec**: `delegate-agent.md` also uses `genesis_agent` throughout
- **Impact**: Both spec and knowledge need name update to `genesis_delegate_agent` after rename

## No Gaps Found

- `delegate-agent.md` verification table aligns with knowledge conventions (structured responses, artifact-based)
- `prompt-registry.md` per-action template approach aligns with skills knowledge (progressive disclosure)
- Error recovery spec section doesn't contradict any knowledge pattern

## Summary

| Gap | Severity | Type |
|-----|----------|------|
| GAP-SK1: MCP tool filtering | LOW | Spec omission |
| GAP-SK2: Tool name in spec/knowledge | LOW | Naming sync |

Overall: Spec and knowledge are well-aligned for this change. The gaps are minor naming/scope issues."