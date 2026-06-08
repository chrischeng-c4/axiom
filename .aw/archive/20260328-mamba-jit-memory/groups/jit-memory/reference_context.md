---
change: mamba-jit-memory
group: jit-memory
date: 2026-03-26
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cranelift-jit | codegen | high | R4: Executable Memory Management — JITModule's finalize_definitions() internally calls mprotect to make pages executable; on aarch64/macOS concurrent calls from parallel test threads race on the same page table, producing SIGBUS. Current workaround is JIT_LOCK: LazyLock<Mutex<()>> that fully serialises all test threads., R1: JIT Module Initialization — JITModule::new(jit_builder) is called per-compilation; the underlying ISA has shared internal mutable state that races under concurrent init., R3: Function Finalization — finalize_definitions() is the critical section that triggers mprotect; replacing it with an ObjectModule emit + manual mmap(MAP_JIT) load makes each compilation independently allocate its own isolated executable region., Fix direction: replace JITModule with ObjectModule emitting in-memory bytes, then load via jit_memory.rs; remove JIT_LOCK from jit.rs and conformance/mod.rs. |
| cranelift-aot | codegen | high | R1: ObjectModule Initialization — ObjectModule is already used in the AOT path (aot.rs) with the same cranelift-object = '0.116' dep. The new jit_memory.rs will initialise ObjectModule the same way but emit to bytes instead of writing to disk., R2: Object File Generation — ObjectModule produces a Mach-O (macOS) or ELF (Linux) in-memory object via module.finish().emit(). jit_memory.rs parses these bytes with the 'object' crate to find text/data sections., R4: Relocation Handling — cranelift-module resolves all intra-module relocations at compile time. Only load-time symbol fixups for mb_* addresses (already known at compile time via jit_builder.symbol()) are needed. |
| cranelift | codegen | medium | R1: CodegenBackend trait — compile_module() currently returns CodegenOutput::Jit { entry } with a raw function pointer into JITModule memory. After this change, the pointer points into the new JitMemory-managed mmap region; the return type is unchanged., R3: Runtime Symbol Wiring — mb_* addresses are registered into JITBuilder via jit_builder.symbol(); in the ObjectModule path they are registered as external imports resolved at load time in jit_memory.rs. |
| cclab-mamba-fix-xfail-spec | testing | medium | Conformance runner run_and_capture() (conformance/mod.rs:96) acquires JIT_LOCK at line 100 to serialise JIT pipeline. After the fix, this lock acquisition is removed — each test thread compiles independently., Test S2 (run_and_capture_concurrent_calls_serialized) verifies both threads complete without SIGBUS or deadlock — this becomes the primary regression guard for issue #1114., cargo test -p mamba --test conformance_tests (multi-threaded) must pass without -- --test-threads=1 workaround. |
| memory-and-safety | ffi | low | R5: Signal Handling for FFI Crashes — documents SIGBUS recovery via sigsetjmp/siglongjmp in FFI context. Background context: the JIT SIGBUS is not an FFI issue but the mmap safety patterns (per-thread isolation, no global signal state) inform jit_memory.rs design., R6: Thread Safety for Concurrent FFI Calls — per-thread FfiBuffer allocation pattern is analogous to per-compilation JitMemory allocation: each caller owns its region, no shared state. |
| mamba-thread-safe-spec | runtime | low | R3: No-GIL Execution Concurrency — the broader goal; the JIT memory fix is a prerequisite for multi-threaded test parallelism which underpins true No-GIL execution., LazyLock<Mutex<()>> pattern established in gc.rs (GC_TEST_LOCK) — same pattern currently used by JIT_LOCK. Removing JIT_LOCK is a step toward eliminating these serialisation bottlenecks. |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| cranelift-jit-memory-fix | modify | crates/mamba/codegen/cranelift-jit.md | overview, changes, test-plan |
| jit-memory | create | crates/mamba/codegen/jit-memory.md | overview, changes, test-plan |

