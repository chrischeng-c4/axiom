---
id: thread-safe-runtime
title: Thread-Safe Runtime Architecture
crate: mamba
files:
  - crates/mamba/src/runtime/iter.rs
  - crates/mamba/src/runtime/exception.rs
  - crates/mamba/src/runtime/class.rs
  - crates/mamba/src/runtime/closure.rs
  - crates/mamba/src/runtime/generator.rs
  - crates/mamba/src/runtime/gc.rs
  - crates/mamba/src/runtime/module.rs
  - crates/mamba/src/runtime/file_io.rs
  - crates/mamba/src/runtime/output.rs
  - crates/mamba/src/runtime/async_rt.rs
  - crates/mamba/src/runtime/async_task.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 146f6c211
---

# Thread-Safe Runtime Architecture

Cross-cutting spec describing how Mamba's per-module runtime state is
distributed across threads. Two layers cooperate:

- **Thread-local registries** for state that is conceptually per-thread:
  iterators, exception slot, class registry method cache, closure
  cells, generator coroutines, GC state, modules, file handles, output
  capture. All accessed via `thread_local! { static X: RefCell<...> }`
  and dropped at thread exit.
- **Globally-shared registries** for state that must be visible
  across threads: async coroutines + tasks (`LazyLock<RwLock<HashMap>>`
  in `async_rt.rs`), heap `MbObject` allocations themselves
  (refcount + RwLock-protected mutable collections per
  `value-and-rc.md`), and Tokio runtime singletons.

The split exists because:

- Most Python code is single-threaded; thread-local avoids atomic
  contention on hot dispatch paths (`mb_iter`, `mb_lookup_dunder`,
  `mb_call_method`).
- Async code by definition must hop threads (Tokio's multi-thread
  scheduler), so its handles cannot live in thread-local registries.

Three load-bearing invariants:

1. **Generator coroutines are thread-local; async coroutines are
   global** — same name "coroutine" but completely different storage.
   `runtime/generator.rs` runs on the creator thread (stack-swapping
   at the same OS thread); `runtime/async_rt.rs` ships work to any
   Tokio worker. Mixing the two registries would corrupt the wrong
   one — see `generator.md` and `async.md`.
2. **`MbObject` itself is `Send + Sync` via atomic rc + RwLock** —
   the heap object is shareable; what's NOT shared is the
   thread-local handle registries that hand out non-pointer IDs.
   Any function returning a heap MbValue can pass it across threads;
   any function returning an iterator / generator / closure / file
   handle ID must NOT.
3. **`cleanup_all_*` between test runs is mandatory on aarch64** —
   stale function pointers in registries SIGBUS the next test.
   `cleanup_all_runtime_state` is called from the test harness; it
   walks each registry and clears it. Skipping it works on x86_64
   but flakes on M-series Macs.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: thread-safe-types
types:
  ThreadLocalRegistries:  { kind: struct, label: "iter / exception / class / closure / generator / gc / module / file_io / output" }
  GlobalRegistries:       { kind: struct, label: "async_rt COROUTINES / TASKS (LazyLock<RwLock<HashMap>>)" }
  HeapMbObject:           { kind: struct, label: "atomic rc + ObjData with RwLock per mutable variant" }
  TokioRuntime:           { kind: struct, label: "tokio::Runtime singleton" }
  CleanupHooks:           { kind: struct, label: "cleanup_all_runtime_state — calls per-module cleanup" }
edges:
  - { from: ThreadLocalRegistries, to: HeapMbObject,  kind: references, label: "registries hold MbValue ids; values may be heap" }
  - { from: GlobalRegistries,      to: HeapMbObject,  kind: references }
  - { from: GlobalRegistries,      to: TokioRuntime,  kind: references, label: "block_on / spawn" }
  - { from: CleanupHooks,          to: ThreadLocalRegistries, kind: references, label: "clear all between tests" }
  - { from: CleanupHooks,          to: GlobalRegistries,      kind: references }
---
classDiagram
    class ThreadLocalRegistries
    class GlobalRegistries
    class HeapMbObject
    class TokioRuntime
    class CleanupHooks
    ThreadLocalRegistries --> HeapMbObject : MbValue ids
    GlobalRegistries --> HeapMbObject : refs
    GlobalRegistries --> TokioRuntime : block_on / spawn
    CleanupHooks --> ThreadLocalRegistries : clear
    CleanupHooks --> GlobalRegistries : clear
