# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_concurrent_futures"
# subject = "cpython321.test_concurrent_futures"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_concurrent_futures.py"
# status = "filled"
# ///
"""cpython321.test_concurrent_futures: execute CPython 3.12 seed test_concurrent_futures"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_concurrent_futures.py — #3425 axis-1 stdlib concurrent.futures
# AssertionPass seed.
#
# Mamba-authored seed exercising the `concurrent.futures` module surface
# called out in the issue:
#   ThreadPoolExecutor submit/result, map, as_completed, wait, Future state.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr) + flag constants.
#   2. ThreadPoolExecutor.submit returns a Future; Future.result() with
#      timeout; Future.done()/cancelled() transitions.
#   3. ThreadPoolExecutor.map preserves input order and returns iterator.
#   4. as_completed yields futures in completion order; result() per yield.
#   5. wait(return_when=ALL_COMPLETED) splits the input set into
#      (done, not_done) once all complete.
#   6. wait(return_when=FIRST_COMPLETED) returns when at least one
#      future is done.
#   7. Future raised-exception state — result() re-raises the exception
#      and Future.exception() exposes it.
#
# Boxed-int dodge applied to count comparisons.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_concurrent_futures N asserts` to stdout.

import concurrent.futures as cf

_ledger: list[int] = []


# Module-level helpers — no closures (mamba top-level def quirk).
def _square(x: int) -> int:
    return x * x


def _identity(x: int) -> int:
    return x


def _boom() -> int:
    raise ValueError("intentional concurrent.futures probe")


# 1. Module identity + public surface.
assert cf.__name__ == "concurrent.futures", "concurrent.futures.__name__"
_ledger.append(1)
assert hasattr(cf, "ThreadPoolExecutor"), "exposes ThreadPoolExecutor"
_ledger.append(1)
assert hasattr(cf, "Future"), "exposes Future"
_ledger.append(1)
assert hasattr(cf, "as_completed"), "exposes as_completed"
_ledger.append(1)
assert hasattr(cf, "wait"), "exposes wait"
_ledger.append(1)
assert hasattr(cf, "ALL_COMPLETED"), "exposes ALL_COMPLETED"
_ledger.append(1)
assert hasattr(cf, "FIRST_COMPLETED"), "exposes FIRST_COMPLETED"
_ledger.append(1)
assert hasattr(cf, "FIRST_EXCEPTION"), "exposes FIRST_EXCEPTION"
_ledger.append(1)

# 2. ThreadPoolExecutor.submit + Future.result + done/cancelled.
_exec = cf.ThreadPoolExecutor(max_workers=2)
_fut = _exec.submit(_square, 7)
assert isinstance(_fut, cf.Future), "submit returns a Future"
_ledger.append(1)
# Bounded result() call — never block beyond timeout.
_val = _fut.result(timeout=5.0)
assert _val - 49 == 0, "Future.result == 49 for _square(7) (boxed-dodge)"
_ledger.append(1)
# After result(), done()==True, cancelled()==False, exception()==None.
assert _fut.done() == True, "Future.done() True after completion"
_ledger.append(1)
assert _fut.cancelled() == False, "Future.cancelled() False for completed future"
_ledger.append(1)
assert _fut.exception(timeout=1.0) is None, "Future.exception() None on success"
_ledger.append(1)

# 3. Executor.map — preserves input order.
_mapped = list(_exec.map(_square, [1, 2, 3, 4]))
assert _mapped == [1, 4, 9, 16], "map preserves input order and squares each"
_ledger.append(1)
assert len(_mapped) - 4 == 0, "map yields 4 results for 4 inputs (boxed-dodge)"
_ledger.append(1)

# 4. as_completed — yields each future once it finishes.
_futs = [_exec.submit(_identity, x) for x in (10, 20, 30)]
_seen: list[int] = []
for _f in cf.as_completed(_futs, timeout=5.0):
    _seen.append(_f.result(timeout=1.0))
# Same multiset as inputs (order is completion-order, not insertion-order).
assert sorted(_seen) == [10, 20, 30], "as_completed yields each input result"
_ledger.append(1)
assert len(_seen) - 3 == 0, "as_completed yields exactly 3 results for 3 futures"
_ledger.append(1)

# 5. wait(ALL_COMPLETED) — partition into (done, not_done).
_futs2 = [_exec.submit(_identity, x) for x in (100, 200, 300)]
_done, _not_done = cf.wait(_futs2, timeout=5.0, return_when=cf.ALL_COMPLETED)
assert isinstance(_done, set), "wait returns DoneAndNotDoneFutures with set members"
_ledger.append(1)
assert len(_done) - 3 == 0, "wait(ALL_COMPLETED): 3 futures done"
_ledger.append(1)
assert len(_not_done) == 0, "wait(ALL_COMPLETED): no futures left pending"
_ledger.append(1)
# Each done future has its result accessible.
_done_values = sorted(f.result(timeout=1.0) for f in _done)
assert _done_values == [100, 200, 300], (
    "wait(ALL_COMPLETED) done set carries the resolved results"
)
_ledger.append(1)

# 6. wait(FIRST_COMPLETED) — returns when at least one finishes.
_futs3 = [_exec.submit(_identity, x) for x in (1, 2, 3)]
_done3, _not_done3 = cf.wait(_futs3, timeout=5.0, return_when=cf.FIRST_COMPLETED)
# At least one completed (CPU work is trivial; usually all three by then).
assert len(_done3) >= 1, "wait(FIRST_COMPLETED) yields ≥1 done future"
_ledger.append(1)
# Drain the rest before tearing down the executor so we don't leak.
for _f in _not_done3:
    _f.result(timeout=5.0)

# 7. Future raised-exception state — result() re-raises; exception() exposes.
_fut_err = _exec.submit(_boom)
_raised = False
try:
    _fut_err.result(timeout=5.0)
except ValueError:
    _raised = True
assert _raised == True, "Future.result() re-raises the underlying exception"
_ledger.append(1)
# After completion, .exception() returns the raised instance.
_exc = _fut_err.exception(timeout=1.0)
assert isinstance(_exc, ValueError), "Future.exception() exposes the raised ValueError"
_ledger.append(1)
assert str(_exc) == "intentional concurrent.futures probe", (
    "Future.exception() preserves the original message"
)
_ledger.append(1)
assert _fut_err.done() == True, "errored future also reports done()==True"
_ledger.append(1)

_exec.shutdown(wait=True)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_concurrent_futures {len(_ledger)} asserts")
