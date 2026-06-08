---
id: agent-protocols-spec
main_spec_ref: "agent/interfaces/agent/agent-protocols-spec.md"
fill_sections: [overview, schema, changes]
---

# Agent Protocols Spec

## Overview
<!-- type: overview lang: markdown -->

The agent protocol layer defines the stable request, response, and event
shapes exchanged between `agent` runners, stream handlers, and tool
adapters. It keeps protocol DTOs separate from concrete model providers so
Claude, OpenAI, Gemini, and local agents can share the same outer contract.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  AgentRunRequest:
    type: object
    required: [input]
    properties:
      input: {type: string}
      session_id: {type: string}
      metadata:
        type: object
        additionalProperties: true

  AgentRunResult:
    type: object
    required: [output, events]
    properties:
      output: {type: string}
      events:
        type: array
        items:
          $ref: "#/definitions/AgentEvent"

  AgentEvent:
    type: object
    required: [kind, payload]
    properties:
      kind:
        type: string
        enum: [started, token, tool_call, tool_result, completed, failed]
      payload:
        type: object
        additionalProperties: true
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agent/core/src/protocols.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Define shared AgentRunRequest, AgentRunResult, and AgentEvent DTOs for runner/provider boundaries."
```
