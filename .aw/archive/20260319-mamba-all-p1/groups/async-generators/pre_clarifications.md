---
change: mamba-all-p1
group: async-generators
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: FULLY IMPLEMENTED. generator.send() (generator.rs:350-428), throw() (441-516), close() (519-567) all exist. GeneratorExit and StopIteration exceptions implemented. dispatch_generator_method() in class.rs handles .send/.throw/.close. Thread-based architecture with OS threads + sync channels. Conformance tests exist.

### Q2: General
- **Answer**: NEEDS EXTENSION. tokio_exec.rs has coroutine support (mb_tokio_spawn, mb_tokio_gather, mb_await) but NO async generator support. Missing: async yield protocol, async generator state machine, StopAsyncIteration exception.

### Q3: General
- **Answer**: SYNC WORKING, ASYNC NOT. Sync __iter__/__next__ work (iter.rs). Sync __enter__/__exit__ work (class.rs:1339-1379). AsyncFor parsed but silently dropped in ast_to_hir.rs. AsyncWith parsed but collapsed into regular With. Missing: __aiter__/__anext__/__aenter__/__aexit__ dunder dispatch, HirStmt::AsyncFor/AsyncWith variants.

