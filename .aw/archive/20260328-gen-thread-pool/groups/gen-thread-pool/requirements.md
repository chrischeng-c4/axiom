---
change: gen-thread-pool
group: gen-thread-pool
date: 2026-03-26
---

# Requirements

Replace per-generator thread::spawn in generator.rs with a reusable worker thread. After ~130 sequential generator thread spawns on macOS aarch64, subsequent JIT code execution crashes with EXC_BAD_ACCESS code=257 (PAC failure). The crash is in the calling thread (not generator thread), thread counts stay at 2 (threads ARE joined), and persists even with mem::forget (no JIT page freeing). Root cause: cumulative pthread lifecycle operations on Apple Silicon corrupt process state. Fix: a long-lived worker thread receives (body_fn, args, channels) tuples and executes generator bodies sequentially, reusing the same thread. Key constraints: generator body uses thread_local GEN_TX/GEN_RX for yield communication — the worker thread must set these up per task. Shared capture buffer must be propagated to worker thread. Must support concurrent generators within a single test (multiple comprehensions).
