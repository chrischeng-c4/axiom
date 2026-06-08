# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "asyncgen"
# dimension = "behavior"
# case = "async_gen_asyncio_test__test_async_gen_asyncio_aclose_06"
# subject = "cpython.test_asyncgen.AsyncGenAsyncioTest.test_async_gen_asyncio_aclose_06"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncgen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_asyncgen.py::AsyncGenAsyncioTest::test_async_gen_asyncio_aclose_06
"""Auto-ported test: AsyncGenAsyncioTest::test_async_gen_asyncio_aclose_06 (CPython 3.12 oracle)."""


import inspect
import types
import unittest
import contextlib
from test.support.import_helper import import_module
from test.support import gc_collect, requires_working_socket


asyncio = import_module('asyncio')

requires_working_socket(module=True)

_no_default = object()

class AwaitException(Exception):
    pass

@types.coroutine
def awaitable(*, throw=False):
    if throw:
        yield ('throw',)
    else:
        yield ('result',)

def run_until_complete(coro):
    exc = False
    while True:
        try:
            if exc:
                exc = False
                fut = coro.throw(AwaitException)
            else:
                fut = coro.send(None)
        except StopIteration as ex:
            return ex.args[0]
        if fut == ('throw',):
            exc = True

def to_list(gen):

    async def iterate():
        res = []
        async for i in gen:
            res.append(i)
        return res
    return run_until_complete(iterate())

def py_anext(iterator, default=_no_default):
    """Pure-Python implementation of anext() for testing purposes.

    Closely matches the builtin anext() C implementation.
    Can be used to compare the built-in implementation of the inner
    coroutines machinery to C-implementation of __anext__() and send()
    or throw() on the returned generator.
    """
    try:
        __anext__ = type(iterator).__anext__
    except AttributeError:
        raise TypeError(f'{iterator!r} is not an async iterator')
    if default is _no_default:
        return __anext__(iterator)

    async def anext_impl():
        try:
            return await __anext__(iterator)
        except StopAsyncIteration:
            return default
    return anext_impl()


# --- test body ---
self_loop = asyncio.new_event_loop()
asyncio.set_event_loop(None)

async def foo():
    try:
        yield 1
        1 / 0
    finally:
        await asyncio.sleep(0.01)
        yield 12

async def run():
    gen = foo()
    it = gen.__aiter__()
    await it.__anext__()
    await gen.aclose()
try:
    self_loop.run_until_complete(run())
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('async generator ignored GeneratorExit', str(_aR_e))
print("AsyncGenAsyncioTest::test_async_gen_asyncio_aclose_06: ok")
