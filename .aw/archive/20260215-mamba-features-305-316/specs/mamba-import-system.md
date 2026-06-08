---
id: mamba-import-system
type: spec
title: "Multi-file Import System (#306)"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-14T09:31:35.605962+00:00
updated_at: 2026-02-14T09:31:35.605962+00:00
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
      title: "Module Import Flow"
history:
  - timestamp: 2026-02-14T09:31:35.605962+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Multi-file Import System (#306)

## Overview

This specification defines the multi-file import system for Mamba. It covers module resolution, path searching, and the caching mechanism (`sys.modules`) to prevent redundant compilation and circular import handling.

## Requirements

### R1 - Module Path Resolution

```yaml
id: R1
priority: high
status: draft
```

Implement a mechanism to find module files based on a search path (PYTHONPATH equivalent).

### R2 - Module Caching

```yaml
id: R2
priority: high
status: draft
```

Use a global cache (sys.modules) to store and reuse already loaded module objects.

### R3 - Circular Import Handling

```yaml
id: R3
priority: high
status: draft
```

Properly handle circular imports by ensuring a module object is created and cached before its body is executed.

### R4 - Import Syntaxes

```yaml
id: R4
priority: high
status: draft
```

Support both `import module` and `from module import name` syntaxes.

## Acceptance Criteria

### Scenario: Successful Module Import

- **GIVEN** Two files a.py and b.py in the same directory.
- **WHEN** 'a.py' contains 'import b'.
- **THEN** Module 'b' should be available in 'a's namespace.

### Scenario: Circular Import Handling

- **GIVEN** Module 'a' imports 'b', and 'b' imports 'a'.
- **WHEN** The modules are imported.
- **THEN** Both modules should be loaded successfully without infinite recursion.

## Diagrams

### Module Import Flow

```mermaid
flowchart TB
    Start(Import Statement Encountered)
    LocateFile[Locate module file on search path]
    CheckCache{Check sys.modules cache} 
    CompileModule[Parse & Compile module]
    CacheModule[Add module to sys.modules cache]
    ReturnModule[Return Module object]
    End(Import Complete)
    Start --> LocateFile
    LocateFile --> CheckCache
    CheckCache -->|Found| ReturnModule
    CheckCache -->|Not Found| CompileModule
    CompileModule --> CacheModule
    CacheModule --> ReturnModule
    ReturnModule --> End
```

</spec>
