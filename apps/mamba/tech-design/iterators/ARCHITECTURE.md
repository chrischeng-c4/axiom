# iterators — architecture (as-is, 2026-07-15)

Scope: the `__iter__`/`__next__` protocol, built-in + itertools lazy iterators, **generators (stackful
coroutines)**, **async coroutines (state-machine lowered)**, and async iteration. Two different
coroutine mechanisms live here — do not conflate them. Value model / refcount protocol
(`retain_if_ptr`), dunder dispatch, exception raising, and capture cells belong to their own domains and
are cross-referenced, not restated: `../memory/ARCHITECTURE.md`, `../object-model/ARCHITECTURE.md`,
`../exceptions/ARCHITECTURE.md`, `../closures/capture-and-scope.md`. Event-loop/tokio task driving is
`../concurrency/` (todo).

## Responsibilities

- Iterator protocol: `mb_iter` (get-iter dispatch), `mb_next`/`mb_next_or_stop`/`mb_next_raise`/`mb_next_default`, `mb_has_next` (`iter.rs`).
- Handle-based lazy iterators over built-ins (list/tuple/str/dict/set/range) and itertools (count/repeat/cycle/chain/groupby/map/filter/zip/enumerate/reversed) — the `IterKind` registry (`iter.rs:28,166`).
- User-defined generators (`def…yield`): stackful coroutines with private mmap stacks + register-swap (`generator.rs`).
- Async coroutines (`async def`/`await`/`async for`): compiler-lowered state machines driven by `mb_coroutine_step` (`async_rt.rs`, `async_task.rs`).
- The exhaustion↔exception boundary: StopIteration/StopAsyncIteration ↔ the `stop_iter_sentinel`; PEP 479; generator finalization exception preservation; `extract_items` propagating mid-iteration generator exceptions (hot path).

## Key structures & invariants

