# concurrency — external contract (as-is, 2026-07-15)

Domain map: `tech-design/concurrency/ARCHITECTURE.md` (EC surface section, no sibling topic docs exist yet).
Verdict law: `HARNESS.md`. Oracle = live python3.12 byte-diff for module conformance; the dedicated
`concurrency/` dimension instead self-checks a deterministic `concurrency: PASS/FAIL` line
(`tests/cpython/concurrency/CONVENTIONS.md`) that the harness still byte-diffs against a subprocess oracle
(see Known contract gaps — that oracle is plain CPython 3.12, not free-threaded 3.13t). xfail = acknowledged
gap, still contract (skip, never executed — HARNESS.md rot hazard).

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

**Dedicated `concurrency/` dimension** (live-counted 2026-07-15, `tests/cpython/` relative):

| Dir | .py | xfail | Covers |
|---|---|---|---|
| `concurrency/atomicity/dict` | 1 | 0 | single `dict[k]=v` atomicity |
| `concurrency/atomicity/list` | 1 | 0 | single `list.append` atomicity |
| `concurrency/atomicity/set` | 1 | 0 | single `set.add` atomicity |
| `concurrency/safety/lock` | 1 | 0 | caller-locked compound op exactness (see Known contract gaps) |
| `concurrency/primitives/threading` | 2 | 1 | `get_ident` distinctness (live); `Barrier.wait` rendezvous (xfail) |

Subtotal: 6 fixtures, 1 xfail (5 live). xfail marker here is the `[tool.mamba] xfail = "..."` PEP-723 field
(these hand-authored fixtures carry no separate inline `# mamba-xfail:` line, unlike the auto-ported dirs below).

**Module conformance** `std-libs/{asyncio,threading,multiprocessing,concurrent_futures}` across the byte-diff
dimensions (per-dir `# mamba-xfail:` counts):

| Module | surface | behavior | errors | real_world | _regression | security | Total .py | xfail |
|---|---|---|---|---|---|---|---|---|
| `asyncio` | 113 | 41 | 1 | 1 | 1 | — | 157 | 30 |
| `threading` | 46 | 112 | 9 | 1 | 1 | 1 | 170 | 85 |
| `multiprocessing` | 52 | 8 | 4 | 1 | 1 | — | 66 | 0 |
| `concurrent_futures` | 26 | 14 | 7 | 1 | 2 | — | 50 | 0 |

