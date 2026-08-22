# execution context — DDD migration contract

Scope: the runtime state owned by one compiled-and-executed Mamba program in
one process. This is the target ownership model for #2530; the current tree
still relies on process-global locks and thread-local registries.

## Bounded context

The `ExecutionContext` context begins when a caller prepares one program for
execution and ends only after result/output ownership is secured and all
registries created by that execution are retired. It owns execution-local
state; process-wide immutable compiler caches and OS services stay outside.

This boundary is narrower than the Python interpreter as a whole and broader
than a single JIT module. A context may create worker threads and coroutines,
but those children remain inside the same aggregate until quiescence.

## Aggregate root

`ExecutionContext` is the aggregate root.

```text
Created -> Installed -> Executing -> Quiescing -> Retired
                         \-> Failed -> Quiescing
```

An ABI entry resolves the currently installed context, performs the operation,
and returns without transferring ownership of the context itself.

### Entities and value objects

| Type | Kind | Identity / value |
|---|---|---|
| `ExecutionContext` | aggregate root | opaque monotonic context id |
| `JitSession` | entity | context id + compiled module id |
| `RuntimeRegistrySet` | entity | context id |
| `OutputCapture` | entity | context id + capture nesting id |
| `ExecutionChild` | entity | context id + child id |
| `ContextHandle` | value | nonzero opaque ABI-safe integer/pointer token |
| `ExecutionPhase` | value | `Created`, `Installed`, `Executing`, `Quiescing`, `Retired`, `Failed` |
| `RegistryKind` | value | modules, closures, cells, functions, exceptions, iterators, generators, async tasks, or native handles |

## Ownership classification

Every mutable static or thread-local reached from compile/execute is classified
before migration:

| Class | Rule | Destination |
|---|---|---|
| context-owned | values differ between two concurrent executions | field or sub-aggregate of `ExecutionContext` |
| child-owned | lifetime is bounded by a worker/coroutine within one context | `ExecutionChild`, joined during quiescence |
| process immutable | initialized once and never mutated after publication | process-global cache, outside the aggregate |
| process service | intentionally shared OS/process resource with independent synchronization | explicit service handle, outside the aggregate |
| compatibility binding | current-context lookup needed by unchanged `mb_*` ABI | scoped TLS stack containing `ContextHandle` only |

The inventory must name symbol, source path, current storage, mutation sites,
reset path, ownership class, migration destination, and evidence. An unknown
classification is a blocker, not a default to TLS.

## Thread model — decided 2026-07-31 (#3191)

The contract previously said a context's worker threads "remain inside the same
aggregate" and named an `ExecutionChild` entity, while the implementation made
that unreachable: `CONTEXT_STACK` is a thread-local and `bind_context` takes
`&ExecutionContext`, so a worker would need `ExecutionContext: Sync` to see its
parent. Every state field is a `RefCell`, so the type is unconditionally
`!Sync`; since #2843 it is also `!Send` (#3190). Nothing in `src/**` referenced
`ExecutionChild`, `Arc<ExecutionContext>`, or `thread::scope`.

The contradiction is resolved as follows.

**`ExecutionContext` is thread-pinned.** It is the per-thread execution slice.
It is neither `Send` nor `Sync`, and this is intentional rather than incidental
to whichever field happens to hold a `RefCell` or a raw pointer. A worker thread
gets its own context; it never binds its parent's.

**Program-wide name-binding state does not live in `ExecutionContext`.** Python
semantics require module globals, `sys.modules`, `sys.path`, class attributes,
and function metadata to be shared by every thread of one program: `counter +=
1` in two workers must sum, and a rebind in a worker must be visible to the
parent *while the worker is still running*. Those registries move into a
separate `ProgramState` sub-aggregate with `Sync`-safe interior mutability,
shared by `Arc` and handed to each worker at spawn. `RefCell` cannot back it.

