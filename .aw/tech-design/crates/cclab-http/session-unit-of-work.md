---
id: session-unit-of-work
type: spec
title: "Session and Unit of Work"
version: 1
spec_type: algorithm
created_at: 2026-01-28T08:02:38.351301+00:00
updated_at: 2026-01-28T08:02:38.351301+00:00
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
      title: "Session and Unit of Work Components"
history:
  - timestamp: 2026-01-28T08:02:38.351301+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Session and Unit of Work

## Overview

This specification defines the Rust implementation of the Session and Unit of Work patterns. It enables identity consistency (only one instance per row) and efficient change tracking (dirty tracking) within a transactional context.

## Requirements

### R1 - Identity Map

```yaml
id: R1
priority: medium
status: draft
```

Implement an IdentityMap using weak references to cache loaded objects by table name and primary key.

### R2 - Dirty Tracking

```yaml
id: R2
priority: medium
status: draft
```

Implement a DirtyTracker that snapshots object state upon loading and detects modifications by comparing current state with snapshots.

### R3 - Unit of Work Logic

```yaml
id: R3
priority: medium
status: draft
```

Implement a UnitOfWork that accumulates new, modified, and deleted objects and executes them in a single batch during flush/commit.

### R4 - Session API

```yaml
id: R4
priority: medium
status: draft
```

Provide a Session struct that provides a unified API for managing object lifecycle and transactional boundaries.

## Acceptance Criteria

### Scenario: Identity Consistency Case

- **GIVEN** An active Session
- **WHEN** Two separate calls to session.get(User, 1) are made
- **THEN** Both calls return the same memory address (same object instance).

### Scenario: Dirty Tracking and Flush Logic

- **GIVEN** A persistent User object in a Session
- **WHEN** user.name is changed and session.flush() is called
- **THEN** Session detects the change and generates an UPDATE statement during flush.

## Diagrams

### Session and Unit of Work Components

```mermaid
flowchart LR
    Session[Session]
    IdentityMap[IdentityMap (Weak Refs)]
    UnitOfWork[UnitOfWork (Batcher)]
    DirtyTracker[DirtyTracker (Snapshotting)]
    DatabasePool[DatabasePool]
    Session --> IdentityMap
    Session --> UnitOfWork
    UnitOfWork --> DirtyTracker
    Session --> DatabasePool
```

</spec>
