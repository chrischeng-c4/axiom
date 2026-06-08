# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "coroutine_test__test_comp_4_2"
# subject = "cpython.test_coroutines.CoroutineTest.test_comp_4_2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_coroutines.py::CoroutineTest::test_comp_4_2
"""Auto-ported test: CoroutineTest::test_comp_4_2 (CPython 3.12 oracle)."""


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
async def f(it):
    for i in it:
        yield i

async def run_list():
    return [i + 10 async for i in f(range(5)) if 0 < i < 4]

assert run_async(run_list()) == ([], [11, 12, 13])

async def run_set():
    return {i + 10 async for i in f(range(5)) if 0 < i < 4}

assert run_async(run_set()) == ([], {11, 12, 13})

async def run_dict():
    return {i + 10: i + 100 async for i in f(range(5)) if 0 < i < 4}

assert run_async(run_dict()) == ([], {11: 101, 12: 102, 13: 103})

async def run_gen():
    gen = (i + 10 async for i in f(range(5)) if 0 < i < 4)
    return [g + 100 async for g in gen]

assert run_async(run_gen()) == ([], [111, 112, 113])
print("CoroutineTest::test_comp_4_2: ok")
