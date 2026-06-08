# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_async_iterators"
# subject = "cpython321.lang_async_iterators"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_async_iterators.py"
# status = "filled"
# ///
"""cpython321.lang_async_iterators: execute CPython 3.12 seed lang_async_iterators"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# lang_async_iterators.py — #3353 axis-1 lang async iterators
# (async for over user __aiter__/__anext__) AssertionPass seed.
#
# Mamba-authored seed exercising the async-iterator surface called out
# in the issue:
#   * __aiter__ + __anext__ protocol
#   * `async for x in source:` consumes asynchronously
#   * StopAsyncIteration terminates loop cleanly
#   * Exception from __anext__ propagates
#
# Contract placement: `spec/` — pins outcome Fail. Mamba runtime gap
# #3501 (`async for` over user class with __aiter__/__anext__ raises
# TypeError 'not iterable') blocks AssertionPass today. Once #3501
# lands and this seed flips to AssertionPass on mamba, drift detection
# prompts a
# `git mv spec/lang_async_iterators.py pass/lang_async_iterators.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. inspect.isasyncgen / isasyncgenfunction distinguish a hand-rolled
#      async iterator from an async generator function — the user class
#      is NOT an async generator.
#   2. `async for v in source:` collects values in order.
#   3. Empty source — StopAsyncIteration on first anext terminates the
#      loop with no iterations.
#   4. Direct anext() drive — values out, then StopAsyncIteration.
#   5. Exception from __anext__ propagates out of `async for`.
#   6. __aiter__ returns self — single-pass iterator pattern reusable
#      across `async for` calls (each call advances shared state, like
#      a one-shot iterator).
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: lang_async_iterators N asserts` to stdout.

import asyncio
import inspect

_ledger: list[int] = []


# Module-level user-defined async iterator (no closures — mamba quirk).
class _Counter:
    """Async iterator yielding 0..n-1."""

    def __init__(self, n: int) -> None:
        self.n = n
        self.i = 0

    def __aiter__(self) -> "_Counter":
        return self

    async def __anext__(self) -> int:
        if self.i >= self.n:
            raise StopAsyncIteration
        v = self.i
        self.i += 1
        return v


class _Empty:
    """Async iterator that yields nothing — StopAsyncIteration immediately."""

    def __aiter__(self) -> "_Empty":
        return self

    async def __anext__(self) -> int:
        raise StopAsyncIteration


class _Boom(Exception):
    """Sentinel exception raised from __anext__ — should propagate."""


class _Exploder:
    """Async iterator that raises _Boom on the third value."""

    def __init__(self) -> None:
        self.i = 0

    def __aiter__(self) -> "_Exploder":
        return self

    async def __anext__(self) -> int:
        if self.i == 2:
            raise _Boom("anext exploded at i=2")
        v = self.i
        self.i += 1
        return v


# Module-level coroutine helpers driving the async iterators (no nested defs).
async def _collect(source) -> list[int]:
    out: list[int] = []
    async for v in source:
        out.append(v)
    return out


async def _drive_anext_to_exhaust() -> tuple[int, int, int, bool]:
    src = _Counter(3)
    it = src.__aiter__()
    a = await it.__anext__()
    b = await it.__anext__()
    c = await it.__anext__()
    raised = False
    try:
        await it.__anext__()
    except StopAsyncIteration:
        raised = True
    return (a, b, c, raised)


async def _drive_exploder() -> tuple[list[int], bool, str]:
    out: list[int] = []
    boomed = False
    msg = ""
    try:
        async for v in _Exploder():
            out.append(v)
    except _Boom as e:
        boomed = True
        msg = str(e)
    return (out, boomed, msg)


# 1. User async iterator is NOT an async generator.
_src = _Counter(3)
assert not inspect.isasyncgen(_src), (
    "_Counter instance is not an async generator (hand-rolled __aiter__)"
)
_ledger.append(1)
assert not inspect.isasyncgenfunction(_Counter), (
    "_Counter class is not an async-gen function"
)
_ledger.append(1)
# __aiter__ exists and returns self.
assert hasattr(_src, "__aiter__"), "_Counter has __aiter__"
_ledger.append(1)
assert hasattr(_src, "__anext__"), "_Counter has __anext__"
_ledger.append(1)
assert _src.__aiter__() is _src, "_Counter.__aiter__ returns self"
_ledger.append(1)


# 2. async for collects values in order.
_collected = asyncio.run(_collect(_Counter(5)))
assert _collected == [0, 1, 2, 3, 4], (
    "async for over _Counter(5) collects [0,1,2,3,4]"
)
_ledger.append(1)
assert len(_collected) - 5 == 0, "async for yields 5 values (boxed-dodge)"
_ledger.append(1)


# 3. Empty source — async for terminates immediately with no iterations.
_empty = asyncio.run(_collect(_Empty()))
assert _empty == [], "async for over _Empty yields no values"
_ledger.append(1)
assert len(_empty) - 0 == 0, "_Empty yields 0 values (boxed-dodge)"
_ledger.append(1)


# 4. Direct anext() drive — values then StopAsyncIteration.
_a, _b, _c, _stopped = asyncio.run(_drive_anext_to_exhaust())
assert _a - 0 == 0, "first anext yields 0 (boxed-dodge)"
_ledger.append(1)
assert _b - 1 == 0, "second anext yields 1 (boxed-dodge)"
_ledger.append(1)
assert _c - 2 == 0, "third anext yields 2 (boxed-dodge)"
_ledger.append(1)
assert _stopped == True, "fourth anext raises StopAsyncIteration"
_ledger.append(1)


# 5. Exception from __anext__ propagates out of `async for`.
_collected_pre, _boomed, _msg = asyncio.run(_drive_exploder())
assert _collected_pre == [0, 1], (
    "_Exploder yields [0, 1] before raising on third anext"
)
_ledger.append(1)
assert _boomed == True, "_Boom exception propagated out of async for"
_ledger.append(1)
assert _msg == "anext exploded at i=2", "exception message preserved"
_ledger.append(1)


# 6. __aiter__ returns self — single-pass shared-state iterator.
_shared = _Counter(4)
_first_pass = asyncio.run(_collect(_shared))
assert _first_pass == [0, 1, 2, 3], "first pass collects [0,1,2,3]"
_ledger.append(1)
# Re-running async for on the same instance yields nothing because state
# is exhausted (single-pass iterator — like a generator object).
_second_pass = asyncio.run(_collect(_shared))
assert _second_pass == [], (
    "single-pass iterator: re-iteration yields nothing after exhaustion"
)
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: lang_async_iterators {len(_ledger)} asserts")
