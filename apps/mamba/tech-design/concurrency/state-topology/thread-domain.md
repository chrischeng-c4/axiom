# Thread domain state topology

Issue: #2988  
Parent inventory: #2968  
Source revision: `b595b9e76b`

This Stage 1 slice classifies the state currently declared by
`runtime/stdlib/threading_mod.rs`. It defines the DDD boundary for logical
Python threads, child execution state, OS-thread services, and object-owned
synchronization primitives. It makes no `src/**` change.

## Bounded contexts

```text
Process
├── LogicalThreadIdAllocator
├── ThreadSpawner
└── NativeCallableCatalog                 # #2982, immutable after publication

ExecutionContext
├── shared Python heap/global/cell domain
└── ThreadDomain                          # aggregate
    ├── main logical-thread descriptor
    ├── child registry[LogicalThreadId]
    ├── live-thread projection
    ├── hook defaults and propagation generation
    └── requested stack policy

ExecutionThreadState                      # child entity
├── logical identity
├── native OS identity
├── current Python Thread reference
├── trace/profile hook execution state
└── local_values[LocalObjectId]

Python heap objects
├── ObjData::Lock                         # object-owned lock state
└── ObjData::Barrier                      # object-owned barrier state
```

`ThreadDomain` belongs to exactly one `ExecutionContext`. It owns logical
thread membership, lifecycle, and projections such as `enumerate()` and
`active_count()`. `ThreadSpawner` supplies OS threads and join handles but does
not own Python logical-thread membership. Separate contexts may use the same
process service without being able to enumerate, join, or mutate each other's
children.

## Aggregate and entities

`ThreadDomain` is a sub-aggregate of `ExecutionContext`.

| Type | Kind | Identity / value |
|---|---|---|
| `ThreadDomain` | aggregate root | `ContextId` |
| `ThreadRecord` | entity | `ContextId + LogicalThreadId` |
| `ExecutionThreadState` | entity | `ContextId + LogicalThreadId` |
| `LogicalThreadId` | value | opaque process-unique integer |
| `NativeThreadId` | value | OS-provided thread identity |
| `ThreadPhase` | value | `Created`, `Starting`, `Running`, `Finished`, `Joined`, `Failed` |
| `ThreadObjectRef` | value | retained context-heap reference |
| `NativeThreadHandle` | value | process-service handle scoped by `ContextId + LogicalThreadId` |
| `LocalObjectId` | value | stable heap-object identity, not a reused raw address |
| `HookGeneration` | value | monotonically increasing domain hook-policy version |

The aggregate transition is:

```text
Created -> Starting -> Running -> Finished -> Joined
              \----------> Failed -----------/
```

Registration happens before the OS child may publish. Every transition is
serialized by the domain registry. A raw `MbValue` bit pattern is never a
cross-context authority to find or join a child.

## Frozen inventory

The 16 production identities have sorted newline SHA-256
`10eb8c818ab0c717ffb7b2bdbfa870325f2e45df38c82f4b74c26fff9545ac5a`.
Three additional declarations are test-only.

