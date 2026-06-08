# async def / await — #2799.
#
# Covers Python's minimal async function semantics without using any
# real-time sleeps, asyncio.sleep, or thread/IO primitives:
#
#   async def           defines a coroutine function. Calling it
#                       returns a coroutine OBJECT (not a value).
#   await coro          drives the awaited coroutine to completion
#                       and yields its return value.
#   asyncio.run         runs a coroutine on a fresh event loop.
#   asyncio.gather      schedules coroutines concurrently and returns
#                       the list of results.
#
# Clauses:
#   1. Calling an async function returns a coroutine object, NOT the
#      return value.
#   2. awaiting a coroutine returns the function's return value.
#   3. await-chain: nested async calls return their value to the
#      caller via await.
#   4. asyncio.run drives a top-level coroutine to completion.
#   5. asyncio.gather runs multiple coroutines concurrently and
#      returns the result list in input order.
#   6. Awaiting an already-awaited coroutine raises RuntimeError
#      (coroutines are one-shot).
#
# Every print line tagged `[async]` so failure output names async
# semantics. Fixture is NOT pre-marked xfail; if the runtime does
# not support async/await, parsing or execution fails loudly per the
# acceptance text.


import asyncio
from inspect import iscoroutine


async def deterministic(x):
    return x * 2


async def chain(x):
    a = await deterministic(x)
    b = await deterministic(a)
    return b


async def gather_demo():
    results = await asyncio.gather(
        deterministic(1),
        deterministic(2),
        deterministic(3),
    )
    return results


# Clause 1: async function call returns a coroutine object.
coro = deterministic(5)
print("[async] clause-1 is-coroutine:", iscoroutine(coro))
print("[async] clause-1 not-value:", coro != 10)
# Close the coroutine so we don't leak a never-awaited warning.
coro.close()


# Clause 2: await returns the function's return value.
async def clause2():
    return await deterministic(7)


print("[async] clause-2 awaited:", asyncio.run(clause2()))


# Clause 3: await chain.
print("[async] clause-3 chain:", asyncio.run(chain(3)))


# Clause 4: asyncio.run drives a top-level coroutine.
async def clause4():
    return "completed"


print("[async] clause-4 run:", asyncio.run(clause4()))


# Clause 5: asyncio.gather concurrency, input-order results.
print("[async] clause-5 gather:", asyncio.run(gather_demo()))


# Clause 6: re-awaiting a coroutine raises RuntimeError.
async def clause6():
    once = deterministic(11)
    first = await once
    try:
        # Re-awaiting the SAME coroutine object — Python rejects this
        # because coroutines are one-shot.
        _ = await once  # pyright: ignore[reportGeneralTypeIssues]
        return ("unexpected-no-error", first)
    except RuntimeError as exc:
        return (type(exc).__name__, first)


print("[async] clause-6 reawait:", asyncio.run(clause6()))
