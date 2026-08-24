# Async domain state topology

Issue: #2993
Parent inventory: #2968
Source revision: `9583449fd1`

This Stage 1 slice classifies the coroutine registry, completion tombstones,
task registry, current-coroutine binding, ID allocation, and process-wide test
serialization declared by `runtime/async_rt.rs`. It defines their DDD
ownership and retirement contract without changing `src/**`.

Timer, waker, deadline, gather-test-hook, and GIL compatibility state declared
by `runtime/async_task.rs` belongs to the following inventory slice.

## Bounded contexts

```text
Process
└── ProcessHandleAllocator
    ├── CoroutineId
    └── TaskId

ExecutionContext
├── AsyncDomain
│   ├── coroutines[CoroutineId]
│   │   ├── lifecycle + exclusive execution claim
│   │   ├── frame/capture state
│   │   ├── owned Python values
│   │   └── JitModuleLease
│   ├── tasks[TaskId]
│   │   ├── lifecycle
│   │   ├── CoroutineId
│   │   └── owned result
│   └── completed_coroutines
├── shared Python heap
├── closure/cell domain
└── JitSession

ExecutionThreadState
└── async_frame
    └── current_coroutine
```

`AsyncDomain` belongs to exactly one `ExecutionContext`. A process allocator
issues collision-free typed handles but cannot look up, step, cancel, complete,
or retire a context's coroutine or task. The OS thread currently stepping a
coroutine is execution locality, not aggregate ownership.

`current_coroutine` is child execution state. It identifies the coroutine
whose frame is presently running on that child and is restored with stack
scope; it does not own the coroutine entity.

## Aggregate and entities

`AsyncDomain` is a sub-aggregate of `ExecutionContext`.

| Type | Kind | Identity / value |
|---|---|---|
| `AsyncDomain` | aggregate root | `ContextId` |
| `Coroutine` | entity | `ContextId + CoroutineId` |
| `Task` | entity | `ContextId + TaskId` |
| `CoroutineExecutionClaim` | lease | `ContextId + CoroutineId + ClaimGeneration` |
| `CoroutineId` | value | process-unique typed handle |
| `TaskId` | value | process-unique typed handle |
| `CoroutinePhase` | value | `Created`, `Suspended`, `Running`, `Completed`, `Closing`, `Retired`, `Failed` |
| `TaskPhase` | value | `Pending`, `Running`, `Completed`, `Cancelled`, `Retired`, `Failed` |
| `CompletedCoroutineSet` | aggregate value | context-scoped completion tombstones |
| `CurrentCoroutineGuard` | child stack guard | prior and installed `CoroutineId` |
| `OwnedMbValue` | value owner | retained Python value with exactly-once release |
| `CoroutineBodyRef` | executable lease | symbol identity + live `JitModuleLease` |

A task observes one coroutine by typed identity. It does not obtain authority
to use a raw coroutine integer from another context. A completed-coroutine
tombstone retains only typed identity and completion semantics, never the
discarded frame or executable address.

## Frozen inventory

The seven production identities have sorted newline SHA-256
`438a1ad4c3d4d7355b6c30a29e1039ec0d6b9d7c5d6ad733473f83e3d8c9613f`.
There are no test-only static declarations in the file.

| Current symbol | Current storage | Current role | Target owner / disposition |
|---|---|---|---|
| `ASYNC_STATE_TEST_LOCK` | process `Mutex<()>` | serializes whole script/test execution around shared async state | remove after context isolation |
| `COMPLETED_COROUTINES` | process `LazyLock<MbRwLock<CompletedCoroutines>>` | completed-ID range tombstones | `ExecutionContext.AsyncDomain.completed_coroutines` |
| `COROUTINES` | process `LazyLock<MbRwLock<HashMap<...>>>` | live coroutine entities | `ExecutionContext.AsyncDomain.coroutines` |
| `CURRENT_COROUTINE_ID` | TLS `Cell<Option<u64>>` | presently stepping coroutine | `ExecutionThreadState.async_frame.current_coroutine` |
| `NEXT_CORO_ID` | process `AtomicU64` | coroutine integer allocation | process `ProcessHandleAllocator` |
| `NEXT_TASK_ID` | process `AtomicU64` | task integer allocation | process `ProcessHandleAllocator` |
| `TASKS` | process `LazyLock<MbRwLock<HashMap<...>>>` | task entities and results | `ExecutionContext.AsyncDomain.tasks` |

The accepted cross-module selector evidence reconciles to 134 distinct
physical path/line rows and 139 identity occurrences:

| Identity | Occurrences |
|---|---:|
| `ASYNC_STATE_TEST_LOCK` | 10 |
| `COMPLETED_COROUTINES` | 12 |
| `COROUTINES` | 73 |
| `CURRENT_COROUTINE_ID` | 7 |
| `NEXT_CORO_ID` | 3 |
| `NEXT_TASK_ID` | 3 |
| `TASKS` | 31 |
| **Total identity occurrences** | **139** |

