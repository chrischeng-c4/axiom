---
id: jit-memory
main_spec_ref: "crates/mamba/codegen/jit-memory.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes, test-plan]
filled_sections: [overview, requirements, scenarios, changes, test-plan]
create_complete: true
---

# Jit Memory

## Overview

New module `jit_memory.rs` implementing `JitMemory` — a platform-aware executable memory loader for Cranelift-emitted object files. Replaces cranelift-jit's built-in memory management which causes SIGBUS (EXC_BAD_ACCESS code=257 / PAC failure) on macOS aarch64 under concurrent JIT compilation.

`JitMemory` takes in-memory Mach-O (macOS) or ELF (Linux) object bytes produced by `ObjectModule::finish().emit()`, parses them with the `object` crate to extract `.text` and `.data` sections, allocates executable pages using platform-appropriate APIs, resolves external `mb_*` symbol relocations, and returns a callable function pointer to the entry symbol.

Platform memory strategies:
- **macOS aarch64**: `mmap(MAP_JIT)` + `pthread_jit_write_protect_np(false)` to write, then `pthread_jit_write_protect_np(true)` to execute. No `mprotect` needed — MAP_JIT pages are always RX when JIT-write-protect is on.
- **Linux / other**: `mmap(PROT_READ | PROT_WRITE)` → copy code → `mprotect(PROT_READ | PROT_EXEC)` to make executable.

Each compilation allocates its own isolated mmap region — no shared mutable state between threads. This eliminates the need for `JIT_LOCK` serialization.

File: `crates/mamba/src/codegen/cranelift/jit_memory.rs`
Issue: #1114
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Object file parsing | P0 | `JitMemory::load(object_bytes, symbols)` parses in-memory Mach-O (macOS) or ELF (Linux) bytes using the `object` crate. Extracts `.text` section offset, size, and content. Returns error if `.text` section is missing or object format is unrecognized |
| R2 | macOS aarch64 executable page allocation | P0 | On `target_os = "macos"` + `target_arch = "aarch64"`, allocates pages via `mmap(MAP_JIT \| MAP_ANON \| MAP_PRIVATE, PROT_READ \| PROT_WRITE \| PROT_EXEC)`. Uses `pthread_jit_write_protect_np(false)` before writing code, `pthread_jit_write_protect_np(true)` after. No `mprotect` calls |
| R3 | Linux/x86_64 executable page allocation | P0 | On non-macOS targets, allocates pages via `mmap(MAP_ANON \| MAP_PRIVATE, PROT_READ \| PROT_WRITE)`, copies code, then calls `mprotect(PROT_READ \| PROT_EXEC)` to make executable |
| R4 | External symbol relocation resolution | P0 | Iterates relocations in the `.text` section. For each relocation referencing an external symbol (`mb_*` runtime functions), patches the instruction at the relocation offset with the absolute address from the provided `symbols: HashMap<String, *const u8>`. Supports `R_AARCH64_CALL26` (macOS) and `R_X86_64_PLT32` / `R_X86_64_PC32` (Linux) relocation types |
| R5 | Entry symbol lookup | P0 | After loading, looks up the entry function symbol (e.g., `_mb_entry` or `_mb_0`) in the parsed object's symbol table and returns its address within the mmap region as a `fn() -> i64` function pointer |
| R6 | Thread-safe isolation | P0 | Each `JitMemory` instance owns its mmap region exclusively. No shared mutable state between instances. Multiple threads can create and use independent `JitMemory` instances concurrently without synchronization |
| R7 | Memory cleanup on Drop | P1 | `JitMemory` implements `Drop` calling `munmap(ptr, len)` on the allocated pages. No executable memory leak after the struct is dropped |
| R8 | Data section support | P2 | If the emitted object contains a `.data` or `.rodata` section, allocates a separate RW region, copies data, and adjusts relocations that reference data symbols. Required for string constants and global data emitted by Cranelift |

### Constraints

