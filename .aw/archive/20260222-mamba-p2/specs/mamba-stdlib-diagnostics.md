---
id: mamba-stdlib-diagnostics
type: spec
title: "Stdlib: traceback, warnings, inspect"
version: 1
spec_type: utility
created_at: 2026-02-22T11:21:23.458541+00:00
updated_at: 2026-02-22T11:21:23.458541+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-22T11:21:23.458541+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Stdlib: traceback, warnings, inspect

## Overview

Implement three diagnostic/introspection stdlib modules for Mamba: traceback (exception stack trace formatting), warnings (warning system with category filters), and inspect (runtime introspection of classes, functions, and closures).

## Requirements

### R1 - traceback module

```yaml
id: R1
priority: medium
status: draft
```

Create traceback_mod.rs. mb_traceback_format_exc() returns string of current exception traceback. mb_traceback_print_exc() prints to stderr. mb_traceback_format_exception(exc) formats an exception value. Uses exception.rs stack frame info.

### R2 - warnings module

```yaml
id: R2
priority: medium
status: draft
```

Create warnings_mod.rs. mb_warnings_warn(message, category) issues a warning. mb_warnings_filterwarnings(action, category) controls warning display. Thread-local filter stack. Categories: UserWarning, DeprecationWarning, FutureWarning. Default: print to stderr.

### R3 - inspect module

```yaml
id: R3
priority: medium
status: draft
```

Create inspect_mod.rs. mb_inspect_isfunction(obj), mb_inspect_isclass(obj), mb_inspect_ismethod(obj): type checks. mb_inspect_getmembers(obj): return list of (name, value) pairs from object attributes. mb_inspect_signature(func): return parameter info.

## Acceptance Criteria

### Scenario: format_exc returns traceback string

- **GIVEN** exception was raised
- **WHEN** traceback.format_exc()
- **THEN** Returns string containing exception type and message

### Scenario: warn prints to stderr

- **WHEN** warnings.warn('deprecated', DeprecationWarning)
- **THEN** DeprecationWarning: deprecated printed to stderr

### Scenario: isfunction detects functions

- **WHEN** inspect.isfunction(some_func)
- **THEN** Returns True for function values

</spec>
