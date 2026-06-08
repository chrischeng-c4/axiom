# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/async: coroutine surface probes (CPython 3.12 oracle)."""

import inspect
import types

# types exposes the coroutine type, and it is named "coroutine".
assert hasattr(types, "CoroutineType")
assert types.CoroutineType.__name__ == "coroutine"

# types.coroutine adapts a generator function into an awaitable.
assert callable(types.coroutine)

# Code-object flags distinguish coroutines from plain generators.
assert hasattr(inspect, "CO_COROUTINE")
assert hasattr(inspect, "CO_GENERATOR")


async def coro():
    return 1


# Calling an async function yields a coroutine object, not the result.
c = coro()
assert type(c) is types.CoroutineType
assert isinstance(c, types.CoroutineType)

# A coroutine object exposes the awaitable + frame introspection surface.
assert "__await__" in dir(c)
assert hasattr(c, "send")
assert hasattr(c, "throw")
assert hasattr(c, "close")
assert hasattr(c, "cr_code")
assert hasattr(c, "cr_frame")

# The function flagged as a coroutine; a plain def is not.
assert bool(coro.__code__.co_flags & inspect.CO_COROUTINE)
assert not (coro.__code__.co_flags & inspect.CO_GENERATOR)
assert inspect.iscoroutinefunction(coro)

c.close()


def plain():
    return 1


assert not (plain.__code__.co_flags & inspect.CO_COROUTINE)
assert not inspect.iscoroutinefunction(plain)

print("surface OK")
