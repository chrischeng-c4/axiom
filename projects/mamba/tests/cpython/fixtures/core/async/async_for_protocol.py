# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/async: the async-iteration protocol — __aiter__/__anext__,
StopAsyncIteration termination, and __anext__ return validation."""

import types


def run(coro):
    while True:
        try:
            coro.send(None)
        except StopIteration:
            return


class Done(Exception):
    pass


# StopAsyncIteration ends an `async for`; collected values match the source.
class Counter:
    def __init__(self, n):
        self.n = n
        self.i = 0

    def __aiter__(self):
        return self

    async def __anext__(self):
        if self.i >= self.n:
            raise StopAsyncIteration
        self.i += 1
        return self.i


collected = []


async def loop_counter():
    async for v in Counter(3):
        collected.append(v)
    raise Done


try:
    loop_counter().send(None)
    raise AssertionError("expected Done")
except Done:
    pass
assert collected == [1, 2, 3]


# An async iterator may subclass StopIteration; only StopAsyncIteration ends
# the loop (StopIteration must NOT terminate it).
class AIterStop(StopIteration):
    i = 0
    value = 42

    def __aiter__(self):
        return self

    async def __anext__(self):
        if self.i:
            raise StopAsyncIteration
        self.i += 1
        return self.value


sub_result = []


async def loop_sub():
    async for v in AIterStop():
        sub_result.append(v)
    raise Done


try:
    loop_sub().send(None)
    raise AssertionError("expected Done")
except Done:
    pass
assert sub_result == [42]


# An async iterator may subclass tuple and still drive `async for`.
class AIterTuple(tuple):
    i = 0

    def __aiter__(self):
        return self

    async def __anext__(self):
        if self.i >= len(self):
            raise StopAsyncIteration
        self.i += 1
        return self[self.i - 1]


tup_result = []


async def loop_tuple():
    async for v in AIterTuple([7, 8]):
        tup_result.append(v)
    raise Done


try:
    loop_tuple().send(None)
    raise AssertionError("expected Done")
except Done:
    pass
assert tup_result == [7, 8]


# A non-awaitable returned from __anext__ raises TypeError.
class BadANext:
    def __aiter__(self):
        return self

    def __anext__(self):
        return ()


async def loop_bad():
    async for _ in BadANext():
        pass


try:
    run(loop_bad())
    raise AssertionError("expected TypeError")
except TypeError as e:
    assert "async for" in str(e) and "__anext__" in str(e)

print("async_for_protocol OK")
