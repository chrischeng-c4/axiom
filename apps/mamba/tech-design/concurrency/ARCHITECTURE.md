# concurrency — architecture (as-is, 2026-07-15)

No-GIL runtime services: real-OS-thread parallelism, the coroutine object model +
Rust-owned asyncio event loop, and the thread/process/futures surface shims. The
GIL is a no-op flag; mamba tells the parallelism story CPython-with-GIL cannot.
Coroutine *lowering* (HIR→MIR state machine) is owned by `codegen/` + `iterators/`
(todo); cross-thread capture by `../closures/capture-and-scope.md`; refcount
hand-off by `../memory/object-lifetime.md`. The target ownership boundary for
removing ambient runtime state and contracting `JIT_LOCK` is
`execution-context.md`.

## Responsibilities
- **Real parallelism** — `threading.Thread.start` + `asyncio.to_thread` run
  targets on real OS threads (`std::thread::spawn`); the "GIL" gates nothing.
- **Coroutine object model** — `MbCoroutine` create/step/send/throw/close, await
  dispatch, tombstoning of finished coroutines.
- **asyncio** — `asyncio.*` surface wired to a hand-rolled, single-threaded
  cooperative `EventLoop` (NOT tokio; NOT fd-selector-backed).
- **Sync primitives** — `Lock/RLock/Event/Condition/Semaphore/Barrier`: present,
  mostly no-op stubs.
- **multiprocessing / concurrent.futures / selectors** — in-process shims (no
  real fork, pool, or platform reactor).

## Key structures & invariants
- `async_rt.rs:MbCoroutine` (`:33`) — `state:u32`, `locals`, `body_fn` (compiled
  state-machine entry `extern "C" fn(i64)->i64`), `capture_context`,
  `pending_await`, `exhausted/running/awaiting`. `unsafe impl Send+Sync` valid
  **only** because every field is touched under the `COROUTINES` lock.
- Global lock-protected registries, all `LazyLock<MbRwLock<FxHashMap<u64,_>>>`:
  `COROUTINES` (`async_rt.rs:147`), `TASKS` (`:159`), `WAKERS`
  (`async_task.rs:698`), `TIMERS` (`:1239`). Async state is **process-global**,
  never per-loop, never freed on thread exit.
- ID spaces: coroutine handles are `int ≥ 2^40` (`CORO_ID_BASE`, `async_rt.rs:162`);
  task ids start at 1. `mb_await` uses the `≥2^40` gap to tell a coroutine handle
  from a plain int result.
- `CompletedCoroutines` (`async_rt.rs:59`) — interval-set tombstone; a re-await of
  a finished id raises `RuntimeError: cannot reuse already awaited coroutine`.
- `MbTask` (`async_rt.rs:131`) — `coroutine_id`, `done`, `cancelled`, `result`.
- `threading_mod.rs` worker handles: `THREAD_HANDLES` OnceLock<Mutex<map of
  `JoinHandle`>> (`:409`); a `Thread` starts **at most once** (`started` guard,
  `:708`).
- **Invariant** — one coroutine is awaited once: `awaiting`/`exhausted` snapshot
  guards (`async_task.rs:756-782`); second await → `RuntimeError`.

## Control flow
`asyncio.run` (`mb_run_until_complete`, `async_task.rs:1269`):
1. wrap main coro in `MbTask`, `EventLoop::schedule` it + all pending tasks.
2. loop ≤ **10_000** ticks: `gc_safepoint`; break if main task `done`; `tick`.
3. `EventLoop::tick` (`:520`): expire `TIMERS`→`mb_coroutine_complete`; drain
   `ready_queue`; per task `mb_coroutine_send(None)` if `pending_await` else
   `mb_coroutine_step`; `StopIteration`→clear; other exc→mark done; exhausted→
   store result else re-schedule.
4. return `main.result` (`retain_if_ptr`, ownership per `../memory/`).

