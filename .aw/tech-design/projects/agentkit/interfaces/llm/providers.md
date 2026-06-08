---
id: agent-llm-providers-spec
main_spec_ref: "agent/interfaces/llm/providers.md"
fill_sections: [overview, schema, interaction, changes]
---

# LLM Providers Spec

## Overview
<!-- type: overview lang: markdown -->

The LLM provider interface defines the shared completion contract for
`agent`. `LLMProvider` exposes provider identity, supported model
validation, non-streaming completion, and streaming completion through one async
trait.

Claude, OpenAI, and Gemini providers adapt the shared `CompletionRequest`,
`CompletionResponse`, `ToolDefinition`, and `StreamChunk` DTOs to
provider-specific HTTP APIs. All providers support a custom base URL so callers
can route through internal gateways or compatible proxy services.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  CompletionRequest:
    type: object
    required: [messages, model, stream, extras]
    properties:
      messages:
        type: array
        items:
          $ref: "agent/interfaces/core/types.md#/definitions/Message"
      model: {type: string}
      temperature:
        type: number
        minimum: 0
        maximum: 2
      max_tokens:
        type: integer
        minimum: 1
      top_p:
        type: number
        minimum: 0
        maximum: 1
      stop:
        type: array
        items: {type: string}
      stream: {type: boolean}
      tools:
        type: array
        items:
          $ref: "#/definitions/ToolDefinition"
      response_schema:
        type: object
        additionalProperties: true
      extras:
        type: object
        additionalProperties: true

  ToolDefinition:
    type: object
    required: [name, description, parameters]
    properties:
      name: {type: string}
      description: {type: string}
      parameters:
        type: object
        additionalProperties: true

  CompletionResponse:
    type: object
    required: [content, finish_reason, usage, model, metadata]
    properties:
      content: {type: string}
      tool_calls:
        type: array
        items:
          $ref: "agent/interfaces/core/types.md#/definitions/ToolCall"
      finish_reason: {type: string}
      usage:
        $ref: "agent/interfaces/core/types.md#/definitions/TokenUsage"
      model: {type: string}
      metadata:
        type: object
        additionalProperties: true

  StreamChunk:
    type: object
    required: [content, is_final]
    properties:
      content: {type: string}
      tool_calls:
        type: array
        items:
          $ref: "agent/interfaces/core/types.md#/definitions/ToolCall"
      finish_reason: {type: string}
      is_final: {type: boolean}

  ProviderBaseUrls:
    type: object
    required: [anthropic, openai, google]
    properties:
      anthropic: {type: string, const: "https://api.anthropic.com"}
      openai: {type: string, const: "https://api.openai.com"}
      google: {type: string, const: "https://generativelanguage.googleapis.com"}
```

## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: llm-provider-class-hierarchy
title: LLM Provider Class Hierarchy
---
classDiagram
    class LLMProvider {
        <<trait>>
        +provider_name() str
        +supported_models() Vec~String~
        +validate_model(model) NovaResult
        +complete(CompletionRequest) NovaResult~CompletionResponse~
        +complete_stream(CompletionRequest) NovaResult~StreamResponse~
    }

    class ClaudeProvider {
        -api_key: String
        -default_model: String
        +new(api_key) NovaResult
        +with_base_url(api_key, base_url) NovaResult
        +with_default_model(model) Self
    }

    class OpenAIProvider {
        -api_key: String
        -default_model: String
        +new(api_key) NovaResult
        +with_base_url(api_key, base_url) NovaResult
        +with_default_model(model) Self
    }

    class GeminiProvider {
        -api_key: String
        -default_model: String
        -base_url: String
        +new(api_key) NovaResult
        +with_base_url(api_key, base_url) NovaResult
        +with_default_model(model) Self
    }

    ClaudeProvider ..|> LLMProvider
    OpenAIProvider ..|> LLMProvider
    GeminiProvider ..|> LLMProvider
```

```mermaid
---
id: llm-supported-models
title: LLM Supported Models
---
graph TD
    subgraph Claude
        CS4["claude-sonnet-4-20250514"]
        CO4["claude-opus-4-20250514"]
        CS35a["claude-3-5-sonnet-20241022"]
        CS35b["claude-3-5-sonnet-20240620"]
        CO3["claude-3-opus-20240229"]
        CS3["claude-3-sonnet-20240229"]
        CH3["claude-3-haiku-20240307"]
    end

    subgraph OpenAI
        G4O["gpt-4o"]
        G4OM["gpt-4o-mini"]
        G4T["gpt-4-turbo"]
        G4["gpt-4"]
        G35["gpt-3.5-turbo"]
        O1["o1"]
        O1M["o1-mini"]
    end

    subgraph Gemini
        G2F["gemini-2.0-flash"]
        G2FL["gemini-2.0-flash-lite"]
        G15P["gemini-1.5-pro"]
        G15F["gemini-1.5-flash"]
    end
```

```mermaid
---
id: llm-message-conversion
title: LLM Message Conversion
---
flowchart LR
    subgraph Internal
        SYS[System]
        USR[User]
        AST[Assistant]
        TL[Tool]
    end

    subgraph Claude
        C_SYS["system param"]
        C_USR["role user"]
        C_AST["role assistant + tool_use blocks"]
        C_TL["role user + tool_result blocks"]
    end

    subgraph OpenAI
        O_SYS["role system"]
        O_USR["role user"]
        O_AST["role assistant + tool_calls"]
        O_TL["role tool + tool_call_id"]
    end

    subgraph Gemini
        G_SYS["systemInstruction"]
        G_USR["role user"]
        G_AST["role model"]
        G_TL["role user + functionResponse"]
    end

    SYS --> C_SYS
    SYS --> O_SYS
    SYS --> G_SYS
    USR --> C_USR
    USR --> O_USR
    USR --> G_USR
    AST --> C_AST
    AST --> O_AST
    AST --> G_AST
    TL --> C_TL
    TL --> O_TL
    TL --> G_TL
```

```mermaid
---
id: llm-streaming-support
title: LLM Streaming Support
---
graph LR
    Claude -->|SSE| Implemented["implemented"]
    OpenAI --> Pending["not implemented"]
    Gemini --> Pending2["not implemented"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agent/core/src/llm/provider.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Define CompletionRequest, ToolDefinition, CompletionResponse, StreamChunk, StreamResponse, and the LLMProvider trait."
  - path: projects/agent/core/src/llm/claude.rs
    action: modify
    section: interaction
    impl_mode: hand-written
    description: "Adapt shared provider DTOs to Anthropic Messages API requests, tool calls, structured output, and SSE streaming."
  - path: projects/agent/core/src/llm/openai.rs
    action: modify
    section: interaction
    impl_mode: hand-written
    description: "Adapt shared provider DTOs to OpenAI-compatible chat completion requests and tool calls."
  - path: projects/agent/core/src/llm/gemini.rs
    action: modify
    section: interaction
    impl_mode: hand-written
    description: "Adapt shared provider DTOs to Gemini generateContent requests, function calls, and response schema settings."
```
