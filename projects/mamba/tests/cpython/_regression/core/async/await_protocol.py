# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/async: the await protocol — __await__ iterators, send/throw value
passing, and how the coroutine wrapper forwards return values."""

import types


def run_send(coro):
    yields = []
    while True:
        try:
            yields.append(coro.send(None))
        except StopIteration as ex:
            return (yields, ex.args[0] if ex.args else None)


def run_await(coro):
    """Drive via coro.__await__(), alternating next() and send(None)."""
    aw = coro.__await__()
    yields = []
    i = 0
    while True:
        try:
            yields.append(next(aw) if i % 2 else aw.send(None))
            i += 1
        except StopIteration as ex:
            return (yields, ex.args[0] if ex.args else None)


# __await__ may return any iterator; its yields surface to the driver and
# its return becomes the await result.
class IterAwaitable:
    def __await__(self):
        return iter([52])


async def via_iter():
    return await IterAwaitable()


assert run_send(via_iter()) == ([52], None)


# __await__ implemented as a generator: yields suspend, return is the value.
class GenAwaitable:
    def __await__(self):
        yield 42
        return 100


async def via_gen():
    return await GenAwaitable()


assert run_send(via_gen()) == ([42], 100)


# A value sent into a suspended awaitable becomes the await expression result.
class FutureLike:
    def __await__(self):
        return (yield)


class Marker(Exception):
    pass


async def consume():
    try:
        return await FutureLike()
    except ZeroDivisionError:
        raise Marker


class Forward:
    def __init__(self, coro):
        self.coro = coro

    def __await__(self):
        return self.coro.__await__()


async def driver():
    return await Forward(consume())


c = driver()
c.send(None)
try:
    c.send("spam")
    raise AssertionError("expected StopIteration")
except StopIteration as ex:
    assert ex.args == ("spam",)

# A thrown exception propagates into the suspended await point.
c = driver()
c.send(None)
try:
    c.throw(ZeroDivisionError)
    raise AssertionError("expected Marker")
except Marker:
    pass


# The coroutine wrapper forwards a returned StopIteration as an object,
# and a returned tuple as a plain value — neither is unpacked.
async def returns_stop():
    return StopIteration(10)


yields, result = run_await(returns_stop())
assert isinstance(result, StopIteration) and result.value == 10


async def returns_tuple():
    return (10,)


assert run_await(returns_tuple()) == ([], (10,))

print("await_protocol OK")
