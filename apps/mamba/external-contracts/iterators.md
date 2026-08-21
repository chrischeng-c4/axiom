# iterators — external contract (as-is, 2026-07-15)

Domain map: `tech-design/iterators/ARCHITECTURE.md` (EC surface section). Verdict law: `HARNESS.md`.
Oracle = live python3.12 byte-diff; xfail = acknowledged gap, still contract (skip, never executed).
Scope: `__iter__`/`__next__` protocol + lazy `IterKind` iterators, generators (stackful coroutines),
async coroutines (state-machine lowered) + async iteration.

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

Live-counted 2026-07-15, `tests/cpython/` relative, `.py` = total fixtures, xfail = `# mamba-xfail` marker count.

**Generators & yield** (`def…yield` stackful coroutines, PEP 380 delegation):

| Dir | .py | xfail | Notes |
|---|---|---|---|
| `behavior/core/generators` | 34 | 32 | core generator semantics |
| `behavior/core/generator_float_inference` | 15 | 0 | numeric-inference interaction w/ generator locals |
| `behavior/core/yield_from` | 40 | 26 | PEP 380 delegation |
| `_regression/core/generators` | 30 | 0 | |
| `_regression/core/generator_lifecycle` | 4 | 0 | close/throw/GC lifecycle |
| `_regression/core/yield_from` | 4 | 0 | |

Subtotal: 127 fixtures, 58 xfail (69 live).

**Iterator protocol** (`mb_iter`/`IterKind` registry, itertools, dict/set view mutation-guard):

| Dir | .py | xfail | Notes |
|---|---|---|---|
| `behavior/core/iter` | 2 | 2 | |
| `behavior/builtin-libs/iter` | 55 | 46 | `mb_iter`/`__iter__`/`__next__` dispatch |
| `behavior/builtin-libs/enumerate` | 43 | 34 | |
| `behavior/builtin-libs/range` | 28 | 23 | range-iterator clone-on-`iter()` semantics |
| `_regression/core/iterator_protocol` | 8 | 0 | |
| `_regression/core/iterators` | 28 | 0 | |
| `behavior/std-libs/itertools` | 173 | 133 | count/repeat/cycle/chain/groupby/map/filter/zip/enumerate `IterKind` variants |
| `behavior/std-libs/iterlen` | 18 | 7 | `__length_hint__` |
| `behavior/std-libs/collections` | 144 | 99 | dict/set view iteration incl. mutation-during-iteration `RuntimeError` |
| `surface/std-libs/itertools` | 41 | 0 | |

Subtotal: 540 fixtures, 344 xfail (196 live).

**Async** (compiler-lowered state-machine coroutines, `async for`):

| Dir | .py | xfail | Notes |
|---|---|---|---|
| `behavior/core/coroutines` | 97 | 94 | `async def`/`await` lowering + dispatch |
| `behavior/core/asyncgen` | 76 | 76 | async generators |
| `behavior/std-libs/asyncio` | 41 | 30 | |
| `behavior/std-libs/_asyncio` | 3 | 0 | C-accelerator surface |
| `behavior/std-libs/async_case` | 13 | 13 | `IsolatedAsyncioTestCase` |
| `behavior/std-libs/contextlib_async` | 35 | 35 | |
| `_regression/core/async` | 8 | 0 | |
| `_regression/core/async_await` | 10 | 0 | |
| `real_world/std-libs/asyncio` | 1 | 0 | |
| `surface/std-libs/asyncio` | 113 | 0 | |

Subtotal: 397 fixtures, 248 xfail (149 live).

**Cross-cutting** (generator/coroutine frame trace events, shared ownership):

| Dir | .py | xfail | Notes |
|---|---|---|---|
| `behavior/core/sys_settrace` | 281 | 275 | gen/coroutine frame events; shared w/ `tech-design/codegen/tracing-and-frames.md` |

Subtotal: 281 fixtures, 275 xfail (6 live).

