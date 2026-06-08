---
id: orbit-zero-copy-apis
type: spec
title: "Orbit Zero-Copy APIs"
version: 1
spec_type: algorithm
created_at: 2026-01-28T07:19:53.110264+00:00
updated_at: 2026-01-28T07:19:53.110264+00:00
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
      title: "Zero-Copy sendfile Flow"
history:
  - timestamp: 2026-01-28T07:19:53.110264+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Orbit Zero-Copy APIs

## Overview

This specification covers the implementation of Zero-Copy networking APIs in cclab-orbit, leveraging OS-level optimizations like sendfile and splice to achieve maximum throughput for file-to-socket transfers.

## Requirements

### R1 - sendfile Support

```yaml
id: R1
priority: high
status: draft
```

Implement sendfile support in TcpTransport for Linux and Unix.

### R2 - High-level File Transfer API

```yaml
id: R2
priority: high
status: draft
```

Expose a high-level API for efficient file transfers from Python.

### R3 - Zero-Copy Fallback Mechanism

```yaml
id: R3
priority: medium
status: draft
```

Fallback to standard read/write loop if Zero-Copy is not supported by the OS or file system.

### R4 - AsyncFile Integration

```yaml
id: R4
priority: medium
status: draft
```

Integrate with AsyncFile for seamless file-to-socket streaming.

## Acceptance Criteria

### Scenario: Large File Transfer

- **WHEN** A Python application uses transport.sendfile(file_handle) to send a 1GB file over TCP.
- **THEN** The file is transferred using sendfile syscall, resulting in significantly lower CPU usage.

### Scenario: Zero-Copy Fallback

- **WHEN** A Python application attempts sendfile on a platform that does not support it.
- **THEN** The transport successfully transfers the file using standard async read/write.

### Scenario: Zero-Copy with TLS

- **WHEN** A user attempts to use sendfile on a TlsTransport.
- **THEN** The system falls back to standard write since TLS requires encryption in user-space.

## Diagrams

### Zero-Copy sendfile Flow

```mermaid
flowchart TB
    App[Python Application]
    Transport[ZeroCopyTransport (Rust)]
    OS_Kernel[OS Kernel (sendfile/splice)]
    Network_Interface[Network Interface Card]
    App -->|sendfile(file, socket)| Transport
    Transport -->|syscall sendfile| OS_Kernel
    OS_Kernel -->|Direct DMA Transfer| Network_Interface
```

</spec>
