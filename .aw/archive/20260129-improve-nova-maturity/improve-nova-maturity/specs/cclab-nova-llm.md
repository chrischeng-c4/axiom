---
id: cclab-nova-llm
type: spec
title: "cclab-nova-llm Specification"
version: 1
spec_type: integration
created_at: 2026-01-28T08:40:42.779341+00:00
updated_at: 2026-01-28T08:40:42.779341+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-28T08:40:42.779341+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# cclab-nova-llm Specification

## Overview

Unified LLM provider interface and implementation for multiple providers (OpenAI, Anthropic Claude) and gateways (LiteLLM, OpenRouter). Supports streaming, tool calling, and structured output integration.

## Requirements

### R1 - Claude Provider Enhancements

```yaml
id: R1
priority: high
status: draft
```

Fix ClaudeProvider compilation errors (chained calls on HttpClient) and implement streaming support using Server-Sent Events (SSE).

### R2 - Gateway Support

```yaml
id: R2
priority: medium
status: draft
```

Add support for LiteLLM and OpenRouter as providers/gateways. Implement dynamic routing logic.

### R3 - Full Streaming Support

```yaml
id: R3
priority: high
status: draft
```

Ensure all providers correctly propagate streaming chunks to the Agent layer.

## Acceptance Criteria

### Scenario: Claude streaming response

- **GIVEN** A ClaudeProvider with a valid API key.
- **WHEN** A streaming completion request is sent to Claude.
- **THEN** A stream of StreamChunk objects is returned, containing content deltas.

### Scenario: Gateway routing to specific model

- **GIVEN** An OpenRouter provider configuration.
- **WHEN** A request is made to a model available via OpenRouter.
- **THEN** The request is correctly routed to the chosen model through the gateway.

## Flow Diagram

```mermaid
sequenceDiagram
    participant Client as Python Client
    participant Agent as Nova Agent (Rust)
    participant LLM as LLM Provider (Rust)
    participant ProviderAPI as External LLM API
    Client->Agent: Agent.run(input, stream=True)
    Agent->LLM: LLMProvider.complete_stream(request)
    LLM->ProviderAPI: OpenAI/Claude API Request (stream=true)
    ProviderAPI->LLM: SSE Chunk 1 (content delta)
    LLM->Agent: StreamChunk(delta)
    Agent->Client: Yield Chunk 1
    ProviderAPI->LLM: SSE Chunk N (finish_reason)
    LLM->Agent: StreamChunk(final)
    Agent->Client: Yield Final Chunk + Finalize Context
```

</spec>
