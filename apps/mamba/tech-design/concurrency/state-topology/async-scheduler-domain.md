# Async scheduler state topology

Issue: #2994
Parent inventory: #2968
Source revision: `0d76cc0603`

This Stage 1 slice classifies timeout deadline state, waker links, timer
registrations, test-only gather synchronization, and the legacy GIL
compatibility cell declared by `runtime/async_task.rs`. It extends the
`AsyncDomain` contract from #2993 without changing `src/**`.

## Bounded contexts

```text
ExecutionContext
└── AsyncDomain
    ├── coroutines[CoroutineId]
    ├── tasks[TaskId]
    └── Scheduler
        ├── wakers[CoroutineId] -> TaskId
        └── timers[CoroutineId] -> TimerRegistration

ExecutionThreadState
└── async_frame
    └── await_deadlines
        └── AwaitDeadlineGuard

Test harness
└── gather incomplete synchronization hook

Removed compatibility state
└── GIL_HELD
```

Scheduler registrations belong to one `ExecutionContext.AsyncDomain`.
Deadline nesting belongs to the child execution frame currently performing the
await. Test instrumentation is not a production aggregate. An always
free-threaded runtime has no target GIL-state owner.

## Aggregate and values

| Type | Kind | Identity / value |
|---|---|---|
| `Scheduler` | `AsyncDomain` sub-aggregate | `ContextId` |
| `WakerRegistration` | entity/link | `ContextId + CoroutineId` |
| `TimerRegistration` | entity | `ContextId + CoroutineId + TimerGeneration` |
| `Deadline` | value | monotonic `Instant` |
| `AwaitDeadlineGuard` | child stack guard | prior + effective deadline |
| `TimerGeneration` | value | rejects delayed expiry after replacement |
| `GatherTestSignal` | test-only synchronization | no production identity |

A waker link is a typed context-local relation between an admitted coroutine
and task, not raw integer authority. A timer is a scheduler registration whose
lifetime is joined to the coroutine/task lifecycle. Neither may survive
context retirement.

## Frozen inventory

The four production identities have sorted newline SHA-256
`87664157834701600709b0c55db1827f68174954022c398e21c61df1a71154df`.
The one test-only identity has sorted newline SHA-256
`e693a6df4d56f27626b651eb9e7479de7a03a36a154e7ad7eb5c367de7405be5`.

| Current symbol | Current storage | Current role | Target owner / disposition |
|---|---|---|---|
| `AWAIT_DEADLINE` | TLS `Cell<Option<Instant>>` | effective nested await timeout | `ExecutionThreadState.async_frame.await_deadlines` |
| `GIL_HELD` | TLS `Cell<bool>` | non-synchronizing await compatibility flag | remove |
| `TIMERS` | process `LazyLock<MbRwLock<HashMap<u64, Instant>>>` | sleep coroutine deadlines | `ExecutionContext.AsyncDomain.Scheduler.timers` |
| `WAKERS` | process `LazyLock<MbRwLock<HashMap<u64, u64>>>` | coroutine-to-task scheduling links | `ExecutionContext.AsyncDomain.Scheduler.wakers` |
| `GATHER_OBSERVED_INCOMPLETE_HOOK` | test-only process `AtomicBool` | coordinates one gather test | discard from production topology |

The accepted selector evidence contains 25 distinct physical path/line rows and
26 identity occurrences:

| Identity | Occurrences |
|---|---:|
| `AWAIT_DEADLINE` | 4 |
| `GATHER_OBSERVED_INCOMPLETE_HOOK` | 4 |
| `GIL_HELD` | 4 |
| `TIMERS` | 8 |
| `WAKERS` | 6 |
| **Total** | **26** |

The architecture comment at `async_task.rs:13` is the sole multi-identity
physical row; it names both `WAKERS` and `TIMERS`.

## Current behavior and defects

### Deadline nesting is manual TLS stack emulation

`mb_await_with_timeout` computes an absolute deadline and installs the earlier
of the existing and new deadlines. After `mb_await` returns, it restores the
prior value. This preserves an outer timeout across nested awaits on the same
OS thread.