| Current symbol | Current storage | Current role | Target owner / disposition |
|---|---|---|---|
| `THREAD_NAME` | TLS `Cell<Option<String>>` | test-written fallback name | `ExecutionThreadState` |
| `PROFILE_FN` | TLS `Cell<MbValue>` | caller-only profile slot | `ThreadDomain` default policy; child active slot is explicit |
| `TRACE_FN` | TLS `Cell<MbValue>` | caller-only trace slot | `ThreadDomain` default policy; child active slot is explicit |
| `TRACE_PROFILE_HOOK_ACTIVE` | TLS `Cell<bool>` | hook reentrancy guard | `ExecutionThreadState` |
| `STACK_SIZE` | TLS `Cell<i64>` | unconsumed compatibility value | `ThreadDomain` requested stack policy |
| `CURRENT_IDENT` | TLS `Cell<i64>` | logical id also misreported as native id | `ExecutionThreadState.logical_thread_id` |
| `NEXT_THREAD_IDENT` | process `AtomicI64` | logical id allocation | process `LogicalThreadIdAllocator` |
| `LIVE_THREADS` | TLS `RefCell<Vec<u64>>` | spawner-local live projection | `ThreadDomain` child registry |
| `CURRENT_THREAD_OBJ` | TLS `Cell<u64>` | current Python Thread bits | `ExecutionThreadState` |
| `WORKER_STDLIB_READY` | TLS `Cell<bool>` | per-worker stdlib registration | remove after sealed native catalog |
| `THREAD_HANDLES` | process `OnceLock<Mutex<HashMap<...>>>` | raw-bits join-handle table | `ThreadDomain` resource registry |
| `LOCK_STATES` | process `OnceLock<Mutex<HashMap<usize, ...>>>` | address-keyed lock side table | `ObjData::Lock` |
| `NEXT_BARRIER_ID` | process `AtomicU64` | barrier side-table surrogate id | remove with barrier side table |
| `BARRIERS` | process `LazyLock<Mutex<HashMap<u64, ...>>>` | id-keyed barrier side table | `ObjData::Barrier` |
| `LOCAL_INSTANCES` | TLS `RefCell<Vec<MbValue>>` | snapshot/clear/restore registry | `ExecutionThreadState.local_values` |
| `MAIN_THREAD` | TLS `RefCell<Option<MbValue>>` | per-OS-thread singleton cache | `ThreadDomain` main descriptor |

The discarded declarations are `TEST_TRACE_EVENTS`,
`TEST_TRACE_RETURN_ARGS`, and `TEST_GLOBAL_TRACE_RETURN`; all are inside the
test module and do not enter production ownership.

## Current defects

### Logical membership is stored on the caller OS thread

`Thread.start()` adds a retained Python Thread reference to the spawner's
`LIVE_THREADS` TLS. `enumerate()` reads only the caller's TLS, and `join()`
removes from the joining thread's TLS. Different callers therefore observe
different membership projections.

`live_threads_add` calls `retain_if_ptr`, while `live_threads_remove` filters
the raw bits without the matching release. The current removal path leaks the
retained reference. `active_count()` is independently hard-coded to `1`, even
while real worker threads may be running.

### A process table grants join authority by raw object bits

`THREAD_HANDLES` maps a Python object bit pattern directly to a process-global
`JoinHandle`. It has no context tag. A joined handle is removed, but an
unjoined handle has no retirement policy and context shutdown cannot prove
that all children stopped publishing.

### Start and join move snapshots instead of binding shared ownership

Start snapshots locals, globals, active closure cells, and class state. Join
merges returned global and cell snapshots into the joining OS thread. This can
lose concurrent updates and incorrectly makes ownership depend on which thread
joins.

Python heap objects, module globals, and closure cells that are semantically
shared remain owned by the parent `ExecutionContext`. A child binds that same
context domain. Only frame stacks, current-thread identity, hook execution
state, and `threading.local` values are child-owned.

### API names overstate behavior

`setprofile_all_threads` and `settrace_all_threads` currently write only the
caller's TLS. New workers do not inherit those slots, and existing workers are
not updated. `STACK_SIZE` is read and written by `threading.stack_size` but is
not consumed by `std::thread::spawn` or a thread builder.

`CURRENT_IDENT` backs both `get_ident()` and `get_native_id()`. Logical Python
identity and OS native identity are separate contracts and must not alias.

### Side tables outlive their Python objects

`LOCK_STATES` and `BARRIERS` are private module statics with no removal path in
the audited module. Raw addresses may be reused, and the tables retain entries
after the corresponding Python object dies. `NEXT_BARRIER_ID` exists only to
support that misplaced barrier table.

### Worker bootstrap mutates process-native facts

`WORKER_STDLIB_READY` permits each OS worker to call `register_stdlib()`.
#2982 instead seals one immutable process-native callable catalog before any
worker runs. Worker creation must acquire a shared reference, not reconstruct
native registration state.

## Thread creation contract

