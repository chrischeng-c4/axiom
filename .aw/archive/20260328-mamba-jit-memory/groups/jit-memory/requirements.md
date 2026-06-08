---
change: mamba-jit-memory
group: jit-memory
date: 2026-03-26
---

# Requirements

Replace cranelift-jit with a custom MambaJitModule that wraps cranelift-codegen directly. On macOS aarch64, use mmap(MAP_JIT) + pthread_jit_write_protect_np() for executable memory instead of alloc::alloc + mprotect. On Linux/x86_64, use standard mmap + mprotect. Must handle: (1) function compilation via cranelift Context, (2) executable page allocation with platform-appropriate APIs, (3) symbol resolution for mb_* runtime functions, (4) relocation patching for cross-function calls and GOT entries, (5) proper free_memory on drop. The 6 cranelift-jit APIs currently used (JITBuilder, JITModule::new, declare_function, define_function, finalize_definitions, get_finalized_function, declare_func_in_func) must be replicated. Drop cranelift-jit from Cargo.toml. Keep cranelift-codegen + cranelift-frontend + cranelift-module.