`await` (`mb_await`, `async_task.rs:727`): generator→`await_iterator`; tombstoned→
`RuntimeError`; no coroutine + `__await__`→dunder; else `mb_orbit_schedule`
(=`create_task`) + `register_waker`, drive a local `EventLoop` ≤ **100_000**
ticks until `coro.exhausted` or `await_deadline` (→TimeoutError).

`to_thread` (`asyncio_mod.rs:895`): `make_future` + `spawn_to_thread_worker` (real
thread installs module/globals/cells, runs fn, stores future result/exc); returns
a coroutine that awaits the future until non-`PENDING`.
`Thread.start` (`threading_mod.rs:689`): guard `started`; snapshot
globals/active-cells/class-state/locals + excepthook; `std::thread::spawn`
`run_thread_target`; stash `JoinHandle`. `join` waits, merges worker globals,
flips `alive=false`.

## CPython-parity semantics
- **GIL** — `mb_gil_release/acquire` toggle a thread-local bool only
  (`async_task.rs:1332`); no serialization. Contract: a *single* container
  mutation is atomic via a per-object critical section; *compound* ops
  (`c[0]+=1`, check-then-act) are NOT atomic — caller must lock (see
  `../../tests/cpython/concurrency/CONVENTIONS.md`).
- **Coroutine reuse** — awaiting a finished/in-flight coro → `RuntimeError`
  (`coroutine is being awaited already` / `cannot reuse already awaited`), CPython-shaped.
- **`asyncio.sleep(t)`** — registers a `TIMER` deadline (`async_task.rs:1209`);
  loop yields 1 ms wall-clock while only timers/pending-futures remain;
  negative/nan/inf → `ZERO`.
- **asyncio primitives** — `Lock/Event/Semaphore/Queue.acquire/wait` return an
  already-**completed** coroutine (`completed_coroutine`); they never suspend or
  model contention. `Task` cancel = mark task `done+cancelled` + coro `exhausted`
  (`async_task.rs:76`); no `CancelledError` injected into the body.
- **`get_ident`** — 1 on main, distinct per live started `Thread`;
  `threading.local` collapses to a plain dict.
- **`ProcessPoolExecutor` aliases `ThreadPoolExecutor`** (no separate process,
  `concurrent_futures_mod.rs:396`); `submit`→lazily-deferred future evaluated on
  `.result()`; `map` runs synchronously inline.
- **`multiprocessing.Process`** runs the target in-process synchronously,
  `exitcode` always 0; `Pipe` is a same-process `VecDeque`
  (`multiprocessing_mod.rs`).

## Known hazards
- **Stale "tokio" header** — `async_rt.rs:3` / `async_task.rs:10` claim tokio +
  "multi-threaded executor"; coroutines actually run on a single-thread `Vec`
  ready-queue poll. Don't trust the doc-comment; tokio is unused here.
- **Bounded tick budgets** — 10_000 (run) / 100_000 (await); a coroutine needing
  more silently finishes *incomplete* with an `eprintln` warning → wrong `None`.
- **No-op sync primitives** — `threading.Lock/Barrier/Condition` don't
  synchronize; real races under `Thread.start` are guarded only by the
  container-level critical section, not user locks. `Barrier.wait` returns a
  rotating index, never rendezvous (`threading_mod.rs:19`).
- **`CONVENTIONS.md` is stale vs code** — it says `get_ident` is single-valued /
  "not parallel"; source now spawns real OS threads. Trust the source.
- **Snapshot globals** — worker threads copy globals/cells by snapshot;
  `Thread` merges worker globals only at `join`; a `to_thread` worker writes into
  a *replaced* namespace — cross-thread global visibility is not live.
- **asyncio semantics unproven** — all 41 `behavior/std-libs/asyncio` fixtures
  are `xfail` (auto-ported, promotion pending); the surface exists, the behavior
  is largely unverified.

