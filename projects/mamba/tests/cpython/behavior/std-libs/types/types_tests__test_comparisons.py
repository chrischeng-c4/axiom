# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_comparisons"
# subject = "cpython.test_types.TypesTests.test_comparisons"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::TypesTests::test_comparisons
"""Auto-ported test: TypesTests::test_comparisons (CPython 3.12 oracle)."""


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
if 0 < 1 <= 1 == 1 >= 1 > 0 != 1:
    pass
else:

    raise AssertionError('int comparisons failed')
if 0.0 < 1.0 <= 1.0 == 1.0 >= 1.0 > 0.0 != 1.0:
    pass
else:

    raise AssertionError('float comparisons failed')
if '' < 'a' <= 'a' == 'a' < 'abc' < 'abd' < 'b':
    pass
else:

    raise AssertionError('string comparisons failed')
if None is None:
    pass
else:

    raise AssertionError('identity test failed')
print("TypesTests::test_comparisons: ok")
