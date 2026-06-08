---
change: cclab-agent-p0
group: structured-output
date: 2026-03-16
status: answered
---

# Pre-Clarifications

### Q1: schema-input-format
- **Answer**: Support both raw JSON Schema dicts and cclab.schema BaseModel classes. BaseModel auto-converts to JSON Schema internally.

### Q2: retry-strategy
- **Answer**: Re-send full prompt with validation error appended as feedback. Default max_retries=3.

