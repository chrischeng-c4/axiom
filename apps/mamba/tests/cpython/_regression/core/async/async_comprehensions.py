# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/async: comprehensions with await and async-for clauses."""

import types


def run(coro):
    yields = []
    while True:
        try:
            yields.append(coro.send(None))
        except StopIteration as ex:
            return (yields, ex.args[0] if ex.args else None)


async def val(x):
    return x


async def stream(it):
    for i in it:
        yield i


# `await` inside list/set/dict comprehensions.
async def await_comps():
    lst = [await c for c in [val(1), val(41)]]
    st = {await c for c in [val(1), val(41)]}
    dct = {await c: "a" for c in [val(1), val(41)]}
    return (lst, st, dct)


assert run(await_comps()) == ([], ([1, 41], {1, 41}, {1: "a", 41: "a"}))


# `async for` in list/set/dict comprehensions and async generator expressions.
async def async_for_comps():
    lst = [i + 1 async for i in stream([10, 20])]
    st = {i + 1 async for i in stream([10, 20])}
    dct = {i + 1: i + 2 async for i in stream([10, 20])}
    gen = (i + 1 async for i in stream([10, 20]))
    via_gen = [g + 100 async for g in gen]
    return (lst, st, dct, via_gen)


assert run(async_for_comps()) == (
    [],
    ([11, 21], {11, 21}, {11: 12, 21: 22}, [111, 121]),
)


# `async for` with an `if` filter.
async def filtered():
    return [i + 1 async for i in stream([10, 20, 30]) if i > 10]


assert run(filtered()) == ([], [21, 31])


# Mixed sync `for` and `async for` clauses in one comprehension.
async def mixed():
    return [
        i + 1
        for pair in ([10, 20], [30, 40])
        if pair[0] > 10
        async for i in stream(pair)
        if i > 30
    ]


assert run(mixed()) == ([], [41])


# Nested comprehensions, async on the inner axis.
async def nested():
    return [[i + j async for i in stream([1, 2])] for j in [10, 20]]


assert run(nested()) == ([], [[11, 12], [21, 22]])


# An exception raised inside the async generator propagates out of the
# comprehension that drives it.
async def boom():
    yield 1
    raise ValueError("boom")


async def propagates():
    return [i async for i in boom()]


try:
    run(propagates())
    raise AssertionError("expected ValueError")
except ValueError as e:
    assert "boom" in str(e)

print("async_comprehensions OK")
