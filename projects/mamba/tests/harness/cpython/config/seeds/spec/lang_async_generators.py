# lang_async_generators.py — #3351 axis-1 lang async generators
# (yield in async def) AssertionPass seed.
#
# Mamba-authored seed exercising the async-generator surface called out
# in the issue:
#   * `async def gen(): yield 1; yield 2` is async generator
#   * `async for` over async generator collects values
#   * `aclose()` closes async generator
#   * StopAsyncIteration semantics on exhaustion
#
# Contract placement: `spec/` — pins outcome Fail. Mamba runtime gap
# #3499 (calling an async generator function returns first yield value
# instead of an async-generator object) blocks AssertionPass today.
# Once #3499 lands and this seed flips to AssertionPass on mamba,
# drift detection prompts a
# `git mv spec/lang_async_generators.py pass/lang_async_generators.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Calling an async-generator function returns an async-generator
#      object — NOT the first yield value.
#   2. inspect.isasyncgenfunction recognises the function; the result
#      satisfies inspect.isasyncgen.
#   3. async for over the generator collects yielded values in order.
#   4. anext() returns the next value when awaited; raises
#      StopAsyncIteration when exhausted.
#   5. aclose() — closes the generator; subsequent anext raises
#      StopAsyncIteration.
#   6. async generator with arguments — preserves parameterised body.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: lang_async_generators N asserts` to stdout.

import asyncio
import inspect

_ledger: list[int] = []


# Module-level async generators — no closures (mamba top-level def quirk).
async def _gen_simple():
    yield 1
    yield 2
    yield 3


async def _gen_params(n: int):
    for i in range(n):
        yield i * 10


async def _collect_via_async_for() -> list[int]:
    out: list[int] = []
    async for v in _gen_simple():
        out.append(v)
    return out


async def _drive_anext_to_exhaust() -> tuple[int, int, int, bool]:
    g = _gen_simple()
    a = await anext(g)
    b = await anext(g)
    c = await anext(g)
    raised = False
    try:
        await anext(g)
    except StopAsyncIteration:
        raised = True
    return (a, b, c, raised)


async def _aclose_cycle() -> bool:
    g = _gen_simple()
    _v1 = await anext(g)  # consume one value first
    assert _v1 - 1 == 0
    await g.aclose()
    # After aclose, anext raises StopAsyncIteration.
    raised = False
    try:
        await anext(g)
    except StopAsyncIteration:
        raised = True
    return raised


async def _collect_params() -> list[int]:
    out: list[int] = []
    async for v in _gen_params(4):
        out.append(v)
    return out


# 1. Calling an async-gen function returns an async-generator object.
_gen_obj = _gen_simple()
assert inspect.isasyncgen(_gen_obj), (
    "calling async-gen function returns an async-generator object"
)
_ledger.append(1)
# Critically, NOT the first yield value (this is the #3499 bug).
assert _gen_obj != 1, (
    "async-gen call does NOT return the first yield value (#3499 regression check)"
)
_ledger.append(1)


# 2. inspect.isasyncgenfunction / isasyncgen.
assert inspect.isasyncgenfunction(_gen_simple), (
    "inspect.isasyncgenfunction recognises the async-gen function"
)
_ledger.append(1)
assert not inspect.isasyncgenfunction(_collect_via_async_for), (
    "regular async def is NOT an async-gen function"
)
_ledger.append(1)


# 3. async for collects values in order.
_collected = asyncio.run(_collect_via_async_for())
assert _collected == [1, 2, 3], (
    "async for over _gen_simple collects [1, 2, 3] in order"
)
_ledger.append(1)
assert len(_collected) - 3 == 0, "async for yields 3 values (boxed-dodge)"
_ledger.append(1)


# 4. anext() returns next value; StopAsyncIteration on exhaustion.
_a, _b, _c, _stopped = asyncio.run(_drive_anext_to_exhaust())
assert _a - 1 == 0, "first anext yields 1 (boxed-dodge)"
_ledger.append(1)
assert _b - 2 == 0, "second anext yields 2 (boxed-dodge)"
_ledger.append(1)
assert _c - 3 == 0, "third anext yields 3 (boxed-dodge)"
_ledger.append(1)
assert _stopped == True, "fourth anext raises StopAsyncIteration"
_ledger.append(1)


# 5. aclose() — subsequent anext raises StopAsyncIteration.
_aclose_ok = asyncio.run(_aclose_cycle())
assert _aclose_ok == True, (
    "aclose() closes the generator — subsequent anext raises StopAsyncIteration"
)
_ledger.append(1)


# 6. Parameterised async generator.
_params = asyncio.run(_collect_params())
assert _params == [0, 10, 20, 30], (
    "_gen_params(4) yields [0, 10, 20, 30] in order"
)
_ledger.append(1)
assert len(_params) - 4 == 0, (
    "parameterised async gen yields 4 values (boxed-dodge)"
)
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: lang_async_generators {len(_ledger)} asserts")
