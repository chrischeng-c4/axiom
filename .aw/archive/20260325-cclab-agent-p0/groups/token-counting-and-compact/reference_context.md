---
change: cclab-agent-p0
group: token-counting-and-compact
date: 2026-03-16
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| context | cclab-agent | high | ContextManager.compress() is the target for smart auto-compact, estimated_tokens uses len/4 — must replace with accurate counting, Compression flow: FIFO removal must be upgraded to LLM summarization, Message window: system prompt always kept, oldest compressed first |
| core-types | cclab-agent | high | TokenUsage type (prompt_tokens, completion_tokens, total_tokens), Message type with tool_calls field — needed for tool-call pairing protection, ToolCall/ToolResult types — must be kept as pairs during compaction |
| llm-providers | cclab-agent | high | CompletionResponse.usage returns TokenUsage — source for API-reported token counts, LLMProvider.complete() used to call summarization model during compaction, CompletionRequest for summarization calls |
| agents | cclab-agent | medium | Agent execution loop calls ContextManager — where auto-compact triggers, CodingAgentConfig/AnalystAgentConfig need compact_model config field, max_context_tokens field drives compaction threshold |
| error-handling | cclab-agent | medium | ContextOverflow error variant — triggered when compaction fails to free enough tokens |
| streaming | cclab-agent | low | StreamChunk.usage — streaming also reports token usage |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: cclab-agent-p0

**Verdict**: APPROVED

### Summary

Comprehensive coverage. context.md (high) directly targets compress() and token estimation. core-types (high) covers TokenUsage and tool-call pairing types. llm-providers (high) provides API-reported usage and summarization call path. agents/error-handling (medium) and streaming (low) are appropriate supporting references.

### Checklist

- ✅ All affected crates/areas from pre-clarifications are covered by at least one spec
  - Token counting covered by context + core-types, auto-compact by context + llm-providers, tool-call pairing by core-types
- ✅ Relevance scores are reasonable
  - Three high specs all directly implement requirements; agents/error-handling medium; streaming low
- ✅ Key requirements listed per spec are accurate
  - Verified: context.md has compress() and len/4 estimation, core-types has TokenUsage/ToolCall, llm-providers has CompletionResponse.usage
- ✅ No irrelevant specs included
  - All 6 specs are relevant to token counting and compaction

### Issues

No issues found.