- Platform-specific code gated with `#[cfg(target_os = "macos")]` / `#[cfg(not(target_os = "macos"))]`
- `object` crate version must match cranelift-object's transitive dependency to avoid duplicate types
- `libc` crate for `mmap`, `munmap`, `mprotect`, `MAP_JIT` constants
- `pthread_jit_write_protect_np` is macOS-only — declared via `extern "C"` FFI block
- Page size alignment: all mmap allocations rounded up to system page size (`sysconf(_SC_PAGESIZE)`)
- Relocation types: only need to handle relocations emitted by Cranelift for the target platform — not a general-purpose linker
## Scenarios

### S1: Load and execute a simple compiled function (R1, R2/R3, R5)

**GIVEN** in-memory object bytes from `ObjectModule::finish().emit()` containing a single function `_mb_entry` that returns constant `42`
**WHEN** `JitMemory::load(bytes, HashMap::new())` is called
**THEN** returns `Ok(jit_mem)` where `jit_mem.entry()` is a valid `fn() -> i64` pointer; calling it returns `42`

### S2: Resolve mb_* external symbol relocations (R4, R5)

**GIVEN** object bytes containing a function that calls `mb_print(value)` with one relocation referencing `mb_print`
**WHEN** `JitMemory::load(bytes, symbols)` is called with `symbols` containing `"mb_print" => mb_print as *const u8`
**THEN** the relocation is patched to the correct address; calling the entry function invokes `mb_print` at the runtime address

### S3: macOS MAP_JIT allocation path (R2)

**GIVEN** running on macOS aarch64
**WHEN** `JitMemory::load()` allocates executable pages
**THEN** uses `mmap` with `MAP_JIT | MAP_ANON | MAP_PRIVATE` flags; calls `pthread_jit_write_protect_np(false)` before `memcpy`, `pthread_jit_write_protect_np(true)` after; no `mprotect` syscall is issued

### S4: Linux mmap+mprotect allocation path (R3)

**GIVEN** running on Linux x86_64
**WHEN** `JitMemory::load()` allocates executable pages
**THEN** uses `mmap` with `PROT_READ | PROT_WRITE`, copies code via `memcpy`, then calls `mprotect` with `PROT_READ | PROT_EXEC`

### S5: Concurrent loads from multiple threads (R6)

**GIVEN** 4 threads each with independent object bytes and symbol maps
**WHEN** all 4 call `JitMemory::load()` concurrently
**THEN** all succeed without SIGBUS, data races, or memory corruption; each returns an independently callable entry pointer

### S6: Memory freed on drop (R7)

**GIVEN** a `JitMemory` instance holding an mmap region of N pages
**WHEN** the instance is dropped
**THEN** `munmap` is called with the base pointer and full allocation size; the address range is no longer mapped

### S7: Invalid object bytes rejected (R1)

**GIVEN** arbitrary non-object bytes (e.g., `b"not an object"`)
**WHEN** `JitMemory::load(bytes, symbols)` is called
**THEN** returns `Err` with a descriptive error message; no mmap allocation occurs

### S8: Missing symbol relocation fails gracefully (R4)

**GIVEN** object bytes with a relocation referencing `mb_unknown` not present in `symbols`
**WHEN** `JitMemory::load(bytes, symbols)` is called
**THEN** returns `Err` indicating the unresolved symbol name; no partial mmap region is leaked
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

