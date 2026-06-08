---
number: 837
title: "Incremental compilation and module caching"
state: open
labels: [enhancement, P1, crate:mamba]
group: "compiler-infrastructure"
---

# #837 — Incremental compilation and module caching

## Summary

Mamba currently recompiles every file from scratch on each run. Implement incremental compilation with cached artifacts to improve developer experience.

## Proposed Design

1. **Cache directory**: `~/.mamba/cache/` or `__mamba_cache__/` (analogous to `__pycache__/`)
2. **Cache key**: Hash of (source content + compiler version + config)
3. **Cached artifact**: Serialized MIR or Cranelift object code
4. **Invalidation**: Source file change, dependency change, compiler version bump
5. **Module graph**: Track import dependencies for minimal recompilation

## Scope

- **Driver**: Check cache before compilation, write cache after
- **Serialization**: MIR or object code (de)serialization
- **Dependency tracking**: Build module dependency graph from imports
- **CLI flag**: `--no-cache` to force recompilation
