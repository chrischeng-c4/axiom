# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_int_format_locale"
# subject = "cpython.test_types.TypesTests.test_int__format__locale"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::TypesTests::test_int__format__locale
"""Auto-ported test: TypesTests::test_int__format__locale (CPython 3.12 oracle)."""


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
x = 123456789012345678901234567890
for i in range(0, 30):

    assert locale.format_string('%d', x, grouping=True) == format(x, 'n')
    x = x // 10
rfmt = '>20n'
lfmt = '<20n'
cfmt = '^20n'
for x in (1234, 12345, 123456, 1234567, 12345678, 123456789, 1234567890, 12345678900):

    assert len(format(0, rfmt)) == len(format(x, rfmt))

    assert len(format(0, lfmt)) == len(format(x, lfmt))

    assert len(format(0, cfmt)) == len(format(x, cfmt))
print("TypesTests::test_int__format__locale: ok")
