---
change_id: mamba-p1
type: knowledge_context
created_at: 2026-02-20T16:34:34.581964+00:00
updated_at: 2026-02-20T16:34:34.581964+00:00
iteration: 3
complexity: high
stage: knowledge
scanned_categories:
  - main_specs
  - knowledge
  - source_code/runtime
---

# Knowledge Context

## Relevant Documents

- **main_spec:cclab-mamba/mamba-oop-model.md**
  - summary: Defines the Mamba OOP model, including C3 MRO, inheritance, attribute access (getattr/setattr), and super() dispatch. Basis for isinstance/issubclass and decorators.
- **main_spec:cclab-mamba/mamba-async-runtime.md**
  - summary: Defines Mamba async/await runtime integration with Orbit bridge and coroutine scheduling using state machines.
- **main_spec:cclab-mamba/mamba-gc-runtime.md**
  - summary: Defines the cycle-detecting Garbage Collector for Mamba, supplementing reference counting for containers like lists and dicts.
- **main_spec:cclab-mamba/mamba-codegen-logic.md**
  - summary: Details lowering of high-level constructs (comprehensions, generators, match/case) into MIR instructions.
- **main_spec:cclab-mamba/mamba-iteration-protocol.md**
  - summary: Specifies __iter__ and __next__ protocols for for-loops and generator expressions.
- **main_spec:cclab-mamba/mamba-import-system.md**
  - summary: Defines module loading, sys.path management, and relative/absolute import resolution.
- **main_spec:cclab-mamba/mamba-stdlib-core.md**
  - summary: Defines built-in functions and core modules (sys, os, time) available in the Mamba environment.
- **knowledge:orbit/bridge-internals.md**
  - summary: Detailed internals of the Orbit bridge connecting Python asyncio with Rust Tokio, including GIL management and Waker implementation.
