---
id: quasar-pyo3-expansion
type: spec
title: "Expand cclab-quasar PyO3 Exports for FastAPI Parity"
version: 1
spec_type: utility
created_at: 2026-02-24T10:43:09.472101+00:00
updated_at: 2026-02-24T10:43:09.472101+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Quasar Expansion Architecture"
history:
  - timestamp: 2026-02-24T10:43:09.472101+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Expand cclab-quasar PyO3 Exports for FastAPI Parity

## Overview

Expand cclab-quasar (API) PyO3 bindings to achieve feature parity with FastAPI. This expansion adds essential plumbing for middleware registration, WebSocket handler integration, and router grouping, enabling more complex application architectures to be built and served directly from Python.

## Requirements

### R1 - Middleware Registration

```yaml
id: R1
priority: high
status: draft
```

Implement add_middleware in PyApiApp to allow registering Python callables as middleware that wrap the request/response cycle.

### R2 - WebSocket Routing

```yaml
id: R2
priority: high
status: draft
```

Implement register_websocket_route in PyApiApp to register Python async handlers for WebSocket connections.

### R3 - Router Composition

```yaml
id: R3
priority: medium
status: draft
```

Implement include_router in PyApiApp to allow merging multiple router instances with optional path prefixes.

### R4 - WebSocket API Parity

```yaml
id: R4
priority: high
status: draft
```

Ensure PyWebSocket properly exposes receive(), send_text(), send_bytes(), and close() methods to Python handlers.

### R5 - Server Configuration Expansion

```yaml
id: R5
priority: medium
status: draft
```

Update PyApiApp.serve() to accept WebSocket configuration parameters like timeouts and message size limits.

## Acceptance Criteria

### Scenario: Middleware Execution

- **GIVEN** An app with a middleware that adds a 'X-Middleware-Seen' header.
- **WHEN** A request is processed through the registered middleware chain.
- **THEN** The response received by the client must contain the 'X-Middleware-Seen' header.

### Scenario: WebSocket Bidirectional Communication

- **GIVEN** A registered WebSocket route '/ws/echo'.
- **WHEN** A client connects and sends a text message.
- **THEN** The Python handler must successfully receive the message and send an 'Echo: {msg}' response back to the client.

### Scenario: Prefix-based Routing

- **GIVEN** Two routers with prefixes '/v1' and '/v2'.
- **WHEN** The routers are included in the main ApiApp and requests are dispatched.
- **THEN** Requests to both '/v1/users' and '/v2/users' must be correctly routed to their respective handlers.

## Diagrams

### Quasar Expansion Architecture

```mermaid
flowchart TB
    python_app[Python Application]
    py_api_app[ApiApp (PyO3)]
    middleware_chain[Middleware Chain]
    websocket_routes[WebSocket Routes]
    sub_routers[Sub-Routers (Prefixes)]
    py_websocket[PyWebSocket Wrapper]
    python_app -->|uses| py_api_app
    py_api_app -->|registers| middleware_chain
    py_api_app -->|registers| websocket_routes
    py_api_app -->|includes| sub_routers
    websocket_routes -->|manages| py_websocket
```

</spec>
