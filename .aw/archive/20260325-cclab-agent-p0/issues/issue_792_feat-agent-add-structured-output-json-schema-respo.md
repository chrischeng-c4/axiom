---
number: 792
title: "feat(agent): add structured output (JSON schema response)"
state: open
labels: [enhancement, crate:agent, P0]
group: "structured-output"
---

# #792 — feat(agent): add structured output (JSON schema response)

## Context
Getting LLMs to return structured JSON is one of the most common patterns. OpenAI has `response_format`, Anthropic has tool-use-as-schema. Developers shouldn't need to hand-roll this.

## Scope
- `agent.generate(prompt, response_schema={...}) -> dict`
- Accept JSON Schema or cclab.schema BaseModel class
- Provider-specific implementation:
  - OpenAI: `response_format: { type: "json_schema", json_schema: {...} }`
  - Claude: Tool-use with single tool as schema extractor
  - Gemini: `response_mime_type: "application/json"` + schema
- Auto-retry on malformed output (configurable max retries)
- Validation against schema before returning

## Example
```python
from cclab.agent import Claude

llm = Claude(api_key="...")
result = await llm.generate(
    "Extract entities from: 'John works at Google in NYC'",
    response_schema={
        "type": "object",
        "properties": {
            "person": {"type": "string"},
            "company": {"type": "string"},
            "location": {"type": "string"},
        }
    }
)
# result: {"person": "John", "company": "Google", "location": "NYC"}
```

## Replaces
- `instructor` library
- Manual JSON parsing from LLM output
- `pydantic` + `openai` structured output boilerplate
