# lang_async_await.py — #3350 axis-1 async def + await on coroutine seed.
#
# Exercises:
#   1. `async def` defines a coroutine function; `asyncio.run(coro)`
#      executes it to completion and returns the value
#   2. `await coro` inside another `async def` body resolves the value
#   3. Chained awaits — sequential await on multiple coros
#   4. Nested awaits — multi-level await chain
#   5. Coroutine returning a non-int (dict literal)
#   6. Coroutine raising an exception — propagated out of asyncio.run
#
# Mamba quirks (tracked separately):
#   * asyncio.gather SIGABRT under our stdlib stub
#   * async generators (#3499)
#   * async for over user class (#3501)
#   * @asynccontextmanager (#3500)
#   * await inside arithmetic expression (#3498)
#
# Contract: AssertionError → Fail; MAMBA_ASSERTION_PASS → AssertionPass.
import asyncio

_ledger: list[int] = []

# (1) async def returning an int + asyncio.run dispatching it
async def _hello():
    return 42

assert asyncio.run(_hello()) - 42 == 0, "asyncio.run(async def returning int)"
_ledger.append(1)

# (2) await on coroutine resolves the value
async def _double(x):
    return x * 2

async def _await_once():
    val = await _double(5)
    return val

assert asyncio.run(_await_once()) - 10 == 0, "single await resolves coro value"
_ledger.append(1)

# (3) Chained awaits — sequential await on the same coro
async def _await_chain():
    a = await _double(5)
    b = await _double(a)
    return b

assert asyncio.run(_await_chain()) - 20 == 0, "chained awaits 5 → 10 → 20"
_ledger.append(1)

# (4) Nested awaits across multiple coros
async def _nested():
    val = await _hello()       # 42
    val2 = await _double(val)  # 84
    val3 = await _double(val2) # 168
    return val3

assert asyncio.run(_nested()) - 168 == 0, "3-level await chain hello→168"
_ledger.append(1)

# (5) Coroutine returning a non-int container
async def _make_dict():
    return {"x": 1, "y": 2}

_d = asyncio.run(_make_dict())
assert _d == {"x": 1, "y": 2}, f"async returning dict, got {_d!r}"
_ledger.append(1)

# (6) Coroutine returning a list
async def _make_list():
    return [10, 20, 30]

_lst = asyncio.run(_make_list())
assert _lst == [10, 20, 30], f"async returning list, got {_lst!r}"
_ledger.append(1)

# (7) Coroutine raising — propagates out of asyncio.run
async def _raises_error():
    raise ValueError("oops")

_caught = None
try:
    asyncio.run(_raises_error())
except ValueError as e:
    _caught = str(e)

assert _caught == "oops", (
    f"ValueError propagates from async def to asyncio.run caller, got {_caught!r}"
)
_ledger.append(1)

# (8) Coroutine taking + returning a string
async def _greet(name):
    return f"hello, {name}"

_g = asyncio.run(_greet("mamba"))
assert _g == "hello, mamba", f"async f-string return, got {_g!r}"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_async_await {sum(_ledger)} asserts")