```mermaid
sequenceDiagram
    participant Parent as Parent ExecutionThreadState
    participant Domain as ThreadDomain
    participant Spawner as ThreadSpawner
    participant Child as Child ExecutionThreadState

    Parent->>Domain: start(ThreadObjectRef, target)
    Domain->>Domain: allocate logical id; insert Starting record
    Domain->>Spawner: spawn(context handle, child bootstrap, stack policy)
    alt spawn fails
        Spawner-->>Domain: error
        Domain->>Domain: remove record; release owned references
    else spawned
        Spawner-->>Domain: NativeThreadHandle
        Domain->>Domain: attach context-scoped handle
        Child->>Child: install ContextHandle + ThreadStateHandle
        Child->>Domain: publish native id and Running
        Child->>Child: execute against shared context heap/globals/cells
        Child->>Domain: publish result or panic; Finished/Failed
    end
```

The registry owns the retained Thread object and native handle from
registration until explicit retirement. A failed spawn rolls both back.
Publication from the child validates `ContextId + LogicalThreadId`; a process
handle alone is insufficient.

No global/cell snapshot is returned for merge. Shared state is synchronized at
its actual owner. The child's completion contains only result, exception, and
resource-retirement evidence.

## Join and quiescence contract

Join first proves that the caller and target belong to the same
`ThreadDomain`, rejects self-join, and then waits through the context-scoped
native handle. Completion advances one record to `Joined`; it does not merge
ambient registries into the joining thread.

`enumerate()` projects records in `Starting` or `Running`.
`active_count()` is the size of the same projection, including the context's
main logical thread while it is active. `main_thread()` returns the one stable
descriptor owned by the domain. `current_thread()` resolves the installed
`ExecutionThreadState`, not OS-thread TLS data copied outside the aggregate.

During `ExecutionContext::quiesce`, the domain:

1. closes admission to new children;
2. waits for every non-daemon child according to the runtime exit policy;
3. records or rejects any still-live daemon child explicitly;
4. joins/reaps every completed native handle, including never-user-joined
   children;
5. releases retained Thread object references exactly once;
6. proves the registry cannot publish after context retirement.

The final daemon/exit behavior is verified by the Tier 1 exit gate; silently
detaching a handle is not quiescence evidence.

## Hook policy

`ThreadDomain` owns `trace_default`, `profile_default`, and a
`HookGeneration`. A newly admitted child copies the current defaults into its
`ExecutionThreadState`. `settrace` and `setprofile` update the domain defaults
used by subsequent domain-created children.

The `*_all_threads` operations update the defaults and each active child slot
under the domain synchronization boundary. The update records a generation so
tests can prove that every targeted child observed the new policy. Hook
invocation and `TRACE_PROFILE_HOOK_ACTIVE` remain child-owned; one child's
callback cannot suppress another child's hook.

Hook values follow normal heap retain/release ownership. Replacing a default or
child slot releases its prior value only after no callback can still observe
it.

## `threading.local`

A `threading.local` Python object retains one stable `LocalObjectId`.
Attributes are stored in:

```text
ExecutionThreadState.local_values
    [LocalObjectId]
    [attribute name]
    -> MbValue
```

The object identity is shared; its attribute map is child-specific. Access
routes through the installed `ExecutionThreadState`. Child retirement drops
that child's maps. Object retirement invalidates its identity and removes or
logically tombstones matching entries without relying on a reusable raw
address.

Snapshotting, clearing, and restoring shared object fields is forbidden.

## Object-owned synchronization

Lock state moves into `ObjData::Lock`; barrier state moves into
`ObjData::Barrier`. Sharing the Python object shares the synchronized payload.
Object destruction releases the final payload reference through normal heap
lifetime.

Barrier identity is the Python object identity. Removing the process side
table removes both `BARRIERS` and `NEXT_BARRIER_ID`. No new global cleanup map
may replace them.

## Compatibility binding

Unchanged `mb_*` functions may use scoped TLS only to resolve:

```text
ContextHandle + ThreadStateHandle
```

