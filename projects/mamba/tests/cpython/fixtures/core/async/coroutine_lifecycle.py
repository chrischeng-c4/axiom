# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/async: coroutine lifecycle — one-shot semantics, send/close states,
frame teardown, and exception context."""

import types


# A just-started coroutine rejects a non-None send value.
async def empty():
    pass


c = empty()
try:
    c.send("spam")
    raise AssertionError("expected TypeError")
except TypeError as e:
    assert "just-started coroutine" in str(e)
c.close()


# A coroutine finishes by raising StopIteration carrying its return value;
# any subsequent send/throw raises "cannot reuse already awaited coroutine".
async def returns_spam():
    return "spam"


coro = returns_spam()
try:
    coro.send(None)
    raise AssertionError("expected StopIteration")
except StopIteration as ex:
    assert ex.args == ("spam",)
try:
    coro.send(None)
    raise AssertionError("expected RuntimeError")
except RuntimeError as e:
    assert "cannot reuse already awaited coroutine" in str(e)
try:
    coro.throw(Exception("wat"))
    raise AssertionError("expected RuntimeError")
except RuntimeError as e:
    assert "cannot reuse already awaited coroutine" in str(e)
coro.close()  # closing a finished coroutine is a no-op
coro.close()


# A coroutine cannot be awaited by two awaiters concurrently.
@types.coroutine
def nop():
    yield


async def suspends():
    await nop()


async def waiter(target):
    await target


running = suspends()
running.send(None)  # leave it suspended
try:
    waiter(running).send(None)
    raise AssertionError("expected RuntimeError")
except RuntimeError as e:
    assert "is being awaited already" in str(e)
running.close()


# Re-entering a coroutine that is currently executing raises ValueError.
async def reentrant():
    me.send(None)


me = reentrant()
try:
    me.send(None)
    raise AssertionError("expected ValueError")
except ValueError as e:
    assert "coroutine already executing" in str(e)


# Swallowing GeneratorExit during close raises RuntimeError.
async def swallows_exit():
    try:
        await nop()
    except GeneratorExit:
        await nop()


s = swallows_exit()
s.send(None)
try:
    s.close()
    raise AssertionError("expected RuntimeError")
except RuntimeError as e:
    assert "ignored GeneratorExit" in str(e)


# cr_frame is live before close and None afterwards.
f = empty()
assert f.cr_frame is not None
f.close()
assert f.cr_frame is None


# A StopIteration produced after the coroutine returns does not inherit the
# active exception as its __context__.
async def returns_value():
    return ValueError()


async def returns_in_except():
    try:
        raise KeyError
    except KeyError:
        return await returns_value()


def run(coro):
    while True:
        try:
            coro.send(None)
        except StopIteration as ex:
            return ex.args[0] if ex.args else None


assert run(returns_in_except()).__context__ is None

print("coroutine_lifecycle OK")
