---
id: quasar-test-client
type: spec
title: "Quasar Test Client Spec"
version: 1
spec_type: utility
created_at: 2026-01-28T17:22:48.170007+00:00
updated_at: 2026-01-28T17:22:48.170007+00:00
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
  - timestamp: 2026-01-28T17:22:48.170007+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Quasar Test Client Spec

## Overview

This specification covers the TestClient for Quasar, which allows for high-speed, in-process integration testing. It enables developers to dispatch mock HTTP requests to a Router instance and receive responses without the overhead of binding to a real TCP port.

## Requirements

### R1 - Implement Test Client in crates/cclab-quasar/src/testing.rs

```yaml
id: R1
priority: high
status: draft
```

Implement a TestClient struct in 'crates/cclab-quasar/src/testing.rs' that can wrap a Router and dispatch mock requests.

### R2 - Sync/Async Support in crates/cclab-quasar/src/testing.rs

```yaml
id: R2
priority: medium
status: draft
```

Support both synchronous and asynchronous request dispatching to accommodate different testing styles.

## Acceptance Criteria

### Scenario: Mock GET Request

- **GIVEN** A Router with a registered GET route "/hello"
- **WHEN** TestClient::get("/hello") is called.
- **THEN** The TestClient returns the response from the "/hello" handler with status 200.

### Scenario: Mock POST Request with Body

- **GIVEN** A Router with a POST route "/data" expecting JSON body
- **WHEN** TestClient::post("/data", body) is called.
- **THEN** The TestClient correctly serializes the body and the handler receives it.

### Scenario: Async Request Dispatch

- **GIVEN** An asynchronous test environment using tokio
- **WHEN** await test_client.get_async("/hello") is called.
- **THEN** The test client correctly awaits the handler response without blocking the executor.

</spec>
