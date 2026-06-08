---
id: pipe-abstraction
type: spec
title: "Cross-Platform Pipe Abstraction"
version: 1
spec_type: integration
created_at: 2026-02-05T08:53:13.864345+00:00
updated_at: 2026-02-05T08:53:13.864345+00:00
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
  - timestamp: 2026-02-05T08:53:13.864345+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Cross-Platform Pipe Abstraction

## Overview

Provide a unified cross-platform API for named pipes that abstracts over Unix FIFO and Windows named pipes. The PipeTransport trait defines the common interface, with platform-specific implementations handling the differences. Supports both server (listener) and client (connector) patterns.

## Requirements

### R1 - PipeTransport Trait

```yaml
id: R1
priority: high
status: draft
```

Define PipeTransport trait with async read(), write(), flush(), and close() methods that work across platforms.

### R2 - PipeListener

```yaml
id: R2
priority: high
status: draft
```

Create PipeListener that accepts incoming connections (Unix: open for read, Windows: ConnectNamedPipe).

### R3 - PipeConnector

```yaml
id: R3
priority: high
status: draft
```

Create PipeConnector that connects to existing pipes (Unix: open, Windows: CreateFile).

### R4 - PipeConfig

```yaml
id: R4
priority: medium
status: draft
```

Unified configuration struct for buffer sizes, timeouts, and platform-specific options with sensible defaults.

### R5 - Feature Gates

```yaml
id: R5
priority: medium
status: draft
```

Use cfg attributes to compile only platform-relevant code. Provide compile-time errors for unsupported platforms.

## Acceptance Criteria

### Scenario: Create pipe server cross-platform

- **GIVEN** Running on any supported platform
- **WHEN** Call PipeListener::bind(path)
- **THEN** Returns listener ready to accept connections

### Scenario: Connect to pipe cross-platform

- **GIVEN** Pipe server is listening
- **WHEN** Call PipeConnector::connect(path)
- **THEN** Returns connected PipeTransport

### Scenario: Transparent read/write

- **GIVEN** Connected pipe transport
- **WHEN** Call transport.write() and transport.read()
- **THEN** Data flows correctly regardless of platform

### Scenario: Graceful shutdown

- **GIVEN** Active pipe connection
- **WHEN** Call transport.close()
- **THEN** Resources cleaned up on both platforms

## Flow Diagram

```mermaid
sequenceDiagram
    title Cross-Platform Pipe API
    actor app as Application
    participant api as PipeTransport API
    participant unix as Unix Backend
    participant win as Windows Backend
    app->>api: PipeListener::bind(path)
    alt Unix
        api->>unix: mkfifo + open
        unix-->>api: UnixPipeTransport
    else Windows
        api->>win: CreateNamedPipeW
        win-->>api: WindowsPipeTransport
    end
    api-->>app: PipeListener
    app->>api: listener.accept()
    api-->>app: PipeTransport (unified)
```

</spec>
