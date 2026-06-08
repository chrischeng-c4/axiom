# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/async: coroutine drive + await-value behavior (CPython 3.12 oracle)."""

import types


def run(coro):
    """Drive a coroutine with send(None); return (yields, return_value)."""
    assert coro.__class__ in {types.GeneratorType, types.CoroutineType}
    yields = []
    while True:
        try:
            yields.append(coro.send(None))
        except StopIteration as ex:
            return (yields, ex.args[0] if ex.args else None)


# A bare async function returns its value; nothing is yielded to the driver.
async def answer():
    return 42


assert run(answer()) == ([], 42)


# await on a coroutine forwards its return value to the caller.
async def inner():
    return 42


async def outer():
    return await inner()


assert run(outer()) == ([], 42)


# Doubly-nested await: outer awaits a value that is itself a coroutine.
async def make_coro():
    return inner()


async def double():
    return await (await make_coro())


assert run(double()) == ([], 42)


# await participates in arbitrary expressions (arithmetic, negation, tuples).
async def one():
    return 1


async def expr():
    return -await one() + await one() * 10


assert run(expr()) == ([], 9)


async def tup():
    return (await one(), "ham")


assert run(tup()) == ([], (1, "ham"))


# await as a keyword-argument value.
def ident(val):
    return val


async def kwarg():
    return ident(val=await one())


assert run(kwarg()) == ([], 1)

print("behavior OK")