| state | owner | reason |
|---|---|---|
| module globals / module symbol registry | `ProgramState` (shared) | CPython shares module globals across threads |
| `sys.modules` module table | `ProgramState` (shared) | one import cache per program |
| `sys.path` search paths | `ProgramState` (shared) | one path list per program |
| class attributes | `ProgramState` (shared) | class objects are program-wide |
| function metadata | `ProgramState` (shared) | reachable from any thread |
| closure/cell **storage** | `ProgramState` (shared) | see "Closures and cells" below (#3194) |
| closure/cell **bindings** | `ExecutionContext` (pinned) | see "Closures and cells" below (#3194) |
| pending exception | `ExecutionContext` (pinned) | per-thread by Python semantics |
| output capture / redirect | `ExecutionContext` (pinned) | harness-scoped capture, not `sys.stdout` |
| module JIT backends | `ExecutionContext` (pinned) | `CraneliftJitBackend` is `!Send` |

Stages #2839-#2843 migrated the pinned column. #2844 onwards must not migrate a
shared-column registry into `ExecutionContext`; doing so would make #3132's
per-thread globals snapshot the intended semantics.

**This decision is mechanical, not documentary.** A compile-time assertion pins
`ExecutionContext` as `!Send`/`!Sync`, and `ProgramState` as `Send + Sync`, so a
later stage adding a field cannot flip either by accident.

**Consequence for what has landed.** No production path constructs or binds a
context: `ExecutionContext::create` and `bind_context` appear only under
`#[cfg(test)]`, and `src/driver/**`, `src/lib.rs`, and `src/main.rs` do not
reference the type at all. Every runtime call still reaches the `FALLBACK_*`
thread-locals. Stages 1-5 are therefore scaffolding with no behavioral effect;
the migration goes live at stage 6, when the driver constructs contexts
explicitly. Stages 1-5 landing is not evidence that threading semantics changed.

### Closures and cells — decided 2026-07-31 (#3194)

The table above deferred this row to the slice that would have to implement it.
#3194 implemented it, and the decision is a **split**, not a single owner:

- **Cell storage is shared.** `ProgramState` holds `closures:
  RwLock<Vec<Option<MbClosure>>>` and `cells: RwLock<Vec<Option<MbValue>>>`,
  alongside the fourteen `func_*` metadata registries.
- **Cell bindings are per-thread.** `ACTIVE_CELLS: RefCell<HashMap<ScopedSymbolKey,
  MbValue>>` stays a `thread_local!` in `runtime/closure.rs`, next to
  `ACTIVE_MODULE_NAMES` and `ACTIVE_QUALNAME_CONTEXTS`.

The reason the row could not be answered in the abstract is that "closures and
cells" is two different things wearing one name.

A **cell** is an object with identity. `nonlocal counter` mutated by a worker must
be visible to the parent while the worker still runs — the same requirement that
put module globals in the shared column. A cell reached through a closure a
worker was handed is by construction reachable from more than one thread, so
per-thread storage would either lose the write or fork the object. Storage is
therefore shared, under the same `RwLock` discipline as every other
`ProgramState` registry.

A **binding** — which `ScopedSymbolKey` currently resolves to which cell — is a
property of the executing frame, not of the program. Two threads inside the same
function have different frames and must resolve the same key to different cells.
Sharing that map would make one worker's scope entry visible to another, which is
the closure-shaped restatement of #3132's per-thread-globals defect. Bindings are
therefore pinned, exactly like the pending exception.

The general rule this instance follows: **object identity is shared, frame
resolution is pinned.** A registry belongs in `ProgramState` when two threads
must agree on *which object*, and in `ExecutionContext` when two threads must
disagree about *which name resolves where*.

Consequences worth stating explicitly, because they are easy to get wrong later:

- A closure handed to `threading.Thread(target=...)` carries its cells with it.
  The worker does not re-derive them from its own binding map; it reads shared
  storage. This is what #3178 and #3201 were failing to do.
- Per-thread teardown may clear the binding map. It must **not** clear shared
  storage — another thread's live cells are in there. `cleanup_all_closures`
  currently does clear it; that is tracked as #3207 and is a violation of this
  contract, not an amendment to it.

### Verification

`gate-3194.sh` 12/12, `gate-3201.sh` 19/19, `gate-2844-r2.sh` PASS, and an
independent before/after probe over #3178 + #3204 moving 3 pass / 5 fail →
8 pass / 8. `cargo test -p mamba --lib runtime:: -- --test-threads=1` at 3538
passed / 0 failed (was 3536 / 2). Separately measured against serialization —
the failure mode where every functional gate goes green because the runtime
quietly stopped being concurrent — four workers show 6 of 6 pairs overlapping in
wall-clock time. Multicore *throughput* evidence (#2022) is not yet taken; the
machine was not quiet enough for the number to mean anything.

## Invariants

1. **No ambient mutable execution state.** Two contexts cannot observe or
   clear each other's capture buffers, registries, exceptions, modules, cells,
   tasks, or teardown bookkeeping.
2. **ABI lookup is scoped.** A public `mb_*` entry resolves exactly one
   installed context from a stack-like TLS binding. Missing or mismatched
   context is an explicit error in debug/test builds.
3. **Nested calls restore bindings.** Installing a nested context restores the
   previous binding through RAII on success, error, or panic.
4. **Child work is program-owned, not context-owned.** A worker thread has its
   own thread-pinned context and shares its parent's `ProgramState`. A program
   cannot retire its `ProgramState` while a worker may still publish into it,
   and a worker's rebind of shared state is visible to every other thread
   immediately — not at `join()`.
5. **Capture is context-local and nestable.** Output from two contexts never
   crosses; nested capture restores only its own prior buffer.
6. **Cleanup is idempotent and local.** Quiescing one context touches no other
   context and may be invoked after partial setup or failure.
7. **JIT serialization is minimal.** Any remaining global lock protects only a
   named process-global Cranelift operation; compile, execute, capture, and
   teardown are not enclosed by one global critical section.
8. **Public behavior is preserved.** The `mb_*` symbol surface and Force Typed
   compile/runtime behavior do not change during ownership migration.
9. **Parallelism is observed, not inferred.** Acceptance requires overlapping
   in-process execution and correctly attributed results/output; removal of a
   lock alone is not evidence.

## Current state inventory roots

The first inventory pass starts at:

- `src/runtime/output.rs`: capture and redirect TLS.
- `src/runtime/closure.rs`: closures, cells, function metadata, global/module
  symbol registries.
- `src/runtime/module.rs`: module tables, search paths, native/variadic/kwargs/
  boxed-return registries, module JIT backends.
- `src/runtime/exception.rs`, iterator/generator/async modules, and GC state.
- `src/runtime/mod.rs::cleanup_all_runtime_state`.
- `src/codegen/cranelift/jit.rs::JIT_LOCK` and mutable JIT session ownership.
- `src/driver/**` and conformance harness call sites that install, execute,
  capture, and clean up.

The inventory expands transitively from reads/writes reached by those roots; it
does not stop at this starter list.

## Migration stages

### Stage 1 — inventory and ownership decision

Controller-owned design work. Produce the complete classification table and
two sequence diagrams: normal execution and failure/cleanup. No `src/**`
change.

Oracle: a source scanner enumerates mutable statics/TLS under the declared
roots; every discovered symbol appears exactly once in the reviewed inventory,
and every inventory row resolves to a real symbol.

### Stage 2 — context shell and scoped ABI binding

Introduce the aggregate root, context handle, scoped TLS binding, and
idempotent local teardown without migrating all registries.

Oracle: two empty contexts can be nested and used on two threads; bindings
restore after normal return and panic; cleanup of one cannot change the other's
sentinel state.

### Stage 3 — output and exception slice

Move capture/redirection and pending-exception state first because their output
is directly observable and supplies a small parallel proof.

Oracle: two contexts execute concurrently with deliberately interleaved
stdout/stderr and distinct exceptions; each result contains only its own data.

### Stage 4 — runtime registry slices

Move registries by cohesive owner: module/import, closure/cell/function
metadata, iterator/generator, async/task, and native handles. One slice per
work item; each adds a two-context isolation test before the next slice starts.

Oracle: the migrated registry has exact per-context sentinel isolation, and
context-local cleanup is idempotent under concurrent execution.

### Stage 5 — JIT session and lock contraction

Attach mutable JIT modules/symbol publication to the owning context. Retain a
process lock only around a proven Cranelift process-global operation.

Oracle: two compile+execute paths overlap in wall time, produce distinct
results, and never hold the remaining lock across execution or teardown.

### Stage 6 — harness parallelism and legacy cleanup

Switch driver/conformance helpers to construct contexts explicitly, delete
obsolete global reset paths, and run the integration target with more than one
test thread.

Oracle: the bounded two-context canary passes repeatedly, the selected
integration denominator is green at `--test-threads > 1`, and no global
serialization is observed.

## Dependency and rollout rules

- Stage 1 precedes every source change.
- Stage 2 precedes all registry migrations.
- Stage 3 is the first product slice and calibrates AGY/controller handoff.
- Stage 4 slices are serial until two independently verified landings show no
  cross-context state leak; concurrency may then increase gradually.
- Stage 5 starts only after registry writes reachable from JIT execution are
  context-owned.
- Stage 6 follows the required stdlib/type-wall evidence work; test-suite
  parallelism must not outrun semantic determinism.

## Forbidden fixes

- Replacing `JIT_LOCK` with another lock spanning compile+execute+cleanup.
- Moving all state into one new process-global singleton named
  `ExecutionContext`.
- Using unscoped raw pointers or a non-restoring TLS setter at the ABI boundary.
- Declaring success because tests happen to pass serially.
- Migrating several independent registry families in one AGY ticket.
- Migrating a shared-column registry (globals, module table, search paths, class
  attributes, function metadata) into `ExecutionContext`. Those belong to
  `ProgramState`; putting them in a thread-pinned context cements #3132.
- Adding `unsafe impl Send`/`Sync` to `ExecutionContext` or to a type it owns in
  order to make a test or a spawn compile.
- Proving a shared-state migration with sibling-context isolation alone.
  Isolation is what the broken design already does; only cross-thread visibility
  during a worker's lifetime discriminates.
- Changing the public `mb_*` ABI before a separately reviewed compatibility
  decision.

## Verification surface

- Source-inventory exact-set gate.
- Source-local context binding and teardown tests.
- Two-context interleaved output/exception canary.
- Per-registry sentinel isolation tests (pinned column only).
- Compile-time trait assertions: `ExecutionContext` is `!Send`/`!Sync`,
  `ProgramState` is `Send + Sync`.
- Cross-thread visibility gate for the shared column: a worker's rebind of a
  global is observed by the parent while the worker is still running, and two
  workers incrementing the same global sum rather than last-writer-win.
- Lock-scope structural assertion plus overlapping compile/execute timing.
- Selected integration denominator with `--test-threads > 1`.
- Existing Force Typed and release-profile gates remain unchanged.
