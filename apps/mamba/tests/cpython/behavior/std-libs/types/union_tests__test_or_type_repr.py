# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_or_type_repr"
# subject = "cpython.test_types.UnionTests.test_or_type_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_or_type_repr
"""Auto-ported test: UnionTests::test_or_type_repr (CPython 3.12 oracle)."""


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

assert repr(int | str) == 'int | str'

assert repr(int | str | list) == 'int | str | list'

assert repr(int | (str | list)) == 'int | str | list'

assert repr(int | None) == 'int | None'

assert repr(int | type(None)) == 'int | None'

assert repr(int | typing.GenericAlias(list, int)) == 'int | list[int]'
print("UnionTests::test_or_type_repr: ok")
