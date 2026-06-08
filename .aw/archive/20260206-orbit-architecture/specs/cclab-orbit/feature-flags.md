---
id: feature-flags
type: spec
title: "Modular Feature Flags"
version: 1
spec_type: utility
spec_group: cclab-orbit
created_at: 2026-02-05T16:14:11.414482+00:00
updated_at: 2026-02-05T16:14:11.414482+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-05T16:14:11.414482+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Modular Feature Flags

## Overview

Implement a comprehensive feature flag system for the orbit crate to allow users to enable/disable functionality based on their needs. Default features preserve current behavior for backwards compatibility. Feature flags control: TLS support, DNS resolution, Unix sockets, subprocess handling, kqueue tuning, and slab allocator.

## Requirements

### R1 - Feature flag definitions

```yaml
id: R1
priority: high
status: draft
```

Define feature flags in Cargo.toml: default (backwards compatible), python, tls, dns, unix-socket, subprocess, kqueue-tuning, slab-allocator

### R2 - Conditional compilation

```yaml
id: R2
priority: high
status: draft
```

Update lib.rs and module files to use #[cfg(feature = ...)] for conditional compilation based on enabled features

### R3 - Dependency gating

```yaml
id: R3
priority: medium
status: draft
```

Gate optional dependencies behind features: rustls/native-tls for tls, trust-dns for dns, mio for kqueue-tuning

### R4 - Default feature set

```yaml
id: R4
priority: high
status: draft
```

Default features must include: python, tls, dns to match current behavior

## Acceptance Criteria

### Scenario: Minimal build

- **WHEN** Build with --no-default-features
- **THEN** Only core event loop functionality is compiled

### Scenario: Full build

- **WHEN** Build with default features
- **THEN** All functionality available, matches current behavior

### Scenario: Selective features

- **WHEN** Build with --features 'python,tls'
- **THEN** Only python bindings and TLS are enabled

### Scenario: Feature combinations

- **WHEN** Enable kqueue-tuning on macOS
- **THEN** Compiles successfully with kqueue-specific code

</spec>
