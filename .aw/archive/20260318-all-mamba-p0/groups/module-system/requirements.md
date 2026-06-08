---
change: all-mamba-p0
group: module-system
date: 2026-03-18
---

# Requirements

Implement complete module system: import aliases (import X as Y, from X import Y as Z), relative imports (from . import, from ..mod import name), and multi-file compilation with module graph, package structure (__init__.py), topological compilation order, cross-module type checking, and separate compilation with linking. These three features are tightly coupled — relative imports and aliases are prerequisites for multi-file compilation, and multi-file compilation provides the driver that resolves imports.
