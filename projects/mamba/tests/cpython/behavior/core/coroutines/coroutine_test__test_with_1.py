# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "coroutine_test__test_with_1"
# subject = "cpython.test_coroutines.CoroutineTest.test_with_1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_coroutines.py::CoroutineTest::test_with_1
"""Auto-ported test: CoroutineTest::test_with_1 (CPython 3.12 oracle)."""


import contextlib
import copy
import inspect
import pickle
import sys
import types
import traceback
import unittest
import warnings
from test import support
from test.support import import_helper
from test.support import warnings_helper
from test.support.script_helper import assert_python_ok


class AsyncYieldFrom:

    def __init__(self, obj):
        self.obj = obj

    def __await__(self):
        yield from self.obj

class AsyncYield:

    def __init__(self, value):
        self.value = value

    def __await__(self):
        yield self.value

async def asynciter(iterable):
    """Convert an iterable to an asynchronous iterator."""
    for x in iterable:
        yield x

def run_async(coro):
    assert coro.__class__ in {types.GeneratorType, types.CoroutineType}
    buffer = []
    result = None
    while True:
        try:
            buffer.append(coro.send(None))
        except StopIteration as ex:
            result = ex.args[0] if ex.args else None
            break
    return (buffer, result)

def run_async__await__(coro):
    assert coro.__class__ is types.CoroutineType
    aw = coro.__await__()
    buffer = []
    result = None
    i = 0
    while True:
        try:
            if i % 2:
                buffer.append(next(aw))
            else:
                buffer.append(aw.send(None))
            i += 1
        except StopIteration as ex:
            result = ex.args[0] if ex.args else None
            break
    return (buffer, result)

@contextlib.contextmanager
def silence_coro_gc():
    with warnings.catch_warnings():
        warnings.simplefilter('ignore')
        yield
        support.gc_collect()


# --- test body ---
class Manager:

    def __init__(self, name):
        self.name = name

    async def __aenter__(self):
        await AsyncYieldFrom(['enter-1-' + self.name, 'enter-2-' + self.name])
        return self

    async def __aexit__(self, *args):
        await AsyncYieldFrom(['exit-1-' + self.name, 'exit-2-' + self.name])
        if self.name == 'B':
            return True

async def foo():
    async with Manager('A') as a, Manager('B') as b:
        await AsyncYieldFrom([('managers', a.name, b.name)])
        1 / 0
f = foo()
result, _ = run_async(f)

assert result == ['enter-1-A', 'enter-2-A', 'enter-1-B', 'enter-2-B', ('managers', 'A', 'B'), 'exit-1-B', 'exit-2-B', 'exit-1-A', 'exit-2-A']

async def foo():
    async with Manager('A') as a, Manager('C') as c:
        await AsyncYieldFrom([('managers', a.name, c.name)])
        1 / 0
try:
    run_async(foo())
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass
print("CoroutineTest::test_with_1: ok")
