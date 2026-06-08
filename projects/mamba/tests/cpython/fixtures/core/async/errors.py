# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/async: await / async-for / async-with error paths (CPython 3.12 oracle)."""

import types


def run(coro):
    while True:
        try:
            coro.send(None)
        except StopIteration as ex:
            return ex.args[0] if ex.args else None


# Awaiting a non-awaitable raises TypeError naming the offending type.
async def await_int():
    await 1


try:
    run(await_int())
    print("await_int: no_raise")
except TypeError as e:
    assert "int" in str(e) and "await" in str(e)
    print("await_int:", type(e).__name__)


# Awaiting an object with no __await__ raises TypeError naming the class.
class Plain:
    pass


async def await_plain():
    return await Plain()


try:
    run(await_plain())
    print("await_plain: no_raise")
except TypeError as e:
    assert "Plain" in str(e) and "await" in str(e)
    print("await_plain:", type(e).__name__)


# __await__ returning a non-iterator raises TypeError.
class BadAwait:
    def __await__(self):
        return 5


async def await_bad():
    return await BadAwait()


try:
    run(await_bad())
    print("await_bad: no_raise")
except TypeError as e:
    assert "__await__" in str(e) and "non-iterator" in str(e)
    print("await_bad:", type(e).__name__)


# async-for over an object lacking __aiter__ raises TypeError.
async def afor_no_aiter():
    async for _ in (1, 2, 3):
        pass


try:
    run(afor_no_aiter())
    print("afor_no_aiter: no_raise")
except TypeError as e:
    assert "async for" in str(e) and "__aiter__" in str(e)
    print("afor_no_aiter:", type(e).__name__)


# __aiter__ present but no __anext__ raises TypeError.
class NoANext:
    def __aiter__(self):
        return self


async def afor_no_anext():
    async for _ in NoANext():
        pass


try:
    run(afor_no_anext())
    print("afor_no_anext: no_raise")
except TypeError as e:
    assert "async for" in str(e) and "__anext__" in str(e)
    print("afor_no_anext:", type(e).__name__)


# async-with whose __aenter__ yields a non-awaitable raises TypeError.
class BadCM:
    def __aenter__(self):
        return 123

    def __aexit__(self, *exc):
        return 456


async def awith_bad():
    async with BadCM():
        pass


try:
    run(awith_bad())
    print("awith_bad: no_raise")
except TypeError as e:
    assert "async with" in str(e) and "__await__" in str(e)
    print("awith_bad:", type(e).__name__)


# A coroutine that raises StopIteration surfaces as RuntimeError, never
# silently terminating the coroutine.
async def raises_stop():
    raise StopIteration


try:
    run(raises_stop())
    print("raises_stop: no_raise")
except RuntimeError as e:
    assert "StopIteration" in str(e)
    print("raises_stop:", type(e).__name__)
