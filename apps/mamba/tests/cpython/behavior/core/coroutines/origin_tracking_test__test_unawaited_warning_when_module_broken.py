# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "origin_tracking_test__test_unawaited_warning_when_module_broken"
# subject = "cpython.test_coroutines.OriginTrackingTest.test_unawaited_warning_when_module_broken"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_coroutines.py::OriginTrackingTest::test_unawaited_warning_when_module_broken
"""Auto-ported test: OriginTrackingTest::test_unawaited_warning_when_module_broken (CPython 3.12 oracle)."""


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
async def corofn():
    pass
orig_wuc = warnings._warn_unawaited_coroutine
try:
    warnings._warn_unawaited_coroutine = lambda coro: 1 / 0
    with support.catch_unraisable_exception() as cm, warnings_helper.check_warnings(('coroutine .* was never awaited', RuntimeWarning)):
        coro = corofn()
        coro_repr = repr(coro)
        del coro
        support.gc_collect()

        assert repr(cm.unraisable.object) == coro_repr

        assert cm.unraisable.exc_type == ZeroDivisionError
    del warnings._warn_unawaited_coroutine
    with warnings_helper.check_warnings(('coroutine .* was never awaited', RuntimeWarning)):
        corofn()
        support.gc_collect()
finally:
    warnings._warn_unawaited_coroutine = orig_wuc
print("OriginTrackingTest::test_unawaited_warning_when_module_broken: ok")
