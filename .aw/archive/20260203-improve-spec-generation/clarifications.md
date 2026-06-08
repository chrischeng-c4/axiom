---
change: improve-spec-generation
date: 2026-01-27
---

# Clarifications

## Q1: Validation Strategy
- **Question**: How should validation failures be handled when spec is missing required design elements?
- **Answer**: Hard fail - MCP tools validate format, LLM focuses on content quality
- **Rationale**: MCP tools already have structured input schemas that enforce required fields. The validation happens at the tool level, not after-the-fact. This ensures specs are always well-formed.

## Q2: Example Storage
- **Question**: Where should the formal spec examples be stored?
- **Answer**: Inline in prompts - LLM focuses on content, MCP validates format
- **Rationale**: The MCP tools (genesis_create_spec) already have strict schemas. The prompt just needs to guide Gemini on WHAT content to produce; the MCP tool ensures HOW it's structured.

## Q3: Scope
- **Question**: Which spec types should be enforced?
- **Answer**: All spec_types: http-api, rpc-api, event-driven, workflow, data-model, algorithm, utility
- **Rationale**: Comprehensive enforcement ensures consistency. The spec_type field already exists and should drive the required design elements.

