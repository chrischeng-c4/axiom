---
id: dynamic-tool-schema
type: spec
title: "Dynamic tools/list Schema Based on Session"
version: 1
spec_type: algorithm
created_at: 2026-02-24T02:58:41.349494+00:00
updated_at: 2026-02-24T02:58:41.349494+00:00
main_spec_ref: "crates/cclab-server/src/http_server.rs"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, logic, changes]
codebase_paths:
  - crates/cclab-server/src/http_server.rs
knowledge_refs:
  - cclab/knowledge/40-mcp/http-server.md
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
---

# Dynamic tools/list Schema Based on Session

## Overview
<!-- type: overview lang: markdown -->

When an MCP session has a bound `project_path` from the `X-Cclab-Project`
header, the `tools/list` response dynamically removes `project_path` from each
tool input schema's `required` array. Properly configured clients get cleaner
schemas while unconfigured clients keep backwards-compatible required
`project_path` arguments.

The old root files were:

- `.aw/tech-design/crates/cclab-server/dynamic-tool-schema.md`
- `.aw/tech-design/crates/cclab-server/dynamic-tool-schema`

The canonical TD now lives at
`.aw/tech-design/crates/cclab-server/interfaces/mcp/dynamic-tool-schema.md`.

## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: dynamic-tool-schema-requirements
entry: R1
---
requirementDiagram
    requirement R1 {
        id: R1
        text: Bound sessions remove project_path from required schemas
        risk: high
        verifymethod: test
    }
    requirement R2 {
        id: R2
        text: Unbound sessions return original schemas unchanged
        risk: medium
        verifymethod: test
    }
    requirement R3 {
        id: R3
        text: Bound tool calls inject session project_path when absent
        risk: high
        verifymethod: test
    }
```

### R1: Dynamic Required Field Removal

When `tools/list` is called and the session has a bound `project_path`, the
server iterates over all tool definitions and removes `project_path` from each
tool's `input_schema.required` array before returning the response.

### R2: No-op For Unbound Sessions

When the session has no bound `project_path`, `tools/list` returns the original
schemas unchanged and `project_path` remains required.

### R3: Tool Call Injection

When a session-bound tool call arrives without `project_path` in arguments, the
server injects `session.project_path` into the arguments before dispatching to
the handler.

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: S1
    requirement: R1
    given: Session has bound project_path from header
    when: Client calls tools/list
    then: All tool schemas show project_path as optional by omitting it from required
  - id: S2
    requirement: R2
    given: Session has no bound project_path
    when: Client calls tools/list
    then: All tool schemas keep project_path in required
  - id: S3
    requirement: R3
    given: Session is bound to /my/project and a tool call has no project_path
    when: Server dispatches the tool call
    then: project_path /my/project is injected into arguments
```

## tools/list Schema Decision
<!-- type: logic lang: mermaid -->

```mermaid
---
id: dynamic-tool-schema-decision
entry: start
---
flowchart TB
    start[tools/list request] --> check{Session has project_path?}
    check -- yes --> modify[Remove project_path from required arrays]
    check -- no --> original[Keep original schemas]
    modify --> respond[Return tools/list response]
    original --> respond
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: .aw/tech-design/crates/cclab-server/interfaces/mcp/dynamic-tool-schema.md
    action: MODIFY
    impl_mode: hand-written
    desc: Move dynamic tool schema TD under interfaces/mcp and normalize sections.
  - path: crates/cclab-server/src/http_server.rs
    action: MODIFY
    impl_mode: hand-written
    desc: Adjust tools/list schemas and inject session project_path for bound MCP sessions.
```
