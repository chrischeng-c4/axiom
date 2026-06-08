# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_strings"
# subject = "cpython.test_types.TypesTests.test_strings"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::TypesTests::test_strings
"""Auto-ported test: TypesTests::test_strings (CPython 3.12 oracle)."""


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
if len('') != 0:

    raise AssertionError("len('')")
if len('a') != 1:

    raise AssertionError("len('a')")
if len('abcdef') != 6:

    raise AssertionError("len('abcdef')")
if 'xyz' + 'abcde' != 'xyzabcde':

    raise AssertionError('string concatenation')
if 'xyz' * 3 != 'xyzxyzxyz':

    raise AssertionError('string repetition *3')
if 0 * 'abcde' != '':

    raise AssertionError('string repetition 0*')
if min('abc') != 'a' or max('abc') != 'c':

    raise AssertionError('min/max string')
if 'a' in 'abc' and 'b' in 'abc' and ('c' in 'abc') and ('d' not in 'abc'):
    pass
else:

    raise AssertionError('in/not in string')
x = 'x' * 103
if '%s!' % x != x + '!':

    raise AssertionError('nasty string formatting bug')
a = '0123456789'

assert a[:] == a

assert a[::2] == '02468'

assert a[1::2] == '13579'

assert a[::-1] == '9876543210'

assert a[::-2] == '97531'

assert a[3::-2] == '31'

assert a[-100:100] == a

assert a[100:-100:-1] == a[::-1]

assert a[-100:100:2] == '02468'
print("TypesTests::test_strings: ok")