The `_regression` column includes 5 `bench/`-subdir fixtures (asyncio 1, threading 1, multiprocessing 1,
concurrent_futures 2) that are C2 pin-owned and C1-skipped (same HARNESS.md convention `exceptions.md` applies
to its own bench fixture, though that doc's "#2239" citation for the same convention does not resolve to a real
GitHub issue — #2239 does not exist, repo max is #1780) — these 5 don't actually run under the byte-diff gate,
so the "449/443 fixtures must RUN and byte-match" framing below over-counts by 5.

Subtotal: 443 fixtures, 115 xfail (328 live; 323 excluding the 5 bench/C1-skipped fixtures).

**Domain grand total**: 449 fixtures, 116 xfail, 333 live (328 excluding the 5 bench/C1-skipped fixtures).

Adjacent, not counted here: `*/std-libs/selectors` (surface/behavior/errors/real_world/_regression/type) — named
as an owned in-process shim in ARCHITECTURE.md's Responsibilities but absent from its EC-surface module list; see
Known contract gaps.

## Negative contract — what must be REJECTED

Concurrency owns no `type/` wall dimension of its own — no `concurrency` row exists in either place a wall
dimension would be declared (README.md's domain map, ARCHITECTURE.md's EC surface). Walls over this domain's
module surface belong to type-system per the dimension rule:

| Wall dir group | Live .py | xfail | Notes |
|---|---|---|---|
| `type/std-libs/asyncio_*` (+ `_asyncio`) | 281 | 27 | 29 submodule-split dirs; `asyncio_graph` is 4/4 (100%) xfail |
| `type/std-libs/multiprocessing_*` | 249 | 30 | 20 submodule-split dirs |
| `type/std-libs/concurrent_futures_*` | 23 | 6 | `__base`/`interpreter`/`process`/`thread` |
| `type/std-libs/threading` (+ `_threading_local`) | 31 | 10 | flat, not submodule-split |

Total walls over this domain: 584 fixtures, 73 xfail (511 live-enforcing). `type/std-libs/selectors` (8 .py, 0
xfail) sits adjacent with the same ownership ambiguity noted above. Runtime rejects belonging to this domain
(non-BaseException matcher, bad primitive args, etc.) are proven positively inside the 21 `errors/std-libs/*`
fixtures counted above, not as walls.

## Known contract gaps

- **Positive-count doc drift**: ARCHITECTURE.md's hazards line states "all 41 `behavior/std-libs/asyncio`
  fixtures are xfail"; live count today is 41 total / 30 xfail / **11 non-xfail** hand-authored fixtures already
  landed and passing (`gather_returns_results_in_order.py`, `queue_preserves_fifo_order.py`,
  `lock_serializes_critical_section.py`, `semaphore_caps_concurrency.py`,
  `task_cancellation_raises_cancelled_error.py`, `task_exception_propagates_on_await.py`,
  `event_wakes_waiter_after_set.py`, `future_cancelled_before_await_raises.py`,
  `future_result_when_not_done_raises.py`, `run_executes_coroutine.py`, `wait_for_timeout_raises_timeout_error.py`).
  The doc undercounts proven positive coverage; the remaining 30 xfail (auto-ported CPython test port) are
  un-xfail-campaign candidates. tracked: #1768.
- **Bounded tick budgets are unfixtured**: `mb_run_until_complete` caps at 10,000 ticks (`async_task.rs:1297`);
  `mb_await` caps at 100,000 (`async_task.rs:807,987,1030,1167`); overflow silently finishes the coroutine *incomplete*, warning only via `eprintln`
  (stderr) — the byte-diffed stdout only catches this if truncation happens to change printed output. No fixture
  across any of the 449 counted above deliberately approaches either ceiling.
- **`safety/lock` fixture is fragile-by-construction, not evidence Lock synchronizes**:
  `threading_mod.rs:925-934` (`lock_cm_enter`/`lock_cm_exit`) are unconditional no-ops — confirmed by reading the
  source, no mutex, no blocking, matching ARCHITECTURE.md's "no-op sync primitives" hazard. Yet
  `concurrency/safety/lock/counter_under_lock_is_exact.py` is live (not xfail) and asserts a 4-thread
  `counter[0]+=1`×1000 compound op under `with lock:` is exact every time. It currently passes on a narrow
  race-window (small N·K, fast in-process ops), not because the primitive upholds the contract — same
  fragile-proof class as `memory.md`'s anchor family. tracked: #1772.
- **Concurrency dimension's stated oracle is aspirational, not what the gate runs**: `CONVENTIONS.md` names the
  reference as free-threaded CPython 3.13t; `runner.rs`'s `harness!` discovery (l.629) and `python3_bin()`
  resolution apply the identical plain-CPython-3.12 oracle to `concurrency/` fixtures as every other dimension —
  there is no dimension-conditional oracle switch. The 3.13t comparison only happens in the separate, non-gating
  `tools/concurrency_matrix.py` (writes the README `CONCURRENCY-CAPABILITY` block). So
  `cargo test --test conformance` only proves mamba's self-check print agrees with a GIL CPython 3.12's self-check
  print (always `PASS`) — it does not exercise the free-threaded contract that gives these fixtures meaning.
- **EC-map path drift, whole-domain**: `external-contracts/README.md`'s Domain contract map table carries no
  `concurrency` row at all (only type-system/object-model/memory/exceptions/closures/stdlib); ARCHITECTURE.md's
  own EC surface bullet also omits `selectors` from its module-conformance list even though Responsibilities
  claims it ("multiprocessing / concurrent.futures / selectors — in-process shims"). Same class as
  object-model.md's descriptor-path drift finding. No existing GitHub issue covers this specific path-drift
  (#1771 is unrelated — runner-verdict sidecar/vacuous-walk scope); new finding, not filed.
- **`asyncio_graph` type wall is fully unenforced**: 4/4 `type/std-libs/asyncio_graph/*_wrong.py` are xfail — a
  std-libs surface fully xfailed, already named in `type-system.md`'s own gap list. tracked: #1768.

## Verification

```bash
# focused inner loop (seconds; runner-parity verdicts, shared oracle cache; set MAMBA_BIN after a release build)
# — from apps/mamba/tests/harness/cpython/ (sweep.py resolves relative paths against tests/cpython/
# already, so args must NOT be prefixed with tests/cpython/ — doing so double-nests and hard-fails)
python3 tools/sweep.py concurrency \
  {surface,behavior,errors,real_world,_regression}/std-libs/{asyncio,threading,multiprocessing,concurrent_futures} \
  security/std-libs/threading
# cargo gate slice (datatest filter is a path substring; trailing slash avoids matching type/std-libs/asyncio_*)
cargo test -p mamba --release --test conformance -- concurrency/
cargo test -p mamba --release --test conformance -- std-libs/asyncio/
# non-gating capability measurement (writes the CONCURRENCY-CAPABILITY README block vs free-threaded 3.13t)
python3 tests/harness/cpython/tools/concurrency_matrix.py
# full C1 gate (~3 min; this domain's slice rides inside it) — never concurrent with a cargo build
cargo test -p mamba --release --test conformance
```
