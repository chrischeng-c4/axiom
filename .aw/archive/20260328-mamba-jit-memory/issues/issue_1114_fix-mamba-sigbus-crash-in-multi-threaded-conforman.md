---
number: 1114
title: "fix(mamba): SIGBUS crash in multi-threaded conformance test execution"
state: open
labels: [type:bug, priority:p0, crate:mamba]
group: "jit-memory"
---

# #1114 — fix(mamba): SIGBUS crash in multi-threaded conformance test execution

## Problem

Running conformance tests with multiple threads (`cargo test -j N` where N > 1) causes intermittent SIGBUS (access to undefined memory) crashes.

## Reproduction

```bash
cargo test -p mamba --test conformance_tests  # multi-threaded: SIGBUS
cargo test -p mamba --test conformance_tests -- --test-threads=1  # single-threaded: passes
```

## Impact

Indicates a memory safety issue in the Mamba runtime. JIT-compiled code or runtime data structures are not thread-safe. This blocks any concurrent Python execution (future threading/multiprocessing support).

## Root Cause

Likely shared mutable state in the runtime (global symbol table, GC state, or JIT code cache) accessed without synchronization.

## Affected Files

- `crates/mamba/src/runtime/` — shared mutable runtime state
- `crates/mamba/src/codegen/cranelift/jit.rs` — JIT code cache
