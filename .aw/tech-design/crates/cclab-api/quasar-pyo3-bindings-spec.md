---
id: quasar-pyo3-bindings-spec
type: spec
title: "Quasar PyO3 Bindings Specification"
version: 1
spec_type: integration
created_at: 2026-02-01T10:30:29.366801+00:00
updated_at: 2026-02-01T10:30:29.366801+00:00
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
history:
  - timestamp: 2026-02-01T10:30:29.366801+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Quasar PyO3 Bindings Specification

## Overview

This specification covers the reorganization of PyO3 bindings in cclab-quasar, the extraction of data conversion logic, and the implementation of a comprehensive testing strategy for the Rust-Python bridge. It ensures a clean separation between core Rust logic and Python integration while addressing existing test failures.

## Requirements

### R1 - Module Reorganization

```yaml
id: R1
priority: medium
status: draft
```

All PyO3-related code must be consolidated into a new src/pyo3_bindings/ module to improve maintainability and discoverability.

### R2 - Data Conversions Extraction

```yaml
id: R2
priority: medium
status: draft
```

Rust-to-Python and Python-to-Rust data conversion logic must be extracted into a dedicated src/pyo3_bindings/conversions.rs module.

### R3 - Comprehensive Type Support

```yaml
id: R3
priority: medium
status: draft
```

The conversion logic must support Request, Response, and WebSocket message types, including complex nested structures.

### R4 - Test Strategy Reorganization

```yaml
id: R4
priority: medium
status: draft
```

PyO3 integration tests must be moved to a standalone crates/cclab-quasar/tests/ directory to ensure clean separation and proper Python environment isolation.

### R5 - Fix Existing Failures

```yaml
id: R5
priority: medium
status: draft
```

Fix the 4 currently failing PyO3 tests in python_handler.rs by implementing proper pyo3::prepare_freethreaded_python() initialization.

## Acceptance Criteria

### Scenario: Successful Python Handler Execution

- **WHEN** A request is dispatched to a PythonHandler wrapped callable.
- **THEN** The response should match the data returned from the Python callable with correct status codes and headers.

### Scenario: Data Conversion Fidelity

- **WHEN** A Rust SerializableRequest is converted to a Python dict.
- **THEN** The Python dict should contain all expected fields (method, path, headers, body) with correct types.

### Scenario: Test Initialization Fix

- **WHEN** Running cargo test with the new pyo3::prepare_freethreaded_python() logic.
- **THEN** All tests in the python_handler.rs (and their successors) should pass without initialization errors.

## Flow Diagram

```mermaid
sequenceDiagram
    participant Server as Quasar Server
    participant Handler as PythonHandler
    participant Conv as Conversions Module
    participant Loop as PyLoop (Orbit)

    Server->>Handler: execute(req)
    activate Handler
    Handler->>Conv: convert_request_to_py(req)
    activate Conv
    Conv-->>Handler: PyDict
    deactivate Conv
    Handler->>Loop: spawn_python_handler(callable, py_req)
    Loop-->>Handler: PyObject (Result)
    Handler->>Conv: convert_py_to_response(result)
    activate Conv
    Conv-->>Handler: Response
    deactivate Conv
    Handler-->>Server: Response
    deactivate Handler
```

</spec>