## Extension points
- New `asyncio` API: dispatcher in `asyncio_mod.rs register()` attrs → an
  `async_task` rt fn. New coroutine primitive: `mb_coroutine_*` in `async_rt.rs`,
  driven by `EventLoop::tick`.
- Real fd loop: swap `EventLoop` poll for a `selectors_mod` `poll(2)` backend
  (today disjoint from asyncio). Real thread sync: replace `make_instance` no-op
  stubs in `threading_mod.rs` with handle-keyed `std::sync` types. Real
  subprocess: `multiprocessing_mod.rs defer_process` → `std::process`/fork.

## Gather invocation aggregate and quiescence boundary

`asyncio.gather(asyncio.to_thread(...), ...)` is one `GatherInvocation`
aggregate. The aggregate root owns the input-order slots, the task/coroutine
registrations created for those slots, and the transition from active work to
an ordered terminal result. A worker thread and its `Future` are implementation
details of a slot; they must not become independently terminal from the
aggregate's point of view.

The aggregate state is:

```
Created -> Scheduled -> AwaitingWorkers -> Collecting -> Quiescent
                                  \-> Failed
```

- `Created` captures a stable copy of the input coroutine handles. No borrowed
  container guard crosses the scheduling boundary.
- `Scheduled` creates exactly one task registration per input slot.
- `AwaitingWorkers` drives or waits for progress without starving worker
  threads. An iteration counter is diagnostic protection, not the semantic
  completion condition.
- `Collecting` begins only after every slot has one terminal outcome. Results
  are read in original input order.
- `Quiescent` means every result has an independent owner, task/coroutine
  bookkeeping created by this invocation is retired, and no worker owned by
  the invocation can publish a later result.
- `Failed` is an explicit exception or deadline outcome. It must not silently
  synthesize `None` for a pending slot.

### Invariants

1. **Exactly one terminal outcome per slot.** Each slot publishes either a
   value or exception once; pending is never interpreted as success.
2. **Publication precedes readiness.** A worker publishes `_result` or
   `_exception` before the synchronized `FINISHED` transition. The driver that
   observes `FINISHED` must observe the corresponding payload.
3. **Progress is scheduling-independent.** Once a worker has been spawned, a
   gather driver may yield or block for notification, but it cannot monopolize
   a lock or CPU path the worker needs to publish completion.
4. **Wall-clock liveness is observable.** A bounded external deadline either
   observes `Quiescent` or reports a non-success timeout with the set of pending
   slot identities. A fixed number of rapid polls cannot masquerade as a
   liveness proof.
5. **Ordering is not completion order.** Workers may finish in any order;
   `GatherInvocation` returns values in input order.
6. **Cleanup follows capture.** Result ownership is retained before
   registrations are tombstoned. Cleanup never races a still-publishing worker.

### Regression shape

The Tier-1 behavior probe performs five invocations with two CPU-bound
`to_thread` slots whose relative work sizes alternate. On the same release
binary it has completed in about 0.55 seconds and has also remained sleeping at
0% CPU until an external 30-second deadline killed it. The latter is a
`GatherInvocation` liveness failure even when a later retry is green. Acceptance
therefore requires deterministic state-machine canaries plus repeated
process-level runs; one successful retry is not evidence of quiescence.

## EC surface
- **Dedicated `concurrency/` dimension** (`tests/cpython/concurrency/`) —
  `atomicity/{list,dict,set}`, `safety/lock`, `primitives/threading`; self-checked
  `PASS`/`FAIL`, oracle = **free-threaded CPython 3.13t** (not GIL CPython).
  Contract + xfail reality in `CONVENTIONS.md`.
- **Module conformance** under `{surface,behavior,errors,real_world,_regression,
  type,security}/std-libs/{asyncio,threading,multiprocessing,concurrent_futures}`;
  asyncio behavior 41/41 xfail — surface proven, semantics not. Harness/verdict:
  `../../external-contracts/HARNESS.md`.
