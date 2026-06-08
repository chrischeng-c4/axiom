---
number: 1132
title: "mamba: compiler import resolution does not wire MAMBA_MODULES registry symbols"
state: open
labels: [type:bug, priority:p1, crate:mamba]
group: "wire-mamba-modules"
---

# #1132 — mamba: compiler import resolution does not wire MAMBA_MODULES registry symbols

## Problem

When mamba scripts import from native modules (`from cclab.api import Router`, `from cclab.log import get_logger`), all imported names resolve to `0.0` (the default float value). Calling these functions returns `None`.

## Root cause

The mamba compiler's import resolution and codegen do not integrate with the `MAMBA_MODULES` distributed slice / `RuntimeSymbol` registry. The infrastructure exists on both sides:

- **Registry side**: `cclab-mamba-registry` provides `MAMBA_MODULES`, `RuntimeSymbol`, `ModuleRegistrar` — all functional
- **FFI side**: `cclab-api-mamba`, `cclab-runtime-mamba`, `cclab-log-mamba` etc. register symbols with `rt_sym!()` and implement `extern "C"` FFI functions — all functional
- **Compiler side**: parser/typechecker/JIT codegen works for basic Python — functional

But the compiler's import resolver never calls `find_module()` / iterates `MAMBA_MODULES` to look up `RuntimeSymbol` entries, so native symbols are unreachable from mamba scripts.

## Reproduction

```python
from cclab.log import get_logger
print(get_logger)  # prints: 0.0
logger = get_logger("test")
print(logger)  # prints: None
```

```bash
cclab mamba run test.py
```

## Expected

`get_logger` should resolve to the `mb_log_get_logger` FFI function pointer and calling it should invoke the Rust implementation.

## Additional note

There's also a Python-name → FFI-name mismatch: the registry uses internal names like `mb_api_router_new` while mamba.toml expose lists use Python names like `Router`. The import resolver needs to handle this mapping (either via the expose table in mamba.toml or via aliases in the registry).

## Context

Discovered while attempting to run the Conductor backend with `cclab mamba run`. This is the primary blocker for mamba-powered project execution.
