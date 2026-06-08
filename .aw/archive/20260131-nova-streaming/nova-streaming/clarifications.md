---
change: nova-streaming
date: 2026-01-31
---

# Clarifications

## Q1: Providers
- **Question**: Which LLM providers should be supported for streaming?
- **Answer**: Claude, OpenAI, Gemini - three main providers
- **Rationale**: Cover the major commercial LLM APIs. Mistral can be added later if needed.

## Q2: Interface
- **Question**: What streaming interface pattern should we use?
- **Answer**: AsyncIterator pattern
- **Rationale**: Standard Rust async stream, compatible with Python async for loop. Clean and idiomatic.

## Q3: Response Type
- **Question**: Should we add SSE server support for forwarding streams?
- **Answer**: Use StreamResponse, not SSE
- **Rationale**: StreamResponse is more flexible and integrates better with existing cclab infrastructure.

