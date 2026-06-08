---
id: cclab-nova-core
type: spec
title: "cclab-nova-core Specification"
version: 1
spec_type: algorithm
created_at: 2026-01-28T08:40:29.578975+00:00
updated_at: 2026-01-28T08:40:29.578975+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-28T08:40:29.578975+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# cclab-nova-core Specification

## Overview

Core agent abstractions, context management, and execution engine. This spec defines the fundamental building blocks for agents, including state management, conversation history, and the execution loop with tool support and structured output validation.

## Requirements

### R1 - Structured Output Validation

```yaml
id: R1
priority: high
status: draft
```

Integrate cclab-shield for schema-validated LLM responses. Support defining expected output models.

### R2 - RunContext Dependency Injection

```yaml
id: R2
priority: high
status: draft
```

Enhance AgentContext to support a generic RunContext for dependency injection. Allow passing resources to tools and agents.

### R3 - Copy-on-Write State Management

```yaml
id: R3
priority: medium
status: draft
```

Ensure SharedState uses Arc correctly for GIL-free state management, optimized for Python integration.

### R4 - Agent Execution Loop

```yaml
id: R4
priority: high
status: draft
```

Implement a robust execution loop with configurable retries, timeouts, and error handling.

## Acceptance Criteria

### Scenario: Successful execution with structured output

- **GIVEN** An agent with a defined output schema (cclab-shield model).
- **WHEN** The agent finishes its task and returns a JSON response.
- **THEN** The final response is validated against the schema and returned as a structured object.

### Scenario: Max turns reached error

- **GIVEN** An agent with a system prompt and a max_turns limit.
- **WHEN** The agent conversation exceeds the max_turns limit.
- **THEN** An AgentError::MaxTurnsReached is returned, and execution stops.

### Scenario: Dependency injection in tool execution

- **GIVEN** A RunContext containing a database connection.
- **WHEN** A tool is executed that requires access to a database.
- **THEN** The tool successfully accesses the database connection from the context.

## Flow Diagram

```mermaid
flowchart TB
    Start((Start Execution))
    InputMsg[Receive Input Message]
    AddContext[Add to AgentContext history]
    CheckTurns[Check Max Turns]
    LLMCall[Call LLM Provider]
    ProcessResponse[Process LLM Response]
    ExecuteTools[[Execute Requested Tools]]
    AddResultsToContext[Add Tool Results to Context]
    ValidateOutput[Validate Output with cclab-shield]
    ErrorMsg[Return Max Turns Error]
    RetryOrError[Retry or Return Validation Error]
    End((Return Final AgentResponse))

    Start --> InputMsg
    InputMsg --> AddContext
    AddContext --> CheckTurns
    CheckTurns -- "Turn limit exceeded" --> ErrorMsg
    CheckTurns -- "Turn limit OK" --> LLMCall
    LLMCall --> ProcessResponse
    ProcessResponse -- "Tool call requested" --> ExecuteTools
    ExecuteTools --> AddResultsToContext
    AddResultsToContext --> CheckTurns
    ProcessResponse -- "No tool call (Final response)" --> ValidateOutput
    ValidateOutput -- "Invalid output schema" --> RetryOrError
    ValidateOutput -- "Valid output schema" --> End
```

</spec>
