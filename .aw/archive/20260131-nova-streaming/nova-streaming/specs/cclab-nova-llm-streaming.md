---
id: cclab-nova-llm-streaming
type: spec
title: "Unified LLM Streaming and Multi-Provider Support"
version: 1
spec_type: integration
created_at: 2026-01-31T02:51:23.774692+00:00
updated_at: 2026-01-31T02:51:23.774692+00:00
requirements:
  total: 9
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
    - R8
    - R9
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-31T02:51:23.774692+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Unified LLM Streaming and Multi-Provider Support

## Overview

Unified streaming interface and multi-provider support (Claude, OpenAI, Gemini) for cclab-nova-llm and cclab-nucleus Python bindings. This spec covers the internal Rust implementation and the exposure to Python.

## Requirements

### R1 - HttpClient Streaming Support

```yaml
id: R1
priority: high
status: draft
```

Add execute_stream to cclab-photon::HttpClient that returns an async stream of Bytes using reqwest.

### R2 - Unified Streaming Model

```yaml
id: R2
priority: high
status: draft
```

Introduce StreamResponse struct in cclab-nova-llm and update LLMProvider trait complete_stream return type.

### R3 - Fix Claude Provider Types

```yaml
id: R3
priority: high
status: draft
```

Fix ClaudeProvider type mismatches in CompletionResponse (usage, metadata) and ToolCall arguments.

### R4 - Claude Streaming

```yaml
id: R4
priority: high
status: draft
```

Implement ClaudeProvider::complete_stream returning unified StreamResponse.

### R5 - Gemini Provider Completion

```yaml
id: R5
priority: high
status: draft
```

Implement GeminiProvider in cclab-nova-llm using Google AI REST API for basic completion.

### R6 - Gemini Streaming support

```yaml
id: R6
priority: high
status: draft
```

Implement GeminiProvider::complete_stream using streamGenerateContent and parsing chunked JSON.

### R7 - OpenAI Streaming Alignment

```yaml
id: R7
priority: medium
status: draft
```

Update OpenAIProvider::complete_stream to return the new StreamResponse type.

### R8 - Python AsyncIterator Wrapper

```yaml
id: R8
priority: high
status: draft
```

Implement PyStreamResponse in cclab-nucleus/src/agent/py_llm.rs as a Python AsyncIterator.

### R9 - Python Provider Integration

```yaml
id: R9
priority: high
status: draft
```

Expose Claude and Gemini providers to Python and implement complete_stream in PyO3 bindings.

## Acceptance Criteria

### Scenario: Claude Streaming Happy Path

- **GIVEN** A configured ClaudeProvider with a valid API key.
- **WHEN** complete_stream is called with a simple user message.
- **THEN** The provider returns a stream of StreamChunks containing delta content.

### Scenario: Gemini Completion Happy Path

- **GIVEN** A configured GeminiProvider with a valid API key.
- **WHEN** complete is called with a simple user message.
- **THEN** The provider returns a CompletionResponse with generated content and usage stats.

### Scenario: Python Async Loop Streaming

- **GIVEN** An LLM provider instance in Python.
- **WHEN** The user iterates over complete_stream using 'async for chunk in provider.complete_stream(...)'.
- **THEN** The loop executes for each chunk returned by the provider.

## Flow Diagram

```mermaid
sequenceDiagram
    actor U as User / Python Agent
    participant P as LLMProvider (Claude/Gemini/OpenAI)
    participant C as HttpClient (cclab-photon)
    participant API as External LLM API (Anthropic/Google/OpenAI)
    U->>+P: complete_stream(request)
    P->C: execute_stream(req)
    C->>+API: send HTTP request
    API-->>C: stream chunk
    C-->>P: bytes chunk
    P-->>U: StreamChunk
    C->>-API: stream closed
```

</spec>