```

## State partition shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "thread-safe-types"
$defs:
  ThreadLocalCategory:
    type: string
    description: "Per-thread state — RefCell<...> in thread_local! block"
    enum:
      - iter::ITERATORS
      - iter::STOP_ITERATION
      - iter::NEXT_ITER_ID
      - exception::CURRENT_EXCEPTION
      - exception::EXCEPTION_HANDLERS
      - class::CLASS_REGISTRY
      - class::CALLABLE_REGISTRY
      - class::SLOTS_REGISTRY
      - class::DICT_SUPPRESSED
      - class::KWARGS_REGISTRY
      - class::METHOD_CACHE
      - class::METHOD_CACHE_GEN
      - class::SIMPLE_CLASS_CACHE
      - closure::CLOSURES
      - closure::CELLS
      - closure::FUNC_NAMES
      - closure::GLOBAL_BY_ID
      - closure::GLOBAL_NAMES
      - generator::GENERATORS
      - generator::ACTIVE_GEN_ID
      - generator::ACTIVE_GEN_CTX
      - generator::YIELD_XFER
      - generator::SEND_XFER
      - generator::THROW_XFER
      - generator::CALLER_CTX_STACK
      - generator::LAST_STOP_VALUE
      - generator::SHARED_CAPTURE
      - gc::GC
      - module::MODULES
      - module::SEARCH_PATHS
      - module::SCRIPT_DIR
      - module::CURRENT_MODULE_PACKAGE
      - module::MODULE_JIT_BACKENDS
      - module::NATIVE_FUNC_ADDRS
      - module::VARIADIC_SYMBOL_IDS
      - module::VARIADIC_FUNC_ADDRS
      - module::KWARGS_SYMBOL_IDS
      - module::KWARGS_FUNC_ADDRS
      - file_io::FILES
      - file_io::NEXT_FILE_ID
      - output::CAPTURE
  GlobalCategory:
    type: string
    description: "Shared across threads — LazyLock<RwLock<HashMap>>"
    enum:
      - async_rt::COROUTINES
      - async_rt::TASKS
      - async_rt::NEXT_CORO_ID
      - async_rt::NEXT_TASK_ID
```

## Cleanup lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: cleanup-lifecycle
initial: TestRunning
nodes:
  TestRunning:    { kind: initial,  label: "fixture executing; registries populated" }
  TestEnded:      { kind: normal,   label: "fixture returned; thread still alive" }
  CleanupRunning: { kind: transient, label: "cleanup_all_runtime_state walking registries" }
  Idle:           { kind: normal,   label: "registries empty; ready for next fixture" }
  ProcessExit:    { kind: terminal, label: "thread exit; thread_locals dropped" }
edges:
  - { from: TestRunning,    to: TestEnded,      event: "fixture body returns" }
  - { from: TestEnded,      to: CleanupRunning, event: "harness invokes cleanup_all_runtime_state (mandatory on aarch64)" }
  - { from: CleanupRunning, to: Idle,           event: "all per-module cleanup_all_* called" }
  - { from: Idle,           to: TestRunning,    event: "next fixture starts" }
  - { from: TestEnded,      to: ProcessExit,    event: "no more fixtures" }
  - { from: Idle,           to: ProcessExit,    event: "harness shutdown" }
---
stateDiagram-v2
    [*] --> TestRunning
    TestRunning --> TestEnded: body returns
    TestEnded --> CleanupRunning: cleanup_all_runtime_state
    CleanupRunning --> Idle: per-module clear
    Idle --> TestRunning: next fixture
    TestEnded --> ProcessExit: harness shutdown
    Idle --> ProcessExit: harness shutdown
    ProcessExit --> [*]
```

## Cross-thread dispatch logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cross-thread-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "JIT-emitted call into runtime" }
  classify:     { kind: decision, label: "what kind of state does it touch?" }
  thread_loc:   { kind: process,  label: "thread-local: RefCell.borrow / borrow_mut" }
  global_lock:  { kind: process,  label: "global: LazyLock + RwLock.read / write" }
  heap_obj:     { kind: process,  label: "heap MbObject: atomic rc + RwLock per mutable container" }
  cross_check:  { kind: decision, label: "is the value an opaque ID (iter / gen / closure / file)?" }
  same_thread:  { kind: terminal, label: "MUST resolve on creator thread; otherwise UB" }
  any_thread:   { kind: terminal, label: "any thread; MbObject is Send + Sync" }
edges:
  - { from: enter,        to: classify }
  - { from: classify,     to: thread_loc,   label: "iter / class / closure / etc." }
  - { from: classify,     to: global_lock,  label: "async coroutine / task" }
  - { from: classify,     to: heap_obj,     label: "list / dict / instance / ..." }
  - { from: thread_loc,   to: cross_check }
  - { from: global_lock,  to: cross_check }
  - { from: heap_obj,     to: cross_check }
  - { from: cross_check,  to: same_thread,  label: "yes (handle id)" }
  - { from: cross_check,  to: any_thread,   label: "no (heap MbValue)" }
---
flowchart TD
    enter([JIT call into runtime]) --> classify{state kind?}
    classify -->|iter / class / closure| thread_loc[RefCell borrow]
    classify -->|async| global_lock[LazyLock + RwLock]
    classify -->|heap object| heap_obj[atomic rc + RwLock]
    thread_loc --> cross_check{handle id?}
    global_lock --> cross_check
    heap_obj --> cross_check
    cross_check -->|yes| same_thread([same-thread only])
    cross_check -->|no| any_thread([any thread])
```

