---
id: mamba-stdlib-core
type: spec
title: "Minimal Standard Library (#310)"
version: 1
spec_type: utility
created_at: 2026-02-14T09:32:24.552773+00:00
updated_at: 2026-02-14T09:32:24.552773+00:00
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
  - timestamp: 2026-02-14T09:32:24.552773+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Minimal Standard Library (#310)

## Overview

This specification defines the minimal standard library for Mamba, including the core modules sys, os, math, and json. It covers the implementation of these modules as built-in runtime functions and their exposure to Mamba code via the import system.

## Requirements

### R1 - Core 'sys' Module

```yaml
id: R1
priority: high
status: draft
```

Implement the 'sys' module with access to command-line arguments, search path, and version info.

### R2 - Core 'os' Module

```yaml
id: R2
priority: high
status: draft
```

Provide the 'os' module for file system operations and environment variable access.

### R3 - Core 'math' Module

```yaml
id: R3
priority: medium
status: draft
```

Implement the 'math' module with standard mathematical constants and functions.

### R4 - Core 'json' Module

```yaml
id: R4
priority: medium
status: draft
```

Provide the 'json' module for encoding and decoding Mamba objects to/from JSON strings.

## Acceptance Criteria

### Scenario: Access sys.argv

- **GIVEN** A script using 'import sys'.
- **WHEN** sys.argv[1] is accessed.
- **THEN** The script should correctly receive and print the arguments.

### Scenario: JSON Serialization

- **GIVEN** An object to be serialized.
- **WHEN** json.dumps(obj) is called.
- **THEN** The runtime should produce a valid JSON string.

</spec>
