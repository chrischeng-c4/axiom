# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "coroutine_test__test_nested_comp"
# subject = "cpython.test_coroutines.CoroutineTest.test_nested_comp"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_coroutines.py::CoroutineTest::test_nested_comp
"""Auto-ported test: CoroutineTest::test_nested_comp (CPython 3.12 oracle)."""


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
async def run_list_inside_list():
    return [[i + j async for i in asynciter([1, 2])] for j in [10, 20]]

assert run_async(run_list_inside_list()) == ([], [[11, 12], [21, 22]])

async def run_set_inside_list():
    return [{i + j async for i in asynciter([1, 2])} for j in [10, 20]]

assert run_async(run_set_inside_list()) == ([], [{11, 12}, {21, 22}])

async def run_list_inside_set():
    return {sum([i async for i in asynciter(range(j))]) for j in [3, 5]}

assert run_async(run_list_inside_set()) == ([], {3, 10})

async def run_dict_inside_dict():
    return {j: {i: i + j async for i in asynciter([1, 2])} for j in [10, 20]}

assert run_async(run_dict_inside_dict()) == ([], {10: {1: 11, 2: 12}, 20: {1: 21, 2: 22}})

async def run_list_inside_gen():
    gen = ([i + j async for i in asynciter([1, 2])] for j in [10, 20])
    return [x async for x in gen]

assert run_async(run_list_inside_gen()) == ([], [[11, 12], [21, 22]])

async def run_gen_inside_list():
    gens = [(i async for i in asynciter(range(j))) for j in [3, 5]]
    return [x for g in gens async for x in g]

assert run_async(run_gen_inside_list()) == ([], [0, 1, 2, 0, 1, 2, 3, 4])

async def run_gen_inside_gen():
    gens = ((i async for i in asynciter(range(j))) for j in [3, 5])
    return [x for g in gens async for x in g]

assert run_async(run_gen_inside_gen()) == ([], [0, 1, 2, 0, 1, 2, 3, 4])

async def run_list_inside_list_inside_list():
    return [[[i + j + k async for i in asynciter([1, 2])] for j in [10, 20]] for k in [100, 200]]

assert run_async(run_list_inside_list_inside_list()) == ([], [[[111, 112], [121, 122]], [[211, 212], [221, 222]]])
print("CoroutineTest::test_nested_comp: ok")
