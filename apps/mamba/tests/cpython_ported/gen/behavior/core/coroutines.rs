use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/coroutines/async_bad_syntax_test__test_badsyntax_3.py`.
#[test]
fn test_gen_behavior_core_coroutines_async_bad_syntax_test__test_badsyntax_3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "async_bad_syntax_test__test_badsyntax_3"
# subject = "cpython.test_coroutines.AsyncBadSyntaxTest.test_badsyntax_3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_coroutines.py::AsyncBadSyntaxTest::test_badsyntax_3
"""Auto-ported test: AsyncBadSyntaxTest::test_badsyntax_3 (CPython 3.12 oracle)."""


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
try:
    compile('async = 1', '<test>', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("AsyncBadSyntaxTest::test_badsyntax_3: ok")
"###);
    assert_output(&out, r###"AsyncBadSyntaxTest::test_badsyntax_3: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/coroutines/coroutine_test__test_gen_1.py`.
#[test]
fn test_gen_behavior_core_coroutines_coroutine_test__test_gen_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "coroutine_test__test_gen_1"
# subject = "cpython.test_coroutines.CoroutineTest.test_gen_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_coroutines.py::CoroutineTest::test_gen_1
"""Auto-ported test: CoroutineTest::test_gen_1 (CPython 3.12 oracle)."""


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
def gen():
    yield

assert not hasattr(gen, '__await__')
print("CoroutineTest::test_gen_1: ok")
"###);
    assert_output(&out, r###"CoroutineTest::test_gen_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/coroutines/unawaited_warning_during_shutdown_test__test_unawaited_warning_during_shutdown.py`.
#[test]
fn test_gen_behavior_core_coroutines_unawaited_warning_during_shutdown_test__test_unawaited_warning_during_shutdown() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "unawaited_warning_during_shutdown_test__test_unawaited_warning_during_shutdown"
# subject = "cpython.test_coroutines.UnawaitedWarningDuringShutdownTest.test_unawaited_warning_during_shutdown"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_coroutines.py::UnawaitedWarningDuringShutdownTest::test_unawaited_warning_during_shutdown
"""Auto-ported test: UnawaitedWarningDuringShutdownTest::test_unawaited_warning_during_shutdown (CPython 3.12 oracle)."""


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
code = 'import asyncio\nasync def f(): pass\nasync def t(): asyncio.gather(f())\nasyncio.run(t())\n'
assert_python_ok('-c', code)
code = 'import sys\nasync def f(): pass\nsys.coro = f()\n'
assert_python_ok('-c', code)
code = 'import sys\nasync def f(): pass\nsys.corocycle = [f()]\nsys.corocycle.append(sys.corocycle)\n'
assert_python_ok('-c', code)
print("UnawaitedWarningDuringShutdownTest::test_unawaited_warning_during_shutdown: ok")
"###);
    assert_output(&out, r###"UnawaitedWarningDuringShutdownTest::test_unawaited_warning_during_shutdown: ok
"###);
}
