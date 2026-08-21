# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "asyncgen"
# dimension = "behavior"
# case = "async_gen_test__test_async_gen_exception_11"
# subject = "cpython.test_asyncgen.AsyncGenTest.test_async_gen_exception_11"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncgen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_asyncgen.py::AsyncGenTest::test_async_gen_exception_11
"""Auto-ported test: AsyncGenTest::test_async_gen_exception_11 (CPython 3.12 oracle)."""


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
def compare_generators(sync_gen, async_gen):

    def sync_iterate(g):
        res = []
        while True:
            try:
                res.append(g.__next__())
            except StopIteration:
                res.append('STOP')
                break
            except Exception as ex:
                res.append(str(type(ex)))
        return res

    def async_iterate(g):
        res = []
        while True:
            an = g.__anext__()
            try:
                while True:
                    try:
                        an.__next__()
                    except StopIteration as ex:
                        if ex.args:
                            res.append(ex.args[0])
                            break
                        else:
                            res.append('EMPTY StopIteration')
                            break
                    except StopAsyncIteration:
                        raise
                    except Exception as ex:
                        res.append(str(type(ex)))
                        break
            except StopAsyncIteration:
                res.append('STOP')
                break
        return res
    sync_gen_result = sync_iterate(sync_gen)
    async_gen_result = async_iterate(async_gen)

    assert sync_gen_result == async_gen_result
    return async_gen_result

def sync_gen():
    yield 10
    yield 20

def sync_gen_wrapper():
    yield 1
    sg = sync_gen()
    sg.send(None)
    try:
        sg.throw(GeneratorExit())
    except GeneratorExit:
        yield 2
    yield 3

async def async_gen():
    yield 10
    yield 20

async def async_gen_wrapper():
    yield 1
    asg = async_gen()
    await asg.asend(None)
    try:
        await asg.athrow(GeneratorExit())
    except GeneratorExit:
        yield 2
    yield 3
compare_generators(sync_gen_wrapper(), async_gen_wrapper())
print("AsyncGenTest::test_async_gen_exception_11: ok")