Mamba exceptions use a runtime side channel and return an `MbValue` normally,
so ordinary timeout and exception returns pass through the restore. A caught
Rust panic/unwind before the restore can leave the inner deadline installed
for a later call on the same worker because no RAII guard owns restoration.
Process abort terminates the process and therefore creates no observable
subsequent-call state.

TLS also binds deadline meaning to the current OS worker. If a suspended await
resumes on another worker, the effective deadline does not travel with the
execution frame.

### Wakers are process-global raw-ID links

`mb_orbit_register_waker` searches the process-global task map for a pending
task and inserts `coroutine_id -> task_id`. `mb_await` then reads every entry
in the process-global `WAKERS` map and schedules all referenced task IDs into
its local event loop.

This can import unrelated executions into one loop. A raw integer does not
prove that the coroutine and task share a context, nor that either entity is
still live.

The ordinary `mb_await` path removes its own waker entry before returning from
timeout handling. Runtime side-channel exceptions do not skip that removal.
Proven stale paths remain:

- direct registration followed by coroutine/task release without the matching
  `mb_await` cleanup;
- a caught Rust panic/unwind before removal;
- broad `cleanup_all_async`, which clears coroutines/tasks and resets current
  ID allocators but leaves wakers unchanged.

The existing waker test itself registers a link, verifies it, and releases the
coroutine without removing the link.

### Timers are process-global and reaped opportunistically

`mb_sleep` creates a coroutine and inserts its raw ID with an absolute
deadline. Every local `EventLoop::tick` scans the entire process timer map,
completes each expired raw coroutine ID, and removes the matching timer.

A later tick can eventually reap an abandoned entry after expiry. There is no
direct timer removal on task cancellation, coroutine release, or broad async
cleanup. Until another tick runs, the entry remains. Because current cleanup
also resets raw handle allocators, a stale deadline may be applied to a newly
allocated entity with the same integer.

Scanning a process-global timer map also allows one context's event loop to
complete another context's coroutine. An `RwLock` prevents map data races but
does not establish context authority.

### The GIL flag does not guard runtime state

MIR lowering emits `mb_gil_release` immediately before each `mb_await` and
`mb_gil_acquire` immediately after it. The runtime symbol catalog exports
those functions and `mb_gil_held`.

Their implementation only toggles or reads `GIL_HELD` in TLS. No interpreter
loop, allocator, registry, or shared-data access consults the flag to admit or
block a thread. It is not a lock and cannot make the process-global registries
safe.

The state identity is removed. A migration may temporarily keep ABI no-op
symbols while compiler-emitted calls are removed, but no replacement flag or
global GIL is introduced.

### Gather synchronization is test-only

`GATHER_OBSERVED_INCOMPLETE_HOOK` is compiled only for tests. The gather loop
stores `true` after observing incomplete work; one worker in
`test_gather_completed_coroutines` waits for that signal before completing a
coroutine.

It is discarded from production ownership. Tests that need the same ordering
after migration use a scoped harness synchronization object, not a
process-global runtime field.

## Scheduler registration contract

```text
WakerRegistration {
    context_id: ContextId,
    coroutine_id: CoroutineId,
    task_id: TaskId,
    generation: RegistrationGeneration,
}

TimerRegistration {
    context_id: ContextId,
    coroutine_id: CoroutineId,
    deadline: Instant,
    generation: TimerGeneration,
}
```

Registration validates that both endpoints are live members of the same
`AsyncDomain`. Replacement increments a generation so a delayed wake/expiry
cannot act on a newer registration for the same typed handle.

Normal task/coroutine cancellation, completion, explicit release, and failure
retire every dependent waker/timer link. Retirement is idempotent by typed
identity and generation; it does not rely on a future loop tick.

## Event-loop tick contract

