---
change: cclab-agent-p0
group: structured-output
date: 2026-03-16
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| llm-providers | cclab-agent | high | CompletionRequest needs response_schema field, Provider-specific structured output: OpenAI response_format, Claude tool-use-as-schema, Gemini response_mime_type |
| core-types | cclab-agent | high | ToolCall/ToolDefinition types used by Claude schema-extractor pattern, Message types for retry flow |
| error-handling | cclab-agent | medium | Need new error variant for schema validation failure and malformed JSON, Retry logic uses is_retriable classification |
| agents | cclab-agent | medium | Agent.run and agent execution loop where generate() is called, CodingAgentConfig/AnalystAgentConfig may need structured output options |
| architecture | cclab-agent | low | Module dependency: agents -> llm -> types |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: cclab-agent-p0

**Verdict**: APPROVED

### Summary

All relevant specs covered. llm-providers (CompletionRequest/Response) and core-types (ToolCall/ToolDefinition for Claude schema pattern) are correctly marked high. error-handling and agents are appropriate medium references. No missing coverage.

### Checklist

- ✅ All affected crates/areas from pre-clarifications are covered by at least one spec
  - cclab-agent crate fully covered by llm-providers, core-types, agents specs
- ✅ Relevance scores are reasonable
  - high: llm-providers (where response_schema goes), core-types (ToolCall for Claude pattern); medium: error-handling (new error variants), agents (execution loop); low: architecture (dependency overview)
- ✅ Key requirements listed per spec are accurate
  - Verified against actual spec contents — CompletionRequest schema, ToolCall/ToolDefinition types, NovaError variants all match
- ✅ No irrelevant specs included
  - All 5 specs are relevant to structured output implementation

### Issues

No issues found.