**Totals: 1,345 fixtures, 925 xfail (420 live).** C2-owned `bench/`/`perf/` companions (e.g.
`_regression/std-libs/{itertools,asyncio}/bench/`, top-level `perf/{coroutines,asyncio_tcp,async_generators,
async_tree,asyncio_websockets,asyncio_to_thread_scaling}.py`) are outside this table — C1-skipped per the
HARNESS.md bench/perf convention (no GitHub issue governs this; "#2239" does not resolve to a real issue — repo
max is #1780), see Verification.

## Negative contract — what must be REJECTED

Iterators owns no `type/` dir of its own (no `type/core/iter`, `type/core/generators`, etc. exist). Two families of
type-system wall dirs surface over this domain's positive coverage, owned by type-system per the README.md
dimension rule:

| Dir | .py | xfail | Walls |
|---|---|---|---|
| `type/std-libs/itertools` | 20 | 14 | `behavior/std-libs/itertools` |
| `type/std-libs/_asyncio` | 6 | 0 | `behavior/std-libs/_asyncio` |
| `type/std-libs/unittest_async_case` | 2 | 2 | `behavior/std-libs/async_case` |
| `type/std-libs/asyncio_*` (28 submodule dirs: base_events, base_subprocess, coroutines, events, exceptions, format_helpers, futures, graph, locks, proactor_events, protocols, queues, runners, selector_events, sslproto, staggered, streams, subprocess, taskgroups, tasks, threads, timeouts, tools, transports, trsock, unix_events, windows_events, windows_utils) | 275 | 27 | `behavior/std-libs/asyncio` |

Totals: 303 fixtures, 43 xfail — every xfail reason is "force-typed arg enforcement pending" (same pattern as
`exceptions.md`'s `type/std-libs/traceback`), tied to the standing force-typed-arg epic #861.

Excluded by name-collision (NOT this domain, despite "iter"/"generator"/"async" in the dirname):
`type/std-libs/{asynchat,asyncore}` (legacy callback-based socket async framework, no positive coverage under
this domain — a different stdlib module's surface) and `type/std-libs/{email_generator,email_iterators}`
(`email.generator`/`email.iterators` classes, owned by the email stdlib surface, not core generators/iterators).

## Known contract gaps

- **100% of this domain's 925 xfails carry only the two generic promotion markers** ("auto-ported CPython test;
  mamba promotion pending" / "auto-extracted CPython test; mamba promotion pending") — verified across every
  fixture dir above, no exceptions. Unlike object-model/exceptions (which already pulled a handful of named
  divergences out of their xfail pools), this domain has differentiated ZERO domain-specific divergences from the
  promotion backlog — any real behavior gaps hiding in the 925 are currently indistinguishable from routine
  un-triaged CPython-test ports. Un-xfail campaign: tracked: #1768.
- **`behavior/core/sys_settrace` is 98% xfail (275/281), but the mechanism the dominant cluster names is
  already fixed**: the cluster of 212/275 originally tracked as "settrace `'exception'` event emitted once at
  the raising frame instead of in every unwinding frame" was #1535, which is CLOSED — the fix landed in commit
  `ad60cc1a6` (confirmed present on this branch: `mb_traceback_notify_unwind_exception()` / `exception_notified`
  in `src/runtime/stdlib/traceback_mod.rs`). The 275 xfail markers here are stale/un-swept, not blocked on a
  still-open mechanism; re-sweeping this dir against a current build should shrink the xfail count
  substantially. The true residual gap is jump-in-trace `f_lineno` and opcode tracing, which remain unfixed and
  currently have no dedicated tracker — new finding, not filed.
- **Generator `ObjKind::Generator=14` carve-out** (`rc.rs:445`, confirmed reserved-not-implemented): generators are
  registry-only and GC-invisible (`GENERATORS` HashMap, not a heap object), so stdlib generator-shaped surfaces
  (`csv.reader`, `re.finditer`, `glob.iglob`, `os.walk`, xml iter) eager-materialize as `List` instead of staying
  lazy — a laziness/RSS gap, not currently a byte-diff failure (ordering is preserved), but blocks true generator
  semantics (e.g. `gen.close()`, infinite-source memory bound) on those surfaces. Cross-domain with
  `tech-design/memory/ARCHITECTURE.md` ObjData row. "#2182" (cited in `rc.rs:445` and
  `tech-design/memory/ARCHITECTURE.md:23,57`) does not resolve to a real GitHub issue (repo max is #1780) — it
  is a source-code-comment placeholder, not an actual tracker entry; new finding, not filed until a real issue
  is created.
- **Coroutine-stack overflow is silent** (spot-checked `generator.rs:62,129`): generator body frames run on a fixed
  256K-release/1M-debug mmap stack with only a `PROT_NONE` guard page — deep recursion inside a generator body
  faults at the guard page instead of raising a clean Python `RecursionError`. No existing fixture or tracker
  exercises this; plain finding, no tracked issue.
- **Yield-from nesting > 16 hard-panics** (spot-checked `generator.rs:260` `MAX_GEN_NESTING=16`, `CallerCtxStack`):
  CPython raises `RecursionError` on excessive `yield from`/generator-delegation depth; mamba panics the process
  past 16 nested frames — a hard cap, not a Python-visible error. No existing fixture or tracker exercises this;
  plain finding, no tracked issue.
- Force-typed-arg walls above (43 xfail) are the same epic #861 backlog already cited in `exceptions.md`
  for `type/std-libs/traceback` — no new tracker needed, just more surface area for the same campaign.

## Verification

```bash
# inner loop (seconds; runner-parity verdicts; paths relative to tests/cpython) — from apps/mamba/tests/harness/cpython/
python3 tools/sweep.py behavior/core/generators behavior/core/generator_float_inference behavior/core/yield_from \
  _regression/core/generators _regression/core/generator_lifecycle _regression/core/yield_from        # generators slice
python3 tools/sweep.py behavior/core/iter behavior/builtin-libs/iter behavior/builtin-libs/enumerate \
  behavior/builtin-libs/range _regression/core/iterator_protocol _regression/core/iterators \
  behavior/std-libs/itertools behavior/std-libs/iterlen behavior/std-libs/collections \
  surface/std-libs/itertools                                                                           # iterator-protocol slice
python3 tools/sweep.py behavior/core/coroutines behavior/core/asyncgen behavior/std-libs/asyncio \
  behavior/std-libs/_asyncio behavior/std-libs/async_case behavior/std-libs/contextlib_async \
  _regression/core/async _regression/core/async_await real_world/std-libs/asyncio \
  surface/std-libs/asyncio                                                                             # async slice
python3 tools/sweep.py behavior/core/sys_settrace                                                       # cross-cutting frame-trace slice
python3 tools/sweep.py type/std-libs/itertools type/std-libs/_asyncio type/std-libs/unittest_async_case \
  type/std-libs/asyncio_*                                                                              # type-wall slice (43 force-typed-arg xfails)
# cargo gate slice (datatest filter is a path substring, one dir per filter run)
cargo test -p mamba --release --test conformance -- core/generators
# C2: perf pins for this domain (external CPU/RSS ratio vs cpython baseline, floor=mem_floor=1.0)
cargo test -p mamba --release --test perf_pin -- perf_pin   # config/perf/pins/{asyncio_1416,asyncio_tcp_1183,coroutines_1184,itertools_1452}.toml
# manifests: config/manifests/core/generators.toml, config/manifests/std-libs/{asyncio,itertools}.toml;
# after fixture edits run tools/fixture_lint.py
# full C1 gate (the only progress signal, ~3 min; never concurrent with a cargo build) — per-fix evidence = before/after readings
cargo test -p mamba --release --test conformance
```