```mermaid
sequenceDiagram
    participant Loop as Context event loop
    participant Scheduler as AsyncDomain.Scheduler
    participant Domain as AsyncDomain
    participant Child as ExecutionThreadState

    Loop->>Scheduler: snapshot_due(ContextId, now)
    Scheduler-->>Loop: typed timer claims
    loop each due timer
        Loop->>Domain: claim coroutine + timer generation
        alt retired/replaced/foreign
            Domain-->>Loop: ignore stale claim
        else valid
            Domain->>Scheduler: retire timer
            Loop->>Child: step/complete through coroutine claim
        end
    end
    Loop->>Scheduler: take_ready_wakers(ContextId)
    Scheduler-->>Loop: same-context typed task IDs
    Loop->>Domain: claim and poll tasks
```

Scheduler locks are released before stepping a coroutine, polling a task,
running Python code, invoking a JIT body, releasing a value, or sleeping. A
tick cannot enumerate, schedule, or complete another context's registrations.

The ready queue may remain local to an event-loop driver, but every queued
task ID is validated against its `ContextId` and current generation before
polling.

## Await deadline contract

`ExecutionThreadState.async_frame` owns a deadline stack:

```text
effective_deadline = min(parent_deadline, requested_deadline)
```

`AwaitDeadlineGuard` pushes the effective deadline and restores the exact
prior frame state in `Drop`. It survives normal return, Mamba side-channel
exception, and caught Rust unwind. If an await may migrate between OS workers,
the deadline travels with the logical child/frame handle rather than remaining
in worker TLS.

Timeout cancellation goes through the same `AsyncDomain` transition that
retires task, waker, and timer dependencies. It does not merely set an
exception flag and leave scheduler registrations for later cleanup.

## Retirement and cleanup matrix

| Identity | Normal lifecycle | Current `cleanup_all_async` | Target retirement |
|---|---|---|---|
| `AWAIT_DEADLINE` | manual install and restore | unchanged | RAII pop with child/frame scope |
| `GIL_HELD` | TLS toggle/read only | unchanged | remove state; retire lowering calls/ABI shims |
| `TIMERS` | insert on sleep; remove after observed expiry | unchanged | remove on expiry, cancel, release, failure, or context quiescence |
| `WAKERS` | insert on registration; selected await path removes | unchanged | remove on wake, cancel, release, failure, or context quiescence |
| `GATHER_OBSERVED_INCOMPLETE_HOOK` | test store/load/reset | unchanged | scoped test-harness signal only |

Context quiescence closes scheduler admission, invalidates outstanding
registration generations, drains or explicitly cancels live timers/wakers,
and proves both maps empty before dropping `AsyncDomain`. It never resets
process handle allocators and never touches another context.

## Migration order

1. Add context-tagged scheduler registration types and generation checks.
2. Route timer and waker lookup through `ExecutionContext.AsyncDomain`.
3. Join timer/waker retirement to task/coroutine cancellation, completion,
   explicit release, failure, and context quiescence.
4. Replace TLS deadline emulation with `AwaitDeadlineGuard` on child/frame
   state.
5. Remove `GIL_HELD`, compiler-emitted state toggles, and unnecessary runtime
   symbols while preserving any explicitly required transition ABI as no-op.
6. Replace the gather atomic with scoped test-harness synchronization.
7. Remove the broad test serialization lock only after the complete async
   aggregate and all other #2968 state families are context-isolated.

## Verification obligations

Later implementation tickets must prove:

- two contexts can register identical-shaped timer/waker workloads without
  cross-scheduling or cross-completion;
- cancel, completion, explicit release, failure, and quiescence leave no
  dependent timer or waker;
- a stale timer/waker generation cannot act on a replaced entity;
- nested deadlines choose the earliest deadline and restore the exact parent
  on normal return, side-channel exception, and caught Rust unwind;
- resuming on another worker observes the logical frame's deadline;
- no runtime shared-state access is gated by `GIL_HELD`, and removing it does
  not introduce a replacement global GIL;
- timer scans and waker scheduling never hold scheduler locks across user/JIT
  execution or sleep;
- #2839 remains blocked until the complete #2968 Stage 1 inventory closes.
