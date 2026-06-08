---
change: mamba-all-p1
group: stdlib-system
date: 2026-03-19
---

# Requirements

Implement 7 native stdlib modules that interface with the Mamba runtime and OS:
- #652 atexit: `register()` / `unregister()` backed by Rust process exit hooks
- #653 gc: `collect()`, `disable()`, `enable()`, `get_count()`, `get_threshold()` — integrate with Mamba allocator/GC
- #654 types: `SimpleNamespace`, `FunctionType`, `MethodType`, `ModuleType`, `NoneType`, `GeneratorType`, etc.
- #655 importlib: `import_module()`, `reload()`, `abc.Loader`, `resources` submodule
- #656 codecs: `encode()`, `decode()`, `lookup()`, `register()`, UTF-8/ASCII/Latin-1 codecs
- #657 errno: expose platform errno constants (ENOENT, EACCES, EEXIST, …) via `libc` crate
- #666 tracemalloc: `start()`, `stop()`, `get_traced_memory()`, `take_snapshot()` — integrate with Mamba allocator
All modules must be importable as native Mamba stdlib and expose the CPython-compatible API surface.
