---
change: cclab-agent-p0
group: structured-output
date: 2026-03-16
---

# Requirements

## Structured Output (#792)
- Add `response_schema` parameter to agent.generate() that accepts JSON Schema or cclab.schema BaseModel
- Provider-specific implementation:
  - OpenAI: use `response_format: { type: "json_schema", json_schema: {...} }`
  - Claude: use tool-use with single tool as schema extractor
  - Gemini: use `response_mime_type: "application/json"` + schema
- Auto-retry on malformed JSON output (configurable max_retries)
- Validate response against schema before returning
- Return typed dict matching the schema
