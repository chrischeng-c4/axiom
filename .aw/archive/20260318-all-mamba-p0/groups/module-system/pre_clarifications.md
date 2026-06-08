---
change: all-mamba-p0
group: module-system
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: Module resolution
- **Question**: Should module resolution follow CPython's exact algorithm (sys.path, __init__.py, namespace packages) or a simplified subset for v1?
- **Answer**: Full CPython algorithm — support sys.path, __init__.py, namespace packages, finder/loader protocol, importlib hooks.

### Q2: Compilation model
- **Question**: Should cross-module type info be persisted as .mambai interface files or kept in-memory only during compilation?
- **Answer**: .mambai interface files — persist compiled type signatures to disk for incremental compilation. Only recompile changed modules.