| Test | Type | Covers | Description |
|------|------|--------|-------------|
| `test_jit_memory_load_simple_function` | unit | S1, R1, R2/R3, R5 | Produce object bytes from ObjectModule containing `_mb_entry` returning constant 42. Call `JitMemory::load()`, invoke `entry()`, assert return value is 42 |
| `test_jit_memory_relocation_mb_print` | unit | S2, R4 | Produce object bytes with a call to `mb_print`. Provide symbol map with `mb_print` address. Load via JitMemory, execute, verify `mb_print` was invoked (use test stub that sets a flag) |
| `test_jit_memory_multiple_relocations` | unit | S2, R4 | Produce object bytes calling `mb_alloc`, `mb_get_attr`, `mb_print`. Load with all symbols. Execute and verify all three are called correctly |
| `test_jit_memory_concurrent_load` | integration | S5, R6 | Spawn 4 threads, each loading independent object bytes via `JitMemory::load()`. All must succeed without SIGBUS. Each entry pointer returns correct value |
| `test_jit_memory_drop_munmap` | unit | S6, R7 | Create JitMemory, record ptr and len, drop it. On Linux, verify via `/proc/self/maps` that the region is unmapped. On macOS, verify munmap return code (wrap in test helper) |
| `test_jit_memory_invalid_bytes` | unit | S7, R1 | Call `JitMemory::load(b"garbage", &HashMap::new())`. Assert returns `Err` with message containing "parse" or "object" |
| `test_jit_memory_missing_symbol` | unit | S8, R4 | Produce object bytes referencing `mb_unknown`. Call `JitMemory::load()` with empty symbol map. Assert returns `Err` mentioning `mb_unknown`. No mmap leak |
| `test_jit_memory_page_alignment` | unit | R2/R3 | Load object bytes of various sizes (1 byte, 4095 bytes, 4096 bytes, 8193 bytes). Verify allocated region is always page-aligned (ptr % page_size == 0, len % page_size == 0) |

### Regression watchlist

- Companion spec `cranelift-jit-memory-fix` tests cover the integration of JitMemory with `CraneliftJitBackend` — those tests exercise the full pipeline
- `test_conformance_multi_threaded` (in companion spec) is the end-to-end regression test for #1114
- AOT path (`aot.rs`) must remain unaffected — it uses ObjectModule independently and does not interact with JitMemory
## Changes

```yaml
files:
  - path: crates/mamba/src/codegen/cranelift/jit_memory.rs
    action: CREATE
    desc: |
      New file implementing JitMemory struct.

      Struct definition:
      - `JitMemory { ptr: *mut u8, len: usize, entry_offset: usize }`
      - `ptr`: base address of mmap'd region
      - `len`: total allocation size (page-aligned)
      - `entry_offset`: offset of entry symbol within the region

      Public API:
      - `JitMemory::load(object_bytes: &[u8], symbols: &HashMap<String, *const u8>) -> Result<Self>`
        1. Parse object_bytes with `object::File::parse()`
        2. Find `.text` section, get content + size
        3. Compute page-aligned allocation size
        4. Allocate executable pages (platform-specific, see below)
        5. Copy `.text` content into mmap region
        6. Iterate relocations in `.text` section:
           - For each external symbol relocation, look up address in `symbols` HashMap
           - Patch instruction at relocation offset with computed target address
           - Support `R_AARCH64_CALL26` and `R_X86_64_PLT32` / `R_X86_64_PC32`
        7. Find entry symbol (`_mb_entry` or `_mb_0`) in symbol table, compute offset
        8. Finalize memory protection (platform-specific)
        9. Return JitMemory with ptr, len, entry_offset
      - `JitMemory::entry(&self) -> fn() -> i64`
        Returns `transmute(self.ptr.add(self.entry_offset))`

      Platform-specific allocation:
      ```
      #[cfg(target_os = "macos")]
      - mmap(NULL, size, PROT_READ|PROT_WRITE|PROT_EXEC, MAP_JIT|MAP_ANON|MAP_PRIVATE, -1, 0)
      - pthread_jit_write_protect_np(false)  // enable writing
      - memcpy .text content
      - patch relocations
      - pthread_jit_write_protect_np(true)   // enable execution

      #[cfg(not(target_os = "macos"))]
      - mmap(NULL, size, PROT_READ|PROT_WRITE, MAP_ANON|MAP_PRIVATE, -1, 0)
      - memcpy .text content
      - patch relocations
      - mprotect(ptr, size, PROT_READ|PROT_EXEC)
      ```

      FFI declarations:
      ```
      extern "C" {
          fn pthread_jit_write_protect_np(enabled: libc::c_int);  // macOS only
      }
      ```

      Drop impl:
      - `unsafe { libc::munmap(self.ptr as *mut libc::c_void, self.len) }`

      Safety:
      - `JitMemory: Send + Sync` — the mmap region is owned exclusively, no shared state
      - Entry pointer is valid only while JitMemory is alive
      - Caller must ensure no code pointers are dereferenced after drop
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
