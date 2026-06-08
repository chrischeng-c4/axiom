---
number: 841
title: "Multi-file compilation — module graph and project compilation"
state: open
labels: [enhancement, P0, crate:mamba]
group: "module-system"
---

# #841 — Multi-file compilation — module graph and project compilation

## Summary

Support compiling multi-file Python projects with proper module resolution. Currently Mamba operates primarily on single files.

## Scope

- **Module graph**: Build dependency graph from import statements
- **Package structure**: `__init__.py` / `__init__.mamba` recognition
- **Compilation order**: Topological sort of module dependencies
- **Cross-module type checking**: Type information flows across module boundaries
- **Separate compilation**: Each module compiled independently, linked at the end
- **Entry point**: `mamba run src/main.py` resolves all imports transitively

## Why P0

Any real Python project has multiple files. Without multi-file support, Mamba is limited to scripts and single-file demos. This is a prerequisite for the package manager (#751) and real-world adoption.
