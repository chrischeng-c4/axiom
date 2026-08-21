# test_asyncio.py — #3423 axis-1 stdlib asyncio AssertionPass seed.
#
# Mamba-authored seed exercising the `asyncio` module surface called
# out in the issue:
#   run / sleep / gather, Task creation, wait_for timeout,
#   Event / Lock / Queue, create_task.
#
# Surface coverage (each `asyncio.run(...)` driver is at module scope;
# top-level async defs serve as the helpers per the mamba top-level
# def() quirk in test_math.py).
#
# Mamba runtime gaps tracked separately:
#   * asyncio.gather SIGABRT under our stdlib stub
#   * await inside arithmetic expression (#3498)
#   * async generators (#3499)
#   * @asynccontextmanager (#3500)
#   * async for (#3501)
# These will surface as Fail outcomes on the mamba runner until the
# linked issues land — the seed itself is CPython-validated.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_asyncio N asserts` to stdout.

import asyncio

_ledger: list[int] = []


# Module-level async helpers — no closures.
async def _square(x: int) -> int:
    await asyncio.sleep(0)
    return x * x


async def _gather_three() -> tuple[int, int, int]:
    return tuple(await asyncio.gather(_square(2), _square(3), _square(4)))  # type: ignore[return-value]


async def _create_task_and_await() -> tuple[bool, int]:
    t = asyncio.create_task(_square(5))
    val = await t
    return (t.done(), val)


async def _slow() -> str:
    await asyncio.sleep(1.0)
    return "done"


async def _wait_for_timeout() -> bool:
    try:
        await asyncio.wait_for(_slow(), timeout=0.05)
    except asyncio.TimeoutError:
        return True
    return False


async def _event_cycle() -> tuple[bool, bool, bool]:
    e = asyncio.Event()
    initial = e.is_set()
    e.set()
    set_state = e.is_set()
    await e.wait()  # returns immediately when set
    e.clear()
    cleared = e.is_set()
    return (initial, set_state, cleared)


async def _lock_cycle() -> bool:
    lk = asyncio.Lock()
    async with lk:
        held = lk.locked()
    not_held = lk.locked()
    return held and not not_held


async def _queue_roundtrip() -> tuple[int, int, int, int]:
    q: asyncio.Queue[int] = asyncio.Queue()
    await q.put(10)
    await q.put(20)
    await q.put(30)
    size_full = q.qsize()
    first = await q.get()
    second = await q.get()
    third = await q.get()
    return (size_full, first, second, third)


# 1. Module identity + public surface.
assert asyncio.__name__ == "asyncio", "asyncio.__name__"
_ledger.append(1)
assert hasattr(asyncio, "run"), "exposes run"
_ledger.append(1)
assert hasattr(asyncio, "sleep"), "exposes sleep"
_ledger.append(1)
assert hasattr(asyncio, "gather"), "exposes gather"
_ledger.append(1)
assert hasattr(asyncio, "create_task"), "exposes create_task"
_ledger.append(1)
assert hasattr(asyncio, "wait_for"), "exposes wait_for"
_ledger.append(1)
assert hasattr(asyncio, "Event"), "exposes Event"
_ledger.append(1)
assert hasattr(asyncio, "Lock"), "exposes Lock"
_ledger.append(1)
assert hasattr(asyncio, "Queue"), "exposes Queue"
_ledger.append(1)
assert hasattr(asyncio, "TimeoutError"), "exposes TimeoutError"
_ledger.append(1)

# 2. asyncio.run on a coroutine returning int.
_result = asyncio.run(_square(7))
assert _result - 49 == 0, "asyncio.run(coro) returns the awaited value (boxed-dodge)"
_ledger.append(1)

# 3. asyncio.gather over three coroutines.
_a, _b, _c = asyncio.run(_gather_three())
assert _a - 4 == 0, "gather yields _square(2)==4"
_ledger.append(1)
assert _b - 9 == 0, "gather yields _square(3)==9"
_ledger.append(1)
assert _c - 16 == 0, "gather yields _square(4)==16"
_ledger.append(1)

# 4. asyncio.create_task — awaiting the task yields the result and
# done() flips True.
_done, _val = asyncio.run(_create_task_and_await())
assert _done == True, "create_task'd task is done() after await"
_ledger.append(1)
assert _val - 25 == 0, "task await result == _square(5) == 25"
_ledger.append(1)

# 5. asyncio.wait_for — timeout raises TimeoutError.
_timed_out = asyncio.run(_wait_for_timeout())
assert _timed_out == True, "wait_for(slow, 0.05) raises asyncio.TimeoutError"
_ledger.append(1)

# 6. asyncio.Event — initial unset → set → wait → clear cycle.
_init, _set_state, _cleared = asyncio.run(_event_cycle())
assert _init == False, "Event initial state is unset"
_ledger.append(1)
assert _set_state == True, "Event.set transitions to set"
_ledger.append(1)
assert _cleared == False, "Event.clear transitions back to unset"
_ledger.append(1)

# 7. asyncio.Lock — async-with grants ownership; lock released after.
_lock_ok = asyncio.run(_lock_cycle())
assert _lock_ok == True, "Lock acquired inside async-with; released after"
_ledger.append(1)

# 8. asyncio.Queue — FIFO ordering across put/get pairs.
_size, _v1, _v2, _v3 = asyncio.run(_queue_roundtrip())
assert _size - 3 == 0, "Queue.qsize() == 3 after 3 puts (boxed-dodge)"
_ledger.append(1)
assert _v1 - 10 == 0, "Queue first get == first put (FIFO)"
_ledger.append(1)
assert _v2 - 20 == 0, "Queue second get == second put"
_ledger.append(1)
assert _v3 - 30 == 0, "Queue third get == third put"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_asyncio {len(_ledger)} asserts")
