---
change: mamba-all-p1
group: compiler-infrastructure
date: 2026-03-19
---

# Requirements

Improve Mamba compiler infrastructure in two areas:
- #837 Incremental compilation and module caching: cache directory (`~/.mamba/cache/` or `__mamba_cache__/`), cache key = hash(source + compiler version + config), cached artifact = serialized MIR or Cranelift object code, invalidation on source change / dependency change / compiler version bump, module dependency graph from imports, `--no-cache` CLI flag; requires driver changes, MIR/object-code (de)serialization, and dependency tracking
- #840 Error diagnostics quality: ANSI colored output (error=red, warning=yellow), wavy underlines, fix suggestions ("did you mean X?"), related information (type inference origins, variable definitions), error codes with `--explain` flag, multi-span errors; consider ariadne or miette crate; requires enhancing the existing `diagnostic` module
Acceptance: incremental builds skip unchanged files; error output matches quality bar comparable to rustc messages.