| Structure | Where | Invariant |
|---|---|---|
| `MbIterator{kind,index,exhausted,peeked}` | `iter.rs:16` | `peeked` caches one value fetched by `mb_has_next` so the following `mb_next` returns it without re-advancing — check-then-next correctness for ALL kinds. `advance_iter` (`:4337`) must drain `peeked` first. |
| `IterKind` 22 variants | `iter.rs:28` | Composite iters (Map/Filter/MapN/Zip/Enumerate/Chain) store inner(s) as **separate `inner_id` registry handles**, never embedded — advance releases the `ITERATORS` borrow before running user code that may re-enter (else reentrant `borrow_mut` panic). |
| `ITERATORS: HashMap<u64,MbIterator>` (thread-local) | `iter.rs:166` | Iterator handles are bare NaN-boxed ints ≥ `ITER_ID_BASE=1<<32` (`:163`); freed via `mb_iter_release`/`drain_iter_to_vec` (removes entry). Not GC/refcount objects; sources inside are retained (`retain_if_ptr` at `mb_iter`). |
| Handle ranges (disjoint, shared int tag space) | — | iterator `1<<32`, generator `GEN_ID_BASE=1<<39` (`generator.rs:397`), coroutine `CORO_ID_BASE=1<<40` (`async_rt.rs:162`). Keeps small ints from ever looking like handles. |
| `GenEntry{coro_ctx:Box,coro_stack,state,body_fn_addr,capture_context,…}` + `GENERATORS` | `generator.rs:216,351` | Generator = registry entry, **not** an `ObjKind` heap object — GC never traces it (see carve-out below). `coro_ctx` Boxed so its addr is stable across HashMap resize (raw ptr fed to `swap_context`). |
| `GenState` = Created/Suspended/Completed | `generator.rs:206` | The only "state machine" a generator has; resume position is the saved register context, not lowered blocks. |
| `CoroStack` = mmap + guard page, `swap_context` asm | `generator.rs:62,129` | 256K release / 1M debug stack; guard page `PROT_NONE`. `swap_context` has no prologue (swaps SP). |
| `CallerCtxStack` (16 slots) + `RUNNING_GEN_STACK` | `generator.rs:265,387` | `MAX_GEN_NESTING=16` yield-from depth cap (panics past). `RUNNING_GEN_STACK` makes re-entering a running ancestor raise ValueError, not corrupt. |
| `GEN_ACTIVE.last_resumed_id` resume cache | `generator.rs:343` | Skips the `GENERATORS` lookup on repeated resume of the same gen; **must be busted** on completion/throw/close (`u64::MAX` sentinel) or a stale ctx ptr is swapped into. |
| `MbCoroutine{state:u32,locals:Vec,resume_value,body_fn,…}` + `COROUTINES` (global RwLock) | `async_rt.rs:33` | Async coroutine locals are indexed (`get_local`/`set_local` `:1127/1145`); `state` selects the resume block; completed coroutines get `compact_completed_coroutine` (`:117`) to drop locals/body (RSS pin #1184). |
| `ObjKind::Generator=14` carve-out (#2182) | `rc.rs:445` (after `CodeObject=13`) | **Reserved, not implemented** — no heap generator type. Generators are registry-only (GC-invisible); `debug_validate` kind checks stay `>13`. Generator-object-shaped stdlib surfaces (csv.reader, re.finditer, glob.iglob, os.walk, xml iter) eager-materialize as `List`; lazy itertools instead live as `IterKind` handles. See `../memory/ARCHITECTURE.md` ObjData row. |

## Control flow

1. **for-loop / comprehension** (`hir_to_mir.rs:8377`): `mb_iter`(iter obj) → header block: `mb_next_or_stop` → `mb_is_stop_iter` branch to exit/body. Async form swaps in `mb_async_iter`/`mb_async_next_or_stop` + `emit_exception_propagate`.
2. **`mb_iter`** (`iter.rs:771`): weakref-proxy unwrap → known-handle passthrough (`iter(g) is g`) → file/ptr kind match → user `__iter__`/`__next__` dunder → build `IterKind`, insert, return int handle.
3. **`mb_next_or_stop`** (`iter.rs:2559`): single `borrow_mut` generator fast path (peek-take / resume / done→`stop_iter_sentinel`); else falls to `mb_has_next`+`mb_next`. `mb_next` (`:2305`) has the same GenFast enum, then out-of-line `advance_*_if_applicable` for callable/userdef/map/filter/mapn/enumerate/zip/chain/groupby, then in-line `advance_iter`. Every returned value gets `retain_if_ptr`.
4. **generator resume** (`generator.rs:resume_generator:706`): reject if already running → prep (init ctx on first `next()`) → set send xfer → `swap_context(caller→gen)`. Body runs via `gen_trampoline` (`:630`, calls `call_body_fn`); `yield` = `mb_generator_yield_value` (`:1261`) sets `yield_v`, swaps back. Completion vs yield signaled through `GEN_XFER.completion` (0=yielded), no HashMap read on the hot path.
5. **generator body lowering** (`hir_to_mir.rs:lower_generator_function:2464`): emits TWO MirBodies — wrapper (`fn_N`: `mb_generator_create`+`store_arg`+`capture_cells`, returns handle) and body (`fn_N_gen` sym `+3_000_000`). `Yield` → `mb_generator_yield_value` + `emit_post_yield_exc_check` (throw/close injection, `:13307`); `YieldFrom` → `mb_generator_yield_from`.
6. **async coroutine** (`hir_to_mir.rs:lower_async_function:2925`): body starts with a **dispatch block** (`build_async_resume_dispatch:2328`) reading `mb_coroutine_get_state_i64` and branching to a resume block per await id; `Await` (`:11865`) = `gil_release`→`mb_await`→`gil_acquire`→propagate→`emit_async_suspend_check` bumps `next_async_resume_state` + registers a resume block. `mb_coroutine_step` (`async_rt.rs:333`) re-invokes `body_fn(handle)` each step; the dispatch jumps back to where the coroutine suspended.
7. **`extract_items`** (`builtins/mod.rs:3902`, hot): direct data extraction for concrete containers; else `mb_iter`→`drain_iter_to_vec` fast batch (`iter.rs:555`); fallback loop guards `raised()` (`:3974`) — a pending non-StopIteration exception BREAKS and is left set so `set()/list()/sorted()/heapq.merge/…` propagate it, never swallow it as exhaustion.

## CPython-parity semantics

- **`iter(obj) is iter(obj)` for iterators/generators**: `mb_iter` returns a known handle unchanged; range objects clone to a fresh iterator (`clone_range_iterator`).
- **None-vs-exhaustion**: `None` is a valid yield, so exhaustion uses `MbValue::stop_iter_sentinel` (`TAG_STOP_ITER`), never `None`. `mb_is_stop_iter` (`:2637`) is the only for-loop stop signal.
- **PEP 479**: StopIteration escaping a generator body → `RuntimeError("generator raised StopIteration")` with the original as `__cause__` (`resume_generator:840`).
- **`send(non-None)` to a just-started generator** → `TypeError` (`mb_generator_send:896`); `next()`/`send(None)` skip that check.
- **throw/close** (`generator.rs:921/1024`): throw on exhausted re-raises the thrown exc; throw before first yield marks Completed + raises; `close()` injects `GeneratorExit`, and never surfaces StopIteration nor GeneratorExit (`:1109`); yielding after GeneratorExit → `RuntimeError("generator ignored GeneratorExit")`.
- **finalization exception preservation** (`resume_generator:834`): on completion, only synthesize StopIteration when no exception is pending — an uncaught body exception must survive to `throw()`/yield-from, not be overwritten.
- **dict/set mutation during iteration** → `RuntimeError` via `dict_iter_changed`/`set_iter_changed` (`iter.rs:242/258`) checked every `advance_iter` step (len+version snapshot); dict view iters retain the source, not degrade to a list.
- **`reversed(list)`** tracks live-list cursor semantics (append past tail ignored; truncate before cursor exhausts) via `Reversed{list_source}` (`iter.rs:97`).
- **`zip(strict=True)`** length mismatch → ValueError (PEP 618); **`iter(callable, sentinel)`** stops on `==` sentinel (PEP 234, `IterKind::Callable`).
- **async**: `async for` needs `__aiter__`/`__anext__`; StopAsyncIteration→`stop_iter_sentinel` (`mb_async_next_or_stop:894`); reusing an awaited coroutine / double-await → RuntimeError (`mb_await:740,756`).

## Known hazards

- **Reentrant `ITERATORS.borrow_mut`**: advancing a composite iter in-line while holding the borrow panics when inner runs user code — WHY every composite uses out-of-line `advance_*_if_applicable`; `advance_iter`'s Enumerate arm (`:4468`) is a dead safety-net that just exhausts.
- **Stale resume cache**: forgetting to bust `last_resumed_id` on throw/close/completion swaps into a freed/wrong `coro_ctx` — WHY every control path clears it (`generator.rs:827,971,1061`).
- **Generator handle is a bare int, not an object**: `del gen`/refcount won't finalize it; `mb_del_var` (`generator.rs:1159`) explicitly `mb_generator_close`s first; leaks show as unclosed stacks, not UAF.
- **Coroutine-stack overflow is silent**: body frames run on a fixed 256K stack with only a guard page — deep recursion inside a generator faults at the guard, not a clean RecursionError.
- **Yield-from nesting > 16** panics (`MAX_GEN_NESTING`) — a hard cap, not a Python-visible error.
- **`extract_items` swallowing exceptions**: reordering the `raised()` guards (`builtins/mod.rs:3974`) or treating a non-StopIteration mid-iteration exception as end-of-iter silently drops errors from `set()/sorted()/list(gen)`.
- **JIT ABI for generator/coroutine params**: body fns are called through an i64 trampoline; a Float/Int param declared F64 leaks raw bits (register-class mismatch) — params arrive NaN-boxed and are unboxed at entry (`hir_to_mir.rs:2494`, `:2947`).
- **`stop_iter_sentinel` ≠ `None`**: any new advance path returning `None` on exhaustion breaks for-loops that yield `None`.

## Extension points

| Adding | Where |
|---|---|
| New lazy iterator kind (itertools-like) | `IterKind` variant (`iter.rs:28`) + `advance_iter` arm (`:4337`); if it drives inner iters or runs user code, add an out-of-line `advance_*_if_applicable` and dispatch it in `mb_next`/`mb_has_next`/`mb_next_raise`/`mb_next_or_stop`, NOT the in-line arm. |
| New get-iter source type | branch in `mb_iter` (`iter.rs:771`) and `extract_items` (`builtins/mod.rs:3902`) — keep them in sync. |
| Real heap generator object (`ObjKind::Generator=14`) | close #2182: add the `ObjKind`/`ObjData` variant + `MbObject::new_generator`, extend GC `visit_contained`/`release_contained_values` (`../memory/ARCHITECTURE.md` extension row), retire the eager-`List` stdlib carve-out. |
| New generator control op (e.g. richer throw) | runtime in `generator.rs` + lowering seam `emit_post_yield_exc_check` (`hir_to_mir.rs:13307`). |
| New await/async-iter shape | `mb_await`/`mb_async_iter`/`mb_async_next_or_stop` (`async_task.rs`) + `emit_async_suspend_check`/`build_async_resume_dispatch` (`hir_to_mir.rs:12520/2328`). |

## EC surface

Per `../../external-contracts/README.md`. Proof-bearing fixture dirs (dimension prefixes `behavior/errors/type/surface/_regression/concurrency`):

- Generators: `behavior/core/{generators,generator_float_inference,yield_from}`, `_regression/core/{generators,generator_lifecycle,yield_from}`.
- Iterator protocol: `behavior/core/iter`, `behavior/builtin-libs/{iter,enumerate,range}`, `_regression/core/{iterator_protocol,iterators}`, `behavior/std-libs/{itertools,iterlen,collections}`, `surface/std-libs/itertools`.
- Async: `behavior/core/{coroutines,asyncgen}`, `behavior/std-libs/{asyncio,_asyncio,async_case,contextlib_async}`, `_regression/core/{async,async_await}`, `real_world/std-libs/asyncio`, `surface/std-libs/asyncio`.
- Cross-cutting: `behavior/core/sys_settrace` (gen frame events — shared with `../codegen/tracing-and-frames.md`); dict/set-mutation RuntimeError under `collections`/`iter`; full gate `cargo test -p mamba --release --test conformance` (oracle = python3.12 byte-diff).
