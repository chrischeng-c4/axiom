# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "asyncgen"
# dimension = "behavior"
# case = "async_gen_asyncio_test__test_sync_anext_raises_exception"
# subject = "cpython.test_asyncgen.AsyncGenAsyncioTest.test_sync_anext_raises_exception"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncgen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_asyncgen.py::AsyncGenAsyncioTest::test_sync_anext_raises_exception
"""Auto-ported test: AsyncGenAsyncioTest::test_sync_anext_raises_exception (CPython 3.12 oracle)."""


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
def _check_async_iterator_anext(ait_class, anext):
    g = ait_class()

    async def consume():
        results = []
        results.append(await anext(g))
        results.append(await anext(g))
        results.append(await anext(g, 'buckle my shoe'))
        return results
    res = self_loop.run_until_complete(consume())

    assert res == [1, 2, 'buckle my shoe']
    try:
        self_loop.run_until_complete(consume())
        raise AssertionError('expected StopAsyncIteration')
    except StopAsyncIteration:
        pass

    async def test_2():
        g1 = ait_class()
        self.assertEqual(await anext(g1), 1)
        self.assertEqual(await anext(g1), 2)
        with self.assertRaises(StopAsyncIteration):
            await anext(g1)
        with self.assertRaises(StopAsyncIteration):
            await anext(g1)
        g2 = ait_class()
        self.assertEqual(await anext(g2, 'default'), 1)
        self.assertEqual(await anext(g2, 'default'), 2)
        self.assertEqual(await anext(g2, 'default'), 'default')
        self.assertEqual(await anext(g2, 'default'), 'default')
        return 'completed'
    result = self_loop.run_until_complete(test_2())

    assert result == 'completed'

    def test_send():
        p = ait_class()
        obj = anext(p, 'completed')
        with self.assertRaises(StopIteration):
            with contextlib.closing(obj.__await__()) as g:
                g.send(None)
    test_send()

    async def test_throw():
        p = ait_class()
        obj = anext(p, 'completed')
        self.assertRaises(SyntaxError, obj.throw, SyntaxError)
        return 'completed'
    result = self_loop.run_until_complete(test_throw())

    assert result == 'completed'

def check_async_iterator_anext(ait_class):
    _check_async_iterator_anext(ait_class, py_anext)
    _check_async_iterator_anext(ait_class, anext)
self_loop = asyncio.new_event_loop()
asyncio.set_event_loop(None)
msg = 'custom'
for exc_type in [StopAsyncIteration, StopIteration, ValueError, Exception]:
    exc = exc_type(msg)

    class A:

        def __anext__(self):
            raise exc
    try:
        anext(A())
        raise AssertionError('expected exc_type')
    except exc_type as _aR_e:
        import re as _re_aR
        assert _re_aR.search(msg, str(_aR_e))
    try:
        anext(A(), 1)
        raise AssertionError('expected exc_type')
    except exc_type as _aR_e:
        import re as _re_aR
        assert _re_aR.search(msg, str(_aR_e))
print("AsyncGenAsyncioTest::test_sync_anext_raises_exception: ok")
