---
change: mamba-all-p1
group: compiler-infrastructure
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: NOT IMPLEMENTED. No MIR serialization format exists. MIR structs (mod.rs) are plain Rust without serde. No disk caching infrastructure. CompilerSession recompiles on every build() call. Only runtime module caching exists (in-memory sys.modules equivalent in module.rs). Recommend serialized MIR with serde + bincode for portability.

### Q2: General
- **Answer**: FULLY IMPLEMENTED. module_graph.rs has complete ModuleGraph with search_roots, nodes HashMap, BFS work queue. collect_imports() extracts all imports. ImportDep supports absolute, relative, star imports. resolve_absolute() and resolve_relative() with probe_module() (mirrors CPython __init__.py probing). Kahn's topo_sort() for compilation ordering with cycle detection.

### Q3: General
- **Answer**: MINIMAL. diagnostic/mod.rs (36 lines) has basic Diagnostic struct (severity, message, file, line, col) and render_error() with simple inline format. Type checker has its own Diagnostic in check.rs:11-21. No ANSI colors, no ariadne/miette dependency, no underlines/suggestions/fix hints. error.rs uses thiserror for MambaError enum. Recommend ariadne for rich rendering with LSP compatibility.

