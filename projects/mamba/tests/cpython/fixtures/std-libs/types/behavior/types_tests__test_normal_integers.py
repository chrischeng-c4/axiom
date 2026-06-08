# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_normal_integers"
# subject = "cpython.test_types.TypesTests.test_normal_integers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::TypesTests::test_normal_integers
"""Auto-ported test: TypesTests::test_normal_integers (CPython 3.12 oracle)."""


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
a = 256
b = 128 * 2
if a is not b:

    raise AssertionError('256 is not shared')
if 12 + 24 != 36:

    raise AssertionError('int op')
if 12 + -24 != -12:

    raise AssertionError('int op')
if -12 + 24 != 12:

    raise AssertionError('int op')
if -12 + -24 != -36:

    raise AssertionError('int op')
if not 12 < 24:

    raise AssertionError('int op')
if not -24 < -12:

    raise AssertionError('int op')
xsize, ysize, zsize = (238, 356, 4)
if not xsize * ysize * zsize == zsize * xsize * ysize == 338912:

    raise AssertionError('int mul commutativity')
m = -sys.maxsize - 1
for divisor in (1, 2, 4, 8, 16, 32):
    j = m // divisor
    prod = divisor * j
    if prod != m:

        raise AssertionError('%r * %r == %r != %r' % (divisor, j, prod, m))
    if type(prod) is not int:

        raise AssertionError('expected type(prod) to be int, not %r' % type(prod))
for divisor in (1, 2, 4, 8, 16, 32):
    j = m // divisor - 1
    prod = divisor * j
    if type(prod) is not int:

        raise AssertionError('expected type(%r) to be int, not %r' % (prod, type(prod)))
m = sys.maxsize
for divisor in (1, 2, 4, 8, 16, 32):
    j = m // divisor + 1
    prod = divisor * j
    if type(prod) is not int:

        raise AssertionError('expected type(%r) to be int, not %r' % (prod, type(prod)))
x = sys.maxsize

assert isinstance(x + 1, int)

assert isinstance(-x - 1, int)

assert isinstance(-x - 2, int)
try:
    5 << -5
except ValueError:
    pass
else:

    raise AssertionError('int negative shift <<')
try:
    5 >> -5
except ValueError:
    pass
else:

    raise AssertionError('int negative shift >>')
print("TypesTests::test_normal_integers: ok")
