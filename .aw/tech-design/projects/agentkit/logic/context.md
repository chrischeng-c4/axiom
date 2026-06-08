---
id: agent-context-spec
main_spec_ref: "agent/logic/context.md"
fill_sections: [overview, schema, logic, changes]
---

# Context Management Spec

## Overview
<!-- type: overview lang: markdown -->

`ContextManager` owns conversation history and token-budget accounting for
agent requests. It keeps an optional system prompt, stores non-system messages,
tracks estimated token usage, reserves response budget, and compacts old
messages when usage crosses the configured threshold.

Compaction prefers provider-backed summarization when an `LLMProvider` is
configured. If summarization is unavailable or fails, the manager falls back to
FIFO removal while preserving recent messages and tool-call pairing.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ContextStats:
    type: object
    required:
      - message_count
      - estimated_tokens
      - max_tokens
      - available_tokens
      - compression_triggered
    properties:
      message_count: {type: integer, minimum: 0}
      estimated_tokens: {type: integer, minimum: 0}
      max_tokens: {type: integer, minimum: 0}
      available_tokens: {type: integer, minimum: 0}
      compression_triggered: {type: boolean}

  ContextPreset:
    type: object
    required: [name, max_tokens]
    properties:
      name:
        type: string
        enum: [default_128k, default_32k]
      max_tokens:
        type: integer
        enum: [128000, 32000]

  CompressionPolicy:
    type: object
    required:
      - reserved_tokens
      - compact_threshold
      - keep_recent
      - fallback
    properties:
      reserved_tokens: {type: integer, const: 4096}
      compact_threshold: {type: number, const: 0.8}
      keep_recent: {type: integer, const: 4}
      fallback:
        type: string
        const: fifo_pair_preserving_removal
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: context-manager-class
title: ContextManager Class
---
classDiagram
    class ContextManager {
        -messages: Vec~Message~
        -system_prompt: Option~String~
        -max_tokens: u32
        -current_tokens: u32
        -reserved_tokens: u32
        -metadata: HashMap~String, Value~
        -compact_threshold: f64
        -keep_recent: usize
        +new(max_tokens) ContextManager
        +for_model(model, max_tokens) ContextManager
        +default_128k() ContextManager
        +default_32k() ContextManager
        +set_system_prompt(prompt)
        +add_message(msg: Message)
        +add_user_message(content: str)
        +add_assistant_message(content: str)
        +add_tool_result(tool_call_id: str, content: str)
        +get_messages() Vec~Message~
        +available_tokens() u32
        +message_count() usize
        +current_token_count() u32
        +stats() ContextStats
    }
```

```mermaid
---
id: context-compression-flow
title: Context Compression Flow
---
flowchart TD
    ADD[add_message] --> COUNT[estimate message tokens]
    COUNT --> CHECK{current_tokens > max_tokens * compact_threshold?}
    CHECK -->|no| DONE[message retained]
    CHECK -->|yes| HAS_PROVIDER{compact_provider configured?}
    HAS_PROVIDER -->|yes| SUMMARY[try provider summarization]
    SUMMARY -->|success| RECALC[recalculate tokens]
    SUMMARY -->|error| FIFO[fifo compression]
    HAS_PROVIDER -->|no| FIFO
    FIFO --> PAIR[preserve assistant tool-call pairs]
    PAIR --> KEEP[keep recent messages]
    KEEP --> RECALC
    RECALC --> DONE
```

```mermaid
---
id: context-message-window
title: Context Message Window
---
graph LR
    subgraph AlwaysKept["Always Kept"]
        SYS["system prompt"]
    end

    subgraph Compressible["Compressible"]
        M1["msg 1 oldest"]
        M2["msg 2"]
        M3["msg 3"]
    end

    subgraph Retained["Recent Retained"]
        MN1["msg N-1"]
        MN["msg N newest"]
    end

    SYS --- M1
    M1 --- M2
    M2 --- M3
    M3 -.- MN1
    MN1 --- MN
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agent/core/src/context.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Define ContextManager token accounting, provider-backed summarization, FIFO compaction, metadata access, and ContextStats."
  - path: projects/agent/core/src/tokenizer.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Provide model-aware and estimate tokenizers used by ContextManager."
```
