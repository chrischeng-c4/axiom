---
id: agent-runtime-agents
main_spec_ref: "agent/logic/agents/runtime.md"
fill_sections: [overview, schema, interaction, state-machine, dependency, changes]
---

# Agent Runtime

## Overview
<!-- type: overview lang: markdown -->

The SDD agent runtime is centered on the `Agent` trait in
`projects/agentic-workflow/src/agents/mod.rs`. `CodingAgent` provides tool-backed coding
turns with security approval and context compaction. `AnalystAgent` uses the
same completion loop with session persistence, analysis tools, platform
integration tools, and pause/resume behavior for user clarification.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Agent:
    type: object
    required: [run, run_with_handler]
    properties:
      run:
        type: string
        const: "async fn run(&self, input: &str) -> NovaResult<String>"
      run_with_handler:
        type: string
        const: "async fn run_with_handler(&self, input: &str, handler: &dyn StreamHandler) -> NovaResult<String>"

  ApprovalHandler:
    type: object
    required: [request_approval]
    properties:
      request_approval:
        type: string
        const: "async fn request_approval(ApprovalRequest) -> NovaResult<ApprovalResponse>"

  CodingAgentConfig:
    type: object
    required:
      - system_prompt
      - max_turns
      - model
      - temperature
      - max_tokens
      - max_context_tokens
      - compact_model
    properties:
      system_prompt: {type: string}
      max_turns: {type: integer, minimum: 1, default: 20}
      model: {type: string, default: "claude-sonnet-4-20250514"}
      temperature: {type: number, nullable: true, default: 0.0}
      max_tokens: {type: integer, nullable: true, default: 8192}
      max_context_tokens: {type: integer, minimum: 1, default: 128000}
      compact_model: {type: string, default: "claude-3-haiku-20240307"}

  AnalystAgentConfig:
    type: object
    required:
      - system_prompt
      - max_turns
      - model
      - temperature
      - max_tokens
      - max_context_tokens
      - compact_model
    properties:
      system_prompt: {type: string}
      max_turns: {type: integer, minimum: 1, default: 30}
      model: {type: string, default: "claude-sonnet-4-20250514"}
      temperature: {type: number, nullable: true, default: 0.3}
      max_tokens: {type: integer, nullable: true, default: 8192}
      max_context_tokens: {type: integer, minimum: 1, default: 128000}
      compact_model: {type: string, default: "claude-3-haiku-20240307"}

  BuilderRequirement:
    type: object
    required: [agent, required_fields, defaulted_fields]
    properties:
      agent: {type: string}
      required_fields:
        type: array
        items: {type: string}
      defaulted_fields:
        type: array
        items: {type: string}
```

## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: agent-runtime-loop
title: Agent Runtime Loop
---
sequenceDiagram
    participant U as User
    participant A as Agent
    participant C as ContextManager
    participant L as LLMProvider
    participant X as ToolExecutor
    participant S as SecurityPolicy
    participant H as StreamHandler

    U->>A: run_with_handler(input)
    A->>H: Started
    A->>C: set_system_prompt and add_user_message

    loop turn <= max_turns
        A->>L: complete(CompletionRequest)
        L-->>A: CompletionResponse
        A->>C: add assistant message
        A->>H: TextChunk when content exists

        alt tool calls present
            loop each tool call
                A->>H: ToolCallRequested
                A->>S: requires_approval(tool_name)
                alt approval required
                    A->>H: ApprovalRequested
                    A->>A: request approval through ApprovalHandler
                    A->>H: ApprovalReceived
                end
                A->>H: ToolExecutionStarted
                A->>X: execute(tool_name, arguments)
                X-->>A: JSON output or error
                A->>C: add_tool_result
                A->>H: ToolExecutionCompleted or ToolExecutionFailed
            end
        else final response
            A->>H: TurnCompleted
            A->>H: Completed
            A-->>U: response content
        end
    end
```

## State Machine
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: analyst-agent-session-state
title: Analyst Agent Session State
initial: Active
nodes:
  Active:
    kind: normal
    label: "Active run loop"
  Paused:
    kind: normal
    label: "Waiting for user clarification"
  Completed:
    kind: terminal
    label: "Final response returned"
  Failed:
    kind: terminal
    label: "Runtime error"
edges:
  - from: Active
    to: Active
    event: "LLM turn without pause"
  - from: Active
    to: Paused
    event: "tool output type=user_input_required"
  - from: Active
    to: Completed
    event: "final LLM response"
  - from: Active
    to: Failed
    event: "max turns or handler cancellation"
  - from: Paused
    to: Active
    event: "resume_with_response"
  - from: Paused
    to: Failed
    event: "resume attempted when session not paused"
---
stateDiagram-v2
    [*] --> Active: run or run_conversation
    Active --> Active: LLM turn without pause
    Active --> Paused: tool output type user_input_required
    Active --> Completed: final LLM response
    Active --> Failed: max turns or handler cancellation
    Paused --> Active: resume_with_response
    Paused --> Failed: resume attempted when session not paused
    Completed --> [*]
    Failed --> [*]
```

## Dependency
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: agent-runtime-dependencies
title: Agent Runtime Dependencies
---
classDiagram
    class Agent {
        +run(input) NovaResult~String~
        +run_with_handler(input, handler) NovaResult~String~
    }
    class ApprovalHandler {
        +request_approval(request) NovaResult~ApprovalResponse~
    }
    class CodingAgent {
        config
        provider
        registry
        security
        approval_handler
    }
    class AnalystAgent {
        config
        provider
        registry
        security
        storage
        session
        integrations
    }
    class CodingAgentBuilder
    class AnalystAgentBuilder

    CodingAgent ..|> Agent
    AnalystAgent ..|> Agent
    CodingAgentBuilder --> CodingAgent
    AnalystAgentBuilder --> AnalystAgent
    CodingAgent --> ApprovalHandler
    CodingAgent --> SecurityPolicy
    CodingAgent --> ToolRegistry
    AnalystAgent --> Storage
    AnalystAgent --> PlatformIntegration
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/mod.rs
    action: modify
    section: schema
    impl_mode: codegen
    description: "Define AutoApproveHandler while keeping Agent and ApprovalHandler traits hand-written."
  - path: projects/agentic-workflow/src/agents/coding.rs
    action: modify
    section: schema
    impl_mode: codegen
    description: "Define CodingAgentConfig including compact_model."
  - path: projects/agentic-workflow/src/agents/coding.rs
    action: modify
    section: interaction
    impl_mode: hand-written
    description: "Implement the LLM/tool loop, approval handling, stream events, and builder validation."
  - path: projects/agentic-workflow/src/agents/analyst.rs
    action: modify
    section: schema
    impl_mode: codegen
    description: "Define AnalystAgentConfig, AnalystAgent, and AnalystAgentBuilder."
  - path: projects/agentic-workflow/src/agents/analyst.rs
    action: modify
    section: state-machine
    impl_mode: hand-written
    description: "Persist sessions, pause on user-input-required tool output, and resume from stored messages."
```