## Cleanup interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: cleanup-flow
actors:
  - { id: Harness,    kind: system, label: "test harness (jit_capture etc.)" }
  - { id: CleanupAll, kind: system, label: "cleanup_all_runtime_state" }
  - { id: PerModule,  kind: system, label: "iter::cleanup / generator::cleanup_all_generators / async_rt::cleanup_all_async / class::cleanup / module::cleanup / file_io / output / closure::cleanup" }
messages:
  - { from: Harness,    to: CleanupAll, name: "cleanup_all_runtime_state()" }
  - { from: CleanupAll, to: PerModule,  name: "iter clear ITERATORS + reset NEXT_ITER_ID" }
  - { from: CleanupAll, to: PerModule,  name: "generator drop coro stacks + clear GENERATORS" }
  - { from: CleanupAll, to: PerModule,  name: "async_rt clear COROUTINES + TASKS + reset counters" }
  - { from: CleanupAll, to: PerModule,  name: "class clear CLASS_REGISTRY + invalidate METHOD_CACHE" }
  - { from: CleanupAll, to: PerModule,  name: "module clear MODULES + drop MODULE_JIT_BACKENDS" }
  - { from: CleanupAll, to: PerModule,  name: "file_io close any open files + clear FILES" }
  - { from: CleanupAll, to: PerModule,  name: "output reset capture buffer" }
  - { from: CleanupAll, to: PerModule,  name: "closure clear CLOSURES + CELLS + GLOBAL_BY_ID + GLOBAL_NAMES" }
  - { from: PerModule,  to: CleanupAll, name: void }
  - { from: CleanupAll, to: Harness,    name: "ready for next fixture" }
---
sequenceDiagram
    participant Harness
    participant CleanupAll
    participant PerModule
    Harness->>CleanupAll: cleanup_all_runtime_state
    CleanupAll->>PerModule: iter clear
    CleanupAll->>PerModule: generator clear + drop stacks
    CleanupAll->>PerModule: async_rt clear
    CleanupAll->>PerModule: class clear + invalidate cache
    CleanupAll->>PerModule: module clear + drop JIT backends
    CleanupAll->>PerModule: file_io close + clear
    CleanupAll->>PerModule: output reset
    CleanupAll->>PerModule: closure clear
    PerModule-->>CleanupAll: void
    CleanupAll-->>Harness: ready
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: cleanup-between-tests
    given: a conformance fixture populates thread-local runtime registries
    when: cleanup_all_runtime_state runs before the next fixture
    then: every per-module registry is empty and stale function pointers cannot affect the next run
  - id: async-cross-thread-task
    given: async_await/gather.py schedules tasks on the Tokio runtime
    when: worker threads poll async task handles
    then: global async_rt registries resolve tasks consistently across threads
  - id: thread-local-isolation
    given: two OS threads create iterator, generator, closure, or file handles
    when: each thread resolves its own handle IDs
    then: thread-local registries keep same numeric IDs isolated per thread
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: thread-safe-runtime-test-plan
title: Thread-Safe Runtime Architecture Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test thread_safety_tests --release -- {name} --test-threads=1"]
    Runner --> Cleanup["test_cleanup_between_runs"]
    Runner --> Async["test_async_task_cross_thread"]
    Runner --> Isolation["test_thread_local_isolation"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/iter.rs
    action: modify
    impl_mode: hand-written
    description: "thread_local ITERATORS / STOP_ITERATION / NEXT_ITER_ID + cleanup hook"
  - file: crates/mamba/src/runtime/exception.rs
    action: modify
    impl_mode: hand-written
    description: "thread_local CURRENT_EXCEPTION / EXCEPTION_HANDLERS"
  - file: crates/mamba/src/runtime/class.rs
    action: modify
    impl_mode: hand-written
    description: "8 class-related thread_local registries"
  - file: crates/mamba/src/runtime/closure.rs
    action: modify
    impl_mode: hand-written
    description: "thread_local CLOSURES / CELLS / FUNC_NAMES / GLOBAL_BY_ID / GLOBAL_NAMES"
  - file: crates/mamba/src/runtime/generator.rs
    action: modify
    impl_mode: hand-written
    description: "thread_local GENERATORS + 7 hot-path cells + CALLER_CTX_STACK + cleanup_all_generators"
  - file: crates/mamba/src/runtime/gc.rs
    action: modify
    impl_mode: hand-written
    description: "thread_local GC state + safepoint stubs"
  - file: crates/mamba/src/runtime/module.rs
    action: modify
    impl_mode: hand-written
    description: "thread_local MODULES + 10 dispatch registries"
  - file: crates/mamba/src/runtime/file_io.rs
    action: modify
    impl_mode: hand-written
    description: "thread_local FILES / NEXT_FILE_ID"
  - file: crates/mamba/src/runtime/output.rs
    action: modify
    impl_mode: hand-written
    description: "thread_local CAPTURE buffer"
  - file: crates/mamba/src/runtime/async_rt.rs
    action: modify
    impl_mode: hand-written
    description: "GLOBAL LazyLock<RwLock<HashMap>> COROUTINES + TASKS + atomic ID counters + cleanup_all_async"
  - file: crates/mamba/src/runtime/async_task.rs
    action: modify
    impl_mode: hand-written
    description: "Public async surface and GIL stub"
```
