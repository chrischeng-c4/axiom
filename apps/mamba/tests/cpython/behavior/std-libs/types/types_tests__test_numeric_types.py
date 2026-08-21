# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_numeric_types"
# subject = "cpython.test_types.TypesTests.test_numeric_types"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::TypesTests::test_numeric_types
"""Auto-ported test: TypesTests::test_numeric_types (CPython 3.12 oracle)."""


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
if 0 != 0.0 or 1 != 1.0 or -1 != -1.0:

    raise AssertionError('int/float value not equal')
if int() != 0:

    raise AssertionError('int() does not return 0')
if float() != 0.0:

    raise AssertionError('float() does not return 0.0')
if int(1.9) == 1 == int(1.1) and int(-1.1) == -1 == int(-1.9):
    pass
else:

    raise AssertionError('int() does not round properly')
if float(1) == 1.0 and float(-1) == -1.0 and (float(0) == 0.0):
    pass
else:

    raise AssertionError('float() does not work properly')
print("TypesTests::test_numeric_types: ok")