Multi-identity source rows count once in the physical denominator. In
particular, `async_rt.rs:163` names the three registries,
`async_task.rs:3` and `:13` name both live registries, and
`asyncio_mod.rs:1962` names `COROUTINES` and `TASKS`.

## Current behavior and defects

### Process-global registries merge unrelated executions

`COROUTINES`, `TASKS`, and `COMPLETED_COROUTINES` are visible to every OS
worker and every execution in the process. `cleanup_all_async` clears all
three without identifying which execution owns an entry. One script or test
can therefore erase another execution's live entities and completion history.

The current workaround is `ASYNC_STATE_TEST_LOCK`. The production driver at
`driver/mod.rs:294` holds it around a script's whole JIT execution and teardown.
Six direct tests acquire it separately:

- `async_task.rs:1469`;
- `asyncio_mod.rs:1965`, `:1992`, `:2049`, `:2075`, and `:2130`.

The declaration and comments are not acquisitions. A broad mutex prevents
test interference by serializing independent executions; it is not a
free-threaded ownership model.

### Handle resets can collide with live entities

`NEXT_CORO_ID` and `NEXT_TASK_ID` allocate process-visible integers, but
`cleanup_all_async` resets them to `CORO_ID_BASE` and `1`. If another execution
still holds a live handle, a later allocation may reuse that integer and bind
it to a different entity.

Typed process handles are never reset during process lifetime. Context
retirement removes the context mapping but does not make an old handle valid
for a new entity.

### Current coroutine is execution state, not registry state

`CURRENT_COROUTINE_ID` is saved and restored around
`mb_coroutine_step_with_post`; reads support current-coroutine completion and
suspension. The writes at `async_rt.rs:1451` and `:1453` are test evidence,
not production mutations.

The target uses an RAII `CurrentCoroutineGuard` installed on the active
`ExecutionThreadState`. Nested execution restores the exact prior value on
normal return, exception, and panic. Broad async cleanup never edits another
child's current frame.

### Coroutine ownership combines heap, closure, and executable lifetimes

`MbCoroutine` contains locals, result/pending/resume `MbValue`s, captured cell
context, module metadata, and an optional raw JIT body pointer. Tombstone and
explicit release paths manually release some stored values and discard frame
payload.

The raw function pointer does not prove that its JIT module is still mapped.
Every live or suspended target `Coroutine` therefore carries a
`CoroutineBodyRef` with a `JitModuleLease`. Retirement releases that lease
only after no child holds an execution claim.

Unsafe `Send`/`Sync` declarations plus an outer `RwLock` prove neither
value-lifetime safety nor executable-memory lifetime. Those contracts must be
represented by the entity and leases.

### Task result ownership is ambiguous

`MbTask.result` is a copied `MbValue`. Completion copies a value from a
coroutine into the task without acquiring a distinct retained reference.
`mb_task_result` and `mb_run_until_complete` retain a value for the return
path, while task removal and broad map clearing have no task-owned release
contract.

This is not safely reducible to a proven leak only. The same stored reference
may be observed through both coroutine and task registries; depending on which
path retires first, the current ambiguity can cause a stale/dangling alias,
double-ownership assumptions, or an unreleased reference. The target contract
is explicit:

1. publishing a heap value into `Task.result` retains one task-owned reference;
2. replacement retains the new value before publication;
3. the old value is released after readers can no longer observe it;
4. task retirement releases the remaining result exactly once;
5. a returned value follows one named borrowed-or-owned ABI rule.

The same explicit rule applies to coroutine result, pending-await, and resume
slots.

### Tombstones need bounded context lifetime

`COMPLETED_COROUTINES` is a range-compressed set. Completion inserts an ID;
explicit coroutine release removes it. Without explicit release, tombstones
persist until broad cleanup.

The target set is context-scoped and retires with the context. Its retention
policy must preserve post-completion `inspect`/already-awaited behavior without
retaining frame payload. Process handle uniqueness means clearing a context's
tombstones cannot make old IDs reusable.

## Coroutine claim and stepping contract

Only one `CoroutineExecutionClaim` may exist for a coroutine.

```mermaid
sequenceDiagram
    participant Child as ExecutionThreadState
    participant Domain as AsyncDomain
    participant Coro as Coroutine
    participant Body as User or JIT body

    Child->>Domain: claim(ContextId, CoroutineId)
    Domain->>Coro: validate phase; mark Running
    alt already running or foreign context
        Coro-->>Child: deterministic error
    else claim acquired
        Coro-->>Child: scoped frame + CoroutineBodyRef
        Child->>Child: install CurrentCoroutineGuard
        Note over Domain: registry lock is released
        Child->>Body: step/send/throw/close
        Body-->>Child: suspend, complete, or fail
        Child->>Domain: commit by id + claim generation
        Domain->>Coro: publish phase/results; release claim
        Child->>Child: restore prior current coroutine
    end
```

