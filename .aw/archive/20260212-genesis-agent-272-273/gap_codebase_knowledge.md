---
change_id: genesis-agent-272-273
type: gap_codebase_knowledge
created_at: 2026-02-12T11:29:51.606192+00:00
updated_at: 2026-02-12T11:29:51.606192+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention Violations

### GAP-K1: MCP response size convention (MEDIUM)
- **Knowledge**: `40-mcp/index.md` documents that MCP tools return concise structured JSON, with artifact content read separately via `genesis_read_file`
- **Code**: `agent.rs:340` returns raw LLM stdout in `output` field, violating the convention
- **Impact**: Responses are 10x larger than other genesis MCP tools

### GAP-K2: Recursion prevention naming (LOW)
- **Knowledge**: `40-mcp/index.md` pattern documents blocking `genesis_agent` in sub-agent configs
- **Code**: `config.rs:115`, `config.rs:249` hardcode `genesis_agent` in excludeTools/disabled_tools
- **Impact**: After rename to `genesis_delegate_agent`, config generators and knowledge docs need sync

## Pattern Mismatches

### GAP-K3: Skill naming convention (LOW)
- **Knowledge**: `30-claude/skills.md` documents skill naming as lowercase-with-hyphens
- **Code**: Skill at `.claude/skills/cclab-genesis-agent/SKILL.md` — name and directory must match new tool name
- **Impact**: Skill rename required but straightforward

## No Gaps Found

- MCP HTTP transport conventions: not directly relevant (delegate_agent spawns CLI subprocesses, not HTTP calls)
- Dynamic MCP configuration: not directly affected by this change

## Summary

| Gap | Severity | Type |
|-----|----------|------|
| GAP-K1: Response size convention | MEDIUM | Convention violation |
| GAP-K2: Recursion prevention naming | LOW | Naming sync |
| GAP-K3: Skill naming | LOW | Naming sync |"