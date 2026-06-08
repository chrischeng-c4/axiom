---
change_id: genesis-agent-272-273
type: gap_codebase_spec
created_at: 2026-02-12T11:29:14.599946+00:00
updated_at: 2026-02-12T11:29:14.599946+00:00
---

# Gap Analysis: Codebase vs Spec

## Spec → Code Gaps (spec exists, code missing/incomplete)

### GAP-1: Action enum (HIGH)
- **Spec**: `delegate-agent.md` defines 30+ actions in OpenRPC enum
- **Code**: `agent.rs:44-47` only accepts `explore | review | custom`
- **Impact**: All workflow actions (explore_spec, create_proposal, review_spec, etc.) fail with 'Unknown action' error

### GAP-2: Response schema (HIGH)
- **Spec**: `delegate-agent.md` response requires `{status, change_id, action, verification, usage, next}`
- **Code**: `agent.rs:340-349` returns `{output (raw stdout), exit_code, usage}` — includes full LLM output, missing structured fields
- **Impact**: Oversized responses (~10.6k tokens), no structured verification in response

### GAP-3: Tool name (HIGH)
- **Spec**: Per clarification, should be `genesis_delegate_agent`
- **Code**: `agent.rs:25` name is `genesis_agent`
- **Impact**: 11 files reference old name (mod.rs, helpers.rs, config.rs, cli_mapper.rs, skill, etc.)

### GAP-4: Error recovery — retry + fallback chain (MEDIUM)
- **Spec**: `delegate-agent.md` Error Recovery section defines retry-once then fallback to next agent in executor chain
- **Code**: No retry/fallback logic in `execute_streaming()` — single attempt, failure returns error
- **Impact**: Agent failures are not recoverable without mainthread intervention

### GAP-5: Per-action prompt templates (MEDIUM)
- **Spec**: `prompt-registry.md` R2 requires per-action prompt generation; `delegate-agent.md` implies action-specific context
- **Code**: `agent.rs:325-329` only has `LLM_EXPLORE` and `LLM_REVIEW` templates, all workflow actions would need `custom` passthrough
- **Impact**: Delegated agents lack structured prompts for workflow actions

## Code → Spec Gaps (code exists, spec incomplete)

### GAP-6: Streaming architecture (LOW)
- **Code**: `execute_streaming()` uses `mpsc::Sender<String>` for SSE streaming, `call_tool_streaming()` wraps it
- **Spec**: `delegate-agent.md` doesn't mention streaming — only describes final response
- **Impact**: Streaming is an internal optimization, not affecting the MCP response contract

### GAP-7: Telemetry recording (LOW)
- **Code**: `agent.rs:388-404` records LlmCall telemetry to STATE.yaml
- **Spec**: `delegate-agent.md` Side Effects section documents this correctly
- **Impact**: No gap — spec and code aligned

## Summary

| Gap | Severity | Direction | Status |
|-----|----------|-----------|--------|
| GAP-1: Action enum | HIGH | Spec→Code | Must fix |
| GAP-2: Response schema | HIGH | Spec→Code | Must fix |
| GAP-3: Tool name | HIGH | Spec→Code | Must fix |
| GAP-4: Error recovery | MEDIUM | Spec→Code | Should fix |
| GAP-5: Per-action templates | MEDIUM | Spec→Code | Should fix |
| GAP-6: Streaming | LOW | Code→Spec | Informational |
| GAP-7: Telemetry | LOW | Code→Spec | Already aligned |"