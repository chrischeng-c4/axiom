---
id: implementation
type: change_implementation
change_id: mamba-thread-safe-and-no-gil
---

# Implementation

## Summary

Complete thread-safe, no-GIL implementation for the Mamba runtime. 55 files changed, 1219 insertions, 770 deletions. All 289 lib tests and 49 runtime integration tests pass.

### R1: Atomic Reference Counting (`rc.rs`)
- `MbObjectHeader.rc`: `u32` → `AtomicU32`
- `mb_retain`: `fetch_add(1, Ordering::Relaxed)`
- `mb_release`: `fetch_sub(1, Ordering::Release)` with acquire fence before deallocation
- `unsafe impl Send for MbObject {}` and `unsafe impl Sync for MbObject {}`
- Concurrent retain/release test: 10 threads × 100 ops

### R2: Global Thread-Safe GC (`gc.rs`)
- `thread_local! { static GC: RefCell<GcState> }` → `static GC: LazyLock<Mutex<GcState>>`
- `thread_local! { static ROOTS: RefCell<Vec<MbValue>> }` → merged into `GcState.roots`
- `gc_track`: acquires mutex, checks threshold, drops lock before `collect()`
- `collect()`: takes snapshot under lock, releases for mark phase, re-acquires for sweep
- `mark_object`: acquires GC lock for marked set, reads collection RwLocks

### R3: Per-Object RwLock on Mutable Collections (`ObjData` variants)
- `List(Vec<MbValue>)` → `List(RwLock<Vec<MbValue>>)`
- `Dict(HashMap<String, MbValue>)` → `Dict(RwLock<HashMap<String, MbValue>>)`
- `Set(Vec<MbValue>)` → `Set(RwLock<Vec<MbValue>>)`
- `ByteArray(Vec<u8>)` → `ByteArray(RwLock<Vec<u8>>)`
- `Instance { fields: HashMap<...> }` → `Instance { fields: RwLock<HashMap<...>> }`
- Deadlock prevention: `mb_list_extend`, `mb_dict_update` read source first, then write target

### R4: No-GIL Execution
- All 55 runtime files updated to use `lock.read().unwrap()` / `lock.write().unwrap()` patterns
- All `MbObjectHeader { rc: 1, ... }` → `MbObjectHeader { rc: AtomicU32::new(1), ... }`
- Runtime core: builtins.rs, bytes_ops.rs, class.rs, closure.rs, exception.rs, generator.rs, iter.rs, module.rs, string_ops.rs, tuple_ops.rs, async_rt.rs, async_task.rs
- All 33 stdlib modules updated

### R5: Async Scheduling Deferred
- No async scheduler changes (kept single-threaded as specified)

## Changed Files (55)

### Core Runtime (15 files)
- `crates/mamba/src/runtime/rc.rs` — AtomicU32 refcount, RwLock ObjData, Send+Sync
- `crates/mamba/src/runtime/gc.rs` — Global Mutex GC state, mark-sweep under lock
- `crates/mamba/src/runtime/list_ops.rs` — RwLock access for all list operations
- `crates/mamba/src/runtime/dict_ops.rs` — RwLock access for all dict operations
- `crates/mamba/src/runtime/set_ops.rs` — RwLock access for all set operations
- `crates/mamba/src/runtime/builtins.rs` — Updated ObjData access patterns
- `crates/mamba/src/runtime/bytes_ops.rs` — RwLock for ByteArray operations
- `crates/mamba/src/runtime/string_ops.rs` — Updated str access patterns
- `crates/mamba/src/runtime/tuple_ops.rs` — Updated tuple access patterns
- `crates/mamba/src/runtime/class.rs` — RwLock for Instance fields
- `crates/mamba/src/runtime/closure.rs` — AtomicU32 for closure objects
- `crates/mamba/src/runtime/exception.rs` — AtomicU32 for exception objects
- `crates/mamba/src/runtime/iter.rs` — RwLock access for iterator state
- `crates/mamba/src/runtime/module.rs` — RwLock for module attributes
- `crates/mamba/src/runtime/generator.rs` — Updated generator value access

### Async Runtime (2 files)
- `crates/mamba/src/runtime/async_rt.rs` — Updated value access patterns
- `crates/mamba/src/runtime/async_task.rs` — Updated value access patterns

### Stdlib (33 files)
- argparse_mod, array_mod, base64_mod, collections_mod, contextlib_mod, copy_mod, csv_mod, dataclasses_mod, datetime_mod, decimal_mod, enum_mod, functools_mod, glob_mod, gzip_mod, hashlib_mod, html_parser_mod, http_mod, inspect_mod, io_mod, itertools_mod, json_mod, logging_mod, pickle_mod, pprint_mod, random_mod, re_mod, socket_mod, sqlite3_mod, struct_mod, subprocess_mod, tarfile_mod, threading_mod, traceback_mod, unittest_mod, weakref_mod, xml_mod, zipfile_mod

### Tests (1 file)
- `crates/mamba/tests/runtime_tests.rs` — Updated `extract_list` helper

## Test Results

```
test result: ok. 289 passed; 0 failed; 0 ignored (lib tests)
test result: ok. 49 passed; 0 failed; 0 ignored (runtime integration tests)
cargo check: clean (0 errors, 0 warnings in cclab-mamba)
```

## Diff

```diff
 55 files changed, 1219 insertions(+), 770 deletions(-)
 See `git diff main` for full diff.
```

## Review: mamba-thread-safe-and-no-gil-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-thread-safe-and-no-gil

**Summary**: All requirements R1-R5 are fully implemented. 55 files changed across the Mamba runtime. Atomic reference counting with correct memory ordering, global Mutex-protected GC state, per-object RwLock on all mutable collections, and Send+Sync on MbObject. All 289 lib tests and 49 runtime integration tests pass. Async scheduling correctly deferred.

### Checklist

- [PASS] R1: Atomic reference counting for MbObject
  - MbObjectHeader.rc changed from u32 to AtomicU32. mb_retain uses fetch_add(1, Relaxed), mb_release uses fetch_sub(1, Release) with acquire fence before drop. Concurrent test with 10 threads × 100 ops passes.
- [PASS] R2: Global thread-safe GC state and ROOTS
  - thread_local! + RefCell replaced with static LazyLock<Mutex<GcState>>. ROOTS merged into GcState. collect() releases lock during mark phase, re-acquires for sweep. All GC tests pass.
- [PASS] R3: Thread-safe mutable List/Dict/Set operations
  - ObjData variants use RwLock: List(RwLock<Vec>), Dict(RwLock<HashMap>), Set(RwLock<Vec>), ByteArray(RwLock<Vec<u8>>), Instance fields(RwLock<HashMap>). All 55 files updated to lock.read()/lock.write() patterns. Deadlock prevention in extend/update ops.
- [PASS] R4: No-GIL execution concurrency
  - unsafe impl Send + Sync for MbObject. All runtime code uses proper synchronization. No GIL dependency in any execution path.
- [PASS] R5: Async scheduling changes deferred
  - No async scheduler changes introduced. Kept single-threaded as specified.
- [PASS] All tests pass
  - 289 lib tests pass, 49 runtime integration tests pass, cargo check clean.

