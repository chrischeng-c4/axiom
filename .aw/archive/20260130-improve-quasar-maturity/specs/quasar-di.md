---
id: quasar-di
type: spec
title: "Quasar Dependency Injection Spec"
version: 1
spec_type: utility
created_at: 2026-01-28T17:23:28.122270+00:00
updated_at: 2026-01-28T17:23:28.122270+00:00
requirements:
  total: 2
  ids:
    - R1
    - R2
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-28T17:23:28.122270+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Quasar Dependency Injection Spec

## Overview

This specification covers the automated Dependency Injection system for Quasar handlers. It enables route handlers to declare dependencies via FastAPI-style 'Depends' markers, which are then topologically resolved and injected by the framework during request processing.

## Requirements

### R1 - Extend DependencyResolver in crates/cclab-quasar/src/dependency.rs

```yaml
id: R1
priority: high
status: draft
```

Extend the DependencyResolver in 'crates/cclab-quasar/src/dependency.rs' to support automated, graph-based resolution of dependencies from handler signatures.

### R2 - Update Handler and Router in crates/cclab-quasar/src/handler.rs and router.rs

```yaml
id: R2
priority: high
status: draft
```

Update the HandlerFn trait and Router in 'crates/cclab-quasar/src/handler.rs' and 'crates/cclab-quasar/src/router.rs' to integrated with the resolver and inject dependencies into handler calls.

## Acceptance Criteria

### Scenario: Resolve Singleton Dependency

- **GIVEN** A dependency graph with singletons and scoped dependencies
- **WHEN** A request is dispatched to a handler with multiple Depends markers.
- **THEN** The handler receives the correct instance and singletons are reused across requests.

### Scenario: Resolve Scoped Dependency

- **GIVEN** A dependency container with a request-scoped dependency
- **WHEN** Two handlers in the same request context require the same scoped dependency.
- **THEN** A new instance is created for each request and shared within that request context.

### Scenario: Circular Dependency Error

- **GIVEN** Two dependencies that depend on each other
- **WHEN** The DI resolver attempts to resolve a circular dependency graph.
- **THEN** The DI resolver returns an informative error during graph resolution.

</spec>
