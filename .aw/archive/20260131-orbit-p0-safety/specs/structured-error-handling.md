---
id: structured-error-handling
type: spec
title: "Structured Error Handling"
version: 1
spec_type: algorithm
created_at: 2026-01-31T10:51:14.980697+00:00
updated_at: 2026-01-31T10:51:14.980697+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: true
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Error Classification Flow"
history:
  - timestamp: 2026-01-31T10:51:14.980697+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Structured Error Handling

## Overview

This specification defines a structured error handling strategy for cclab-orbit using the thiserror crate. It replaces manual io::Error constructions with domain-specific error enums to improve observability and robustness.

## Requirements

### R1 - Consolidated Error Enum

```yaml
id: R1
priority: medium
status: draft
```

Expand the PyLoopError enum in crates/cclab-orbit/src/error.rs using thiserror to include specific variants for Network, DNS, Subprocess, and Timer domains.

### R2 - Error Refactoring

```yaml
id: R2
priority: medium
status: draft
```

Replace all instances of manual io::Error::new(Other, ...) with specific PyLoopError variants across the codebase.

### R3 - Rich Error Context

```yaml
id: R3
priority: medium
status: draft
```

Ensure all error variants provide clear, actionable messages and correctly wrap underlying OS errors where applicable.

### R4 - Error Conversions

```yaml
id: R4
priority: medium
status: draft
```

Implement Conversion traits (From<io::Error>, etc.) to allow seamless error propagation with the ? operator.

## Acceptance Criteria

### Scenario: Structured Network Error

- **GIVEN** A network connection fails.
- **WHEN** The operation fails.
- **THEN** The error is returned as PyLoopError::Network(NetworkError::ConnectionRefused).

### Scenario: Structured DNS Error

- **GIVEN** A hostname cannot be resolved.
- **WHEN** DNS lookup fails.
- **THEN** The error is returned as PyLoopError::Dns(DnsError::ResolutionFailed).

### Scenario: Wrapped IO Error

- **GIVEN** An OS error occurs.
- **WHEN** A file operation fails.
- **THEN** The OS error is wrapped in PyLoopError::Io and includes the original context.

### Scenario: Structured Subprocess Error

- **GIVEN** A subprocess fails to start.
- **WHEN** Spawning a child process.
- **THEN** The error is returned as PyLoopError::Subprocess(SubprocessError::ProcessNotFound).

## Diagrams

### Error Classification Flow

```mermaid
flowchart LR
    InputError(Manual io::Error or OS Error)
    Classifier{Error Classifier} 
    NetworkVariant[PyLoopError::Network]
    DnsVariant[PyLoopError::Dns]
    SubprocessVariant[PyLoopError::Subprocess]
    IoVariant[PyLoopError::Io]
    InputError --> Classifier
    Classifier -->|Network IO?| NetworkVariant
    Classifier -->|DNS Resolution?| DnsVariant
    Classifier -->|Subprocess?| SubprocessVariant
    Classifier -->|Generic IO?| IoVariant
```

## Data Model

```json
{
  "properties": {
    "code": {
      "description": "Error code string",
      "type": "string"
    },
    "message": {
      "description": "Human-readable error message",
      "type": "string"
    },
    "source": {
      "description": "Underlying error source (optional)",
      "type": "string"
    }
  },
  "required": [
    "code",
    "message"
  ],
  "type": "object"
}
```

</spec>
