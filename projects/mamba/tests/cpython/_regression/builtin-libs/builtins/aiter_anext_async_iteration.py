# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/builtins: aiter()/anext() drive async iteration."""

import asyncio


class Counter:
    def __init__(self, stop):
        self.i = 0
        self.stop = stop

    def __aiter__(self):
        return self

    async def __anext__(self):
        if self.i >= self.stop:
            raise StopAsyncIteration
        value = self.i + 10
        self.i += 1
        return value


async def drive_counter():
    it = aiter(Counter(2))
    first = await anext(it)
    second = await anext(it)
    fallback = await anext(it, 77)
    stopped = False
    try:
        await anext(it)
    except StopAsyncIteration:
        stopped = True
    return first, second, fallback, stopped


assert asyncio.run(drive_counter()) == (10, 11, 77, True)

try:
    aiter(object())
    raise AssertionError("expected TypeError")
except TypeError:
    pass

print("aiter_anext_async_iteration OK")
