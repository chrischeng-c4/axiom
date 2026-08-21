# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "coroutine_test__test_func_18"
# subject = "cpython.test_coroutines.CoroutineTest.test_func_18"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_coroutines.py::CoroutineTest::test_func_18
"""Auto-ported test: CoroutineTest::test_func_18 (CPython 3.12 oracle)."""


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
async def coroutine():
    return 'spam'
coro = coroutine()
await_iter = coro.__await__()
it = iter(await_iter)
try:
    it.send(None)
    raise AssertionError('expected StopIteration')
except StopIteration as _aR_e:
    import re as _re_aR
    assert _re_aR.search('spam', str(_aR_e))
try:
    it.send(None)
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('cannot reuse already awaited coroutine', str(_aR_e))
try:
    next(it)
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('cannot reuse already awaited coroutine', str(_aR_e))
try:
    it.throw(Exception('wat'))
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('cannot reuse already awaited coroutine', str(_aR_e))
try:
    it.throw(Exception('wat'))
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('cannot reuse already awaited coroutine', str(_aR_e))
it.close()
it.close()
print("CoroutineTest::test_func_18: ok")
