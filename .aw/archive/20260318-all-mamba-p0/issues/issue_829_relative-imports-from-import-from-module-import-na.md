---
number: 829
title: "Relative imports — from . import, from ..module import name"
state: open
labels: [enhancement, P0, crate:mamba]
group: "module-system"
---

# #829 — Relative imports — from . import, from ..module import name

## Summary

Support relative import syntax used in Python packages:
- `from . import sibling`
- `from .. import parent_module`
- `from ..utils import helper`
- `from .subpkg.mod import Class`

## Why P0

Relative imports are required for any non-trivial Python package. Without this, Mamba cannot compile multi-file projects with package structure.

## Scope

- **Parser**: Handle leading dots in `from` imports
- **Resolver**: Resolve relative paths against current module's package
- **Module system**: Package hierarchy awareness (`__init__.py` handling)
- **Driver**: Multi-file compilation with module graph
