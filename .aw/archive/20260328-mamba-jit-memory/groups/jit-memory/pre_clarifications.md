---
change: mamba-jit-memory
group: jit-memory
date: 2026-03-26
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: ObjectModule vs direct emission?
- **Answer**: ObjectModule + loader. Use cranelift ObjectModule to produce in-memory Mach-O/ELF object, parse with `object` crate, load code/data sections into mmap(MAP_JIT) pages. Relocations handled by cranelift-module at compile time; only load-time symbol fixups needed. This is the wasmtime approach and avoids manual relocation patching.

### Q2: General
- **Question**: Where to put the platform-specific JIT memory code?
- **Answer**: Inline in cclab-mamba: src/codegen/cranelift/jit_memory.rs alongside jit.rs.

