# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_tests__test_gen"
# subject = "cpython.test_types.CoroutineTests.test_gen"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::CoroutineTests::test_gen
"""Auto-ported test: CoroutineTests::test_gen (CPython 3.12 oracle)."""


from test.support import run_with_locale, cpython_only, iter_builtin_types, iter_slot_wrappers, MISSING_C_DOCSTRINGS
from test.test_import import no_rerun
import collections.abc
from collections import namedtuple
import copy
import gc
import inspect
import pickle
import locale
import sys
import textwrap
import types
import unittest.mock
import weakref
import typing


T = typing.TypeVar('T')

class Example:
    pass

class Forward:
    ...

def clear_typing_caches():
    for f in typing._cleanups:
        f()


# --- test body ---
def gen_func():
    yield 1
    return (yield 2)
gen = gen_func()

@types.coroutine
def foo():
    return gen
wrapper = foo()

assert isinstance(wrapper, types._GeneratorWrapper)

assert wrapper.__await__() is gen
for name in ('__name__', '__qualname__', 'gi_code', 'gi_running', 'gi_frame'):

    assert getattr(foo(), name) is getattr(gen, name)

assert foo().cr_code is gen.gi_code

assert next(wrapper) == 1

assert wrapper.send(None) == 2
try:
    wrapper.send('spam')
    raise AssertionError('expected StopIteration')
except StopIteration as _aR_e:
    import re as _re_aR
    assert _re_aR.search('spam', str(_aR_e))
gen = gen_func()
wrapper = foo()
wrapper.send(None)
try:
    wrapper.throw(Exception('ham'))
    raise AssertionError('expected Exception')
except Exception as _aR_e:
    import re as _re_aR
    assert _re_aR.search('ham', str(_aR_e))
foo = types.coroutine(foo)

assert foo().__await__() is gen
print("CoroutineTests::test_gen: ok")
