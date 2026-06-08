---
number: 421
title: "mamba: module/package system (__init__.py, sys.path, relative imports)"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #421 — mamba: module/package system (__init__.py, sys.path, relative imports)

## Description

Implement a real file-based module/package system. Currently `import` is parsed but not fully resolved at runtime.

## Requirements

### Module resolution
- R1: `import foo` — search sys.path for `foo.py` or `foo/__init__.py`
- R2: `from foo import bar` — import specific names
- R3: `from foo import *` — import all public names (respecting `__all__`)
- R4: Relative imports: `from . import sibling`, `from ..parent import thing`

### Package support
- R5: `__init__.py` recognition — directory with `__init__.py` is a package
- R6: Nested packages: `import foo.bar.baz`
- R7: `__name__`, `__file__`, `__package__` module attributes
- R8: `__all__` enforcement for `import *`

### Module caching
- R9: `sys.modules` — cache imported modules, prevent re-execution
- R10: `sys.path` — configurable search paths
- R11: Circular import detection

### importlib (basic)
- R12: `importlib.import_module(name)` — programmatic import

## Priority

P1 — without this, any multi-file Python program cannot run.