No aggregate registry lock is held across arbitrary Python/JIT execution.
Claim generation prevents a delayed completion from committing into a retired
or replaced entity. Panic unwinding restores child state and resolves the
claim to one defined failed state.

Different coroutines may execute concurrently when their explicit shared
dependencies permit it. Two workers cannot concurrently step the same
coroutine.

## Task lifecycle contract

Task creation allocates a process-unique `TaskId`, validates the referenced
`CoroutineId` in the same `ContextId`, and inserts `Pending` before scheduling.

```text
Pending -> Running -> Completed -> Retired
                    -> Cancelled -> Retired
                    -> Failed ----> Retired
```

Cancellation and completion publish through `AsyncDomain`, not raw map access.
`gather`, `run_until_complete`, and asyncio bridges consume task snapshots or
scoped entity guards. Removing a task:

1. prevents new readers;
2. waits for or invalidates outstanding guards by generation;
3. releases its owned result;
4. removes scheduler/waker/timer dependencies defined by the next slice;
5. records retirement evidence.

Task and coroutine retirement are separate transitions. Removing one never
silently asserts that the other's value ownership was released.

## Lock and reentrancy policy

The target may use synchronization inside `AsyncDomain`, but lock scope is
bounded to lookup, claim, and commit. It excludes:

- coroutine body execution;
- Python callbacks;
- JIT entry;
- scheduler polling that may re-enter async APIs;
- result destructors or value release that can call runtime code;
- context quiescence waits.

This prevents replacing the process-wide test lock with a context-wide
deadlock. Entity phase and claim generation carry the concurrency protocol;
mutex possession alone is not semantic ownership.

## Retirement and cleanup matrix

| Identity | Normal lifecycle | Current broad cleanup | Target retirement |
|---|---|---|---|
| `ASYNC_STATE_TEST_LOCK` | lock persists after each guard | unchanged | removed after context isolation |
| `COMPLETED_COROUTINES` | insert on completion; remove on explicit release | clear all ranges | drop only this context's tombstones |
| `COROUTINES` | tombstone/release removes entity and some stored values | clear all entries | quiesce claims; release values and JIT leases exactly once |
| `CURRENT_COROUTINE_ID` | scoped save/restore | unchanged | RAII child-frame restore; child retirement asserts empty |
| `NEXT_CORO_ID` | monotonic allocation | reset to `CORO_ID_BASE` | process-lifetime allocator; never reset |
| `NEXT_TASK_ID` | monotonic allocation | reset to `1` | process-lifetime allocator; never reset |
| `TASKS` | state updates; gather removes selected entries | clear all entries | retire context tasks and owned results exactly once |

`ExecutionContext::quiesce` closes async admission, stops scheduler publication,
resolves or rejects live task/coroutine claims, retires scheduler dependencies,
releases owned values and executable leases, and then drops the domain.
Quiescence never clears another context and never resets process allocators.

## Migration order

1. Introduce typed, non-resetting coroutine and task handle allocation.
2. Add `AsyncDomain` under `ExecutionContext` with context-tagged lookup.
3. Add `ExecutionThreadState.async_frame` and scoped
   `CurrentCoroutineGuard`.
4. Define retained ownership types for coroutine/task stored `MbValue`s and
   `CoroutineBodyRef` JIT leases.
5. Migrate all 134 current physical access rows through context/domain APIs.
6. Integrate the following async-task state slice for scheduler, timer, and
   waker retirement.
7. Replace broad cleanup with context quiescence and remove
   `ASYNC_STATE_TEST_LOCK`.

The lock cannot be removed before all three registries and their scheduler
dependencies are context-isolated. ID counter resets cannot remain as a
compatibility cleanup after typed allocation lands.

## Verification obligations

Later implementation tickets must prove:

- two contexts concurrently create, step, complete, and retire coroutines and
  tasks without cross-observation or cross-cleanup;
- handle IDs are never reused after context retirement or cleanup;
- concurrent step of one coroutine is rejected while distinct coroutines can
  progress in parallel;
- nested stepping restores the exact prior current-coroutine binding on
  success, exception, and panic;
- task/coroutine result replacement and retirement have balanced
  retain/release evidence;
- suspended coroutine executable leases prevent JIT module retirement;
- tombstones preserve post-completion behavior without retaining frames;
- context quiescence leaves no task, coroutine, timer, waker, retained value,
  or JIT lease;
- the seven current lock acquisitions are removed rather than moved to a new
  broad critical section;
- #2839 remains blocked until the complete #2968 Stage 1 inventory closes.