The binding is stack-like and RAII-restored across normal return, error, and
panic. It stores no thread-domain data itself. A missing binding or a handle
whose context does not match the current domain fails explicitly in debug and
test builds.

## Invariants

1. Every logical child belongs to exactly one `ThreadDomain`.
2. A process service never owns or exposes another context's membership.
3. Join authority requires `ContextId + LogicalThreadId`, never raw object bits.
4. Logical and native thread identities are distinct values.
5. `enumerate()` and `active_count()` are projections of the same registry.
6. The main logical Thread object is stable within one context only.
7. Child execution shares context-owned Python heap/global/cell state without
   snapshot-and-merge ownership transfer.
8. Frame, current-thread, hook execution, and local-value state are child-owned.
9. `threading.local` values from two children cannot alias through object fields.
10. Hook defaults are context-owned; active hook execution is child-owned.
11. Every retained Thread object and native handle has one retirement path.
12. Context retirement prevents all later child publication.
13. Lock and barrier payload lifetime equals Python object lifetime.
14. Worker start performs no stdlib/native-catalog mutation.
15. Scoped TLS contains handles only and always restores its prior binding.

## Migration order

1. Finish the remaining #2968 inventory slices and close the exact-set parent.
2. Introduce the #2839 `ExecutionContext` and scoped context/thread-state
   handles without changing Python threading behavior.
3. Add `ThreadDomain`, child records, logical/native identity separation, and
   fail-closed cross-context lookup tests.
4. Move `LIVE_THREADS`, `MAIN_THREAD`, and `THREAD_HANDLES` into the domain;
   make enumerate/count/start/join share one lifecycle projection.
5. Bind workers to shared context-owned heap/global/cell state and remove
   snapshot-and-merge transfer.
6. Move hook defaults/active slots and `threading.local` values to their
   respective domain and child owners.
7. Embed lock/barrier state in object payloads and remove their side tables and
   barrier surrogate IDs.
8. Consume stack policy through `ThreadSpawner`, retire per-worker stdlib
   registration, and add quiescence/never-joined-handle retirement.
9. Run race, leak, multicore, and process-exit gates before Tier 1 closure.

Each source step is a separate bounded AGY ticket with frozen design inputs and
controller-owned verification. Removing one TLS/static declaration is not
acceptance unless the corresponding semantic oracle passes.

## Forbidden fixes

- Replacing the current tables with one process-global `ThreadManager`.
- Copying the whole `ExecutionContext` or its registries into every OS worker.
- Keeping snapshot-and-merge while adding locks around it.
- Using `MbValue::to_bits()` or object addresses as cross-context authority.
- Treating `get_native_id()` as an alias for the logical id.
- Making `*_all_threads` a caller-only TLS write.
- Keeping lock/barrier side tables with a best-effort cleanup sweep.
- Detaching unjoined handles without context-exit evidence.
- Re-registering stdlib/native facts in every worker.

## Verification surface

- Inventory: exactly 19 declarations, 16 production plus 3 test-only.
- Production digest:
  `10eb8c818ab0c717ffb7b2bdbfa870325f2e45df38c82f4b74c26fff9545ac5a`.
- Two contexts start children concurrently; neither can enumerate or join the
  other's child.
- `active_count()` equals the live projection throughout start, finish, and
  join transitions.
- Logical and native IDs are independently witnessed inside a real OS worker.
- A never-user-joined worker is reaped during context quiescence with no
  retained Thread-object or handle leak.
- Two children mutate shared globals/cells without join-order merge loss.
- Two children using one `threading.local` object observe isolated values.
- Hook default inheritance and `*_all_threads` propagation are witnessed across
  main and worker children; reentrancy suppression remains per child.
- Lock and barrier objects retire without persistent side-table entries or raw
  address reuse.
- Requested stack size reaches the actual OS-spawn path or fails explicitly as
  unsupported.
- Worker start leaves the sealed native catalog byte-for-byte unchanged.
- Missing/mismatched context or thread-state bindings fail closed.
- Snapshot rule: #2988 permits no AGY repository writes and no controller
  `apps/mamba/src/**` changes.
