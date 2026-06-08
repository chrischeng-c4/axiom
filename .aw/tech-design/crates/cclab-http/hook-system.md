---
id: hook-system
type: spec
title: "Lifecycle Hook System"
version: 1
spec_type: algorithm
created_at: 2026-01-28T08:03:04.287392+00:00
updated_at: 2026-01-28T08:03:04.287392+00:00
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
  diagrams:
    - type: flowchart
      title: "Lifecycle Hook Execution Flow"
history:
  - timestamp: 2026-01-28T08:03:04.287392+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Lifecycle Hook System

## Overview

This specification defines a hook system for cclab-titan, allowing users to register callbacks for various lifecycle events such as before/after insert, update, and delete. This is essential for auditing, automatic timestamping, and other cross-cutting concerns.

## Requirements

### R1 - Lifecycle Events

```yaml
id: R1
priority: medium
status: draft
```

Define a set of lifecycle events: before_insert, after_insert, before_update, after_update, before_delete, after_delete.

### R2 - Hook Registration

```yaml
id: R2
priority: medium
status: draft
```

Provide a mechanism to register hook functions for specific tables or globally.

### R3 - Hook Execution Order

```yaml
id: R3
priority: medium
status: draft
```

Ensure hooks are executed in the correct order relative to database operations.

### R4 - Async Hook Support

```yaml
id: R4
priority: medium
status: draft
```

Support both sync and async hook functions.

## Acceptance Criteria

### Scenario: Before Insert Hook Logic

- **GIVEN** A registered before_insert hook for the 'users' table that sets a created_at timestamp
- **WHEN** A new User is added to the session and flush() is called
- **THEN** The hook is executed, and the generated INSERT statement includes the timestamp set by the hook.

### Scenario: After Update Hook Logic

- **GIVEN** An after_update hook that logs changes to an audit table
- **WHEN** A User is updated and session.commit() is called
- **THEN** The hook is executed after the UPDATE statement succeeds, and audit records are created.

## Diagrams

### Lifecycle Hook Execution Flow

```mermaid
flowchart TB
    Session_flush[Session.flush()]
    BeforeFlushHook[Before Flush Hooks]
    ExecuteSQL[Execute SQL Statements]
    AfterFlushHook[After Flush Hooks]
    Session_flush --> BeforeFlushHook
    BeforeFlushHook --> ExecuteSQL
    ExecuteSQL --> AfterFlushHook
```

</spec>
