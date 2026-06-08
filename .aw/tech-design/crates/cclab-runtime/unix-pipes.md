---
id: unix-pipes
type: spec
title: "Unix Named Pipes (FIFO) Support"
version: 1
spec_type: integration
created_at: 2026-02-05T08:52:47.811971+00:00
updated_at: 2026-02-05T08:52:47.811971+00:00
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
  - timestamp: 2026-02-05T08:52:47.811971+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Unix Named Pipes (FIFO) Support

## Overview

Implement Unix FIFO (named pipe) support for inter-process communication. This module provides async read/write operations on named pipes created with mkfifo, integrating with tokio's async I/O. Supports both reader and writer modes with proper cleanup on drop.

## Requirements

### R1 - FIFO Creation

```yaml
id: R1
priority: high
status: draft
```

Create named pipes using mkfifo syscall with configurable permissions (mode). Handle EEXIST gracefully for idempotent creation.

### R2 - Async Read/Write

```yaml
id: R2
priority: high
status: draft
```

Provide async read and write operations using tokio::fs::File or tokio::net::unix for non-blocking I/O on FIFO file descriptors.

### R3 - Reader/Writer Modes

```yaml
id: R3
priority: high
status: draft
```

Support opening FIFO in read-only, write-only, or read-write modes. Handle blocking semantics (open blocks until other end connects).

### R4 - Cleanup on Drop

```yaml
id: R4
priority: medium
status: draft
```

Optionally remove the FIFO file on drop (owner mode). Implement Drop trait for automatic cleanup.

### R5 - Error Handling

```yaml
id: R5
priority: medium
status: draft
```

Map Unix errno to descriptive Rust errors (ENXIO for no reader, EPIPE for broken pipe, etc.).

## Acceptance Criteria

### Scenario: Create and write to FIFO

- **GIVEN** No FIFO exists at /tmp/test.fifo
- **WHEN** Create FIFO with mode 0o644 and open for writing
- **THEN** FIFO file exists and writer handle is returned

### Scenario: Read from FIFO

- **GIVEN** FIFO exists and writer is connected
- **WHEN** Open FIFO for reading and call read()
- **THEN** Data written by writer is received

### Scenario: Handle missing reader

- **GIVEN** FIFO exists but no reader is connected
- **WHEN** Open for write with O_NONBLOCK
- **THEN** Returns ENXIO error (no such device)

### Scenario: Cleanup on drop

- **GIVEN** FIFO created with cleanup=true
- **WHEN** FifoHandle is dropped
- **THEN** FIFO file is removed from filesystem

## Flow Diagram

```mermaid
sequenceDiagram
    title FIFO Read/Write Flow
    actor writer as Writer Process
    participant fifo as FIFO
    actor reader as Reader Process
    writer->>fifo: mkfifo(/tmp/pipe)
    writer->>+fifo: open(O_WRONLY)
    reader->>+fifo: open(O_RDONLY)
    writer->>fifo: write(data)
    fifo-->>reader: read() -> data
    writer->>-fifo: close()
    fifo-->>-reader: read() -> EOF
```

</spec>
