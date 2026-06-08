---
change: mamba-p0-bugfix
group: mamba-codegen-runtime-fixes
date: 2026-03-25
---

# Requirements

list(), tuple(), set() must produce empty containers without codegen errors. 'abcdef'[::-1] must produce 'fedcba'. Multi-threaded cargo test must not SIGBUS — JIT code cache and runtime state must be thread-safe.
