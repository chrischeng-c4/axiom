# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_floats"
# subject = "cpython.test_types.TypesTests.test_floats"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::TypesTests::test_floats
"""Auto-ported test: TypesTests::test_floats (CPython 3.12 oracle)."""


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
if 12.0 + 24.0 != 36.0:

    raise AssertionError('float op')
if 12.0 + -24.0 != -12.0:

    raise AssertionError('float op')
if -12.0 + 24.0 != 12.0:

    raise AssertionError('float op')
if -12.0 + -24.0 != -36.0:

    raise AssertionError('float op')
if not 12.0 < 24.0:

    raise AssertionError('float op')
if not -24.0 < -12.0:

    raise AssertionError('float op')
print("TypesTests::test_floats: ok")
