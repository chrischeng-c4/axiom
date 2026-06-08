# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_truth_values"
# subject = "cpython.test_types.TypesTests.test_truth_values"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::TypesTests::test_truth_values
"""Auto-ported test: TypesTests::test_truth_values (CPython 3.12 oracle)."""


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
if None:

    raise AssertionError('None is true instead of false')
if 0:

    raise AssertionError('0 is true instead of false')
if 0.0:

    raise AssertionError('0.0 is true instead of false')
if '':

    raise AssertionError("'' is true instead of false")
if not 1:

    raise AssertionError('1 is false instead of true')
if not 1.0:

    raise AssertionError('1.0 is false instead of true')
if not 'x':

    raise AssertionError("'x' is false instead of true")
if not {'x': 1}:

    raise AssertionError("{'x': 1} is false instead of true")

def f():
    pass

class C:
    pass
x = C()
if not f:

    raise AssertionError('f is false instead of true')
if not C:

    raise AssertionError('C is false instead of true')
if not sys:

    raise AssertionError('sys is false instead of true')
if not x:

    raise AssertionError('x is false instead of true')
print("TypesTests::test_truth_values: ok")
