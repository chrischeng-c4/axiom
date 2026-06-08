# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/async: coroutine introspection, the types.coroutine adapter, and the
boundary between coroutines and plain generators."""

import inspect
import copy
import pickle
import traceback
import types


def run(coro):
    yields = []
    while True:
        try:
            yields.append(coro.send(None))
        except StopIteration as ex:
            return (yields, ex.args[0] if ex.args else None)


# types.coroutine turns a generator function into an awaitable; awaiting it
# drives the underlying generator, yields pass through, return is the value.
@types.coroutine
def two_steps():
    yield 1
    yield 2


async def uses_adapter():
    await two_steps()


f = uses_adapter()
assert f.send(None) == 1
assert f.send(None) == 2
try:
    f.send(None)
    raise AssertionError("expected StopIteration")
except StopIteration:
    pass


# A types.coroutine adapter can `yield from` a real coroutine and return its
# value as the await result.
@types.coroutine
def delegate():
    return (yield from inner_coro)


async def inner():
    return "spam"


inner_coro = inner()
assert run(delegate()) == ([], "spam")
inner_coro.close()


# A plain generator function is NOT awaitable.
def plain_gen():
    yield


assert not hasattr(plain_gen, "__await__")
assert not inspect.iscoroutinefunction(plain_gen)


# `yield from` of a coroutine inside a non-coroutine generator is rejected.
async def some_coro():
    return 10


co = some_coro()


def non_coro_gen():
    yield from co


try:
    list(non_coro_gen())
    raise AssertionError("expected TypeError")
except TypeError as e:
    assert "yield from" in str(e) and "coroutine" in str(e)
co.close()


# A coroutine's __await__ wrapper is an iterator and reports itself in repr.
async def empty():
    pass


c = empty()
aw = c.__await__()
assert "__iter__" in dir(aw)
assert "coroutine_wrapper" in repr(aw)
c.close()


# Coroutines and their await wrappers are neither copyable nor picklable.
c2 = empty()
try:
    copy.copy(c2)
    raise AssertionError("expected TypeError")
except TypeError:
    pass
try:
    pickle.dumps(c2)
    raise AssertionError("expected pickling failure")
except (TypeError, pickle.PicklingError):
    pass
c2.close()


# A coroutine that defines async def reports as a coroutine function even when
# compiled alongside ordinary one-line defs via exec.
src = "def plain(): return 7\nasync def acoro(): return 8\n"
ns = {}
exec(src, ns, ns)
assert ns["plain"]() == 7
assert not inspect.iscoroutinefunction(ns["plain"])
assert inspect.iscoroutinefunction(ns["acoro"])
ns["acoro"]().close()


# `throw` into an await chain preserves the Python stack depth: the frame
# resumed by throw sits at the same depth as the one suspended by send.
async def a():
    return await b()


async def b():
    return await stepper()


@types.coroutine
def stepper():
    try:
        yield len(traceback.extract_stack())
    except ZeroDivisionError:
        yield len(traceback.extract_stack())


chain = a()
depth_send = chain.send(None)
depth_throw = chain.throw(ZeroDivisionError)
assert depth_send == depth_throw
chain.close()


# `async` is a reserved keyword: using it as a name is a SyntaxError.
try:
    compile("async = 1", "<fixture>", "exec")
    raise AssertionError("expected SyntaxError")
except SyntaxError:
    pass

print("coroutine_introspection OK")
