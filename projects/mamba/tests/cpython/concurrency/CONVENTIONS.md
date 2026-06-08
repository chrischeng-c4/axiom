# `concurrency` dimension — thread-safety contract

The seventh conformance facet. Unlike every other dimension, concurrency
**cannot use "match CPython" as its oracle** — CPython-with-GIL appears
thread-safe for almost everything, but that is a *bytecode-atomicity artifact of
the GIL*, not a language guarantee. The real contract is **free-threaded CPython
(PEP 703, `python3.13t`)**, which removes the GIL and exposes what is actually
atomic.

Measure the live picture with `tools/concurrency_matrix.py` (writes the
`CONCURRENCY-CAPABILITY` block in the README). It auto-discovers a free-threaded
CPython via `uv python install cpython-3.13+freethreaded-<platform>`.

## The contract mamba must uphold

- **ATOMIC (must not corrupt or lose updates):** a *single* built-in container
  mutation — `list.append`/`pop`, `dict[k] = v`/`del dict[k]`, `set.add`/
  `discard`. Implemented with a lightweight per-object critical section taken
  **only on the mutation path**, with a biased/uncontended fast path — **not** a
  lock around whole compound expressions (that is both stricter than CPython and
  the expensive locking we explicitly reject).
- **NOT ATOMIC (races allowed; the caller must lock):** *compound* ops —
  `c[0] += 1` (load-add-store), check-then-act. Matching CPython means **not**
  over-promising here. A fixture for a compound op asserts the *locked* form is
  exact, never that the unlocked form is atomic.
- **ABSOLUTE (never CORRUPT):** no crash, no memory corruption, no impossible or
  wrong-deterministic value — for *any* pattern. This is the Rust `Send`/`Sync`
  guarantee that replaces the GIL.

## Verdict protocol

Concurrency execution is non-deterministic, so a fixture self-checks a
**deterministic property** and prints exactly one line:

```
concurrency: PASS
concurrency: FAIL: <detail>
```

The reference interpreters (CPython 3.12 GIL and 3.13t free-threaded) both print
`PASS`. A fixture that pins a current mamba gap carries an `xfail` naming the
divergence (it stays red until mamba upholds the contract).

## Layout

`{dimension}/{bucket}/{lib}/{case}.py` as everywhere else:

- `atomicity/{list,dict,set}/…` — single-mutation atomicity + no-corruption.
- `safety/{lock,…}/…` — caller-locked compound ops are exact.
- `primitives/{threading,…}/…` — `get_ident`, `Barrier`, `Event`, … semantics
  required before any of the above can be trusted under true parallelism.

## Current mamba reality (pinned as xfail)

mamba runs the `threading` API but is **not yet truly parallel**: a tight loop
does not preempt, `threading.get_ident()` returns one id for all threads,
`threading.Barrier.wait` is unimplemented, and a shared `dict`/`set` under
concurrent writes collapses to one thread's worth of entries (data corruption).
So mamba currently passes the atomicity fixtures it passes *by serialization*,
not by a real free-threaded guarantee — these fixtures become meaningful tests
the moment the GIL-equivalent is removed (capability C2).
