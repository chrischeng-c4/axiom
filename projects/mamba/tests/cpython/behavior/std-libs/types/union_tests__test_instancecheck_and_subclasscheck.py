# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_instancecheck_and_subclasscheck"
# subject = "cpython.test_types.UnionTests.test_instancecheck_and_subclasscheck"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_instancecheck_and_subclasscheck
"""Auto-ported test: UnionTests::test_instancecheck_and_subclasscheck (CPython 3.12 oracle)."""


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
for x in (int | str, typing.Union[int, str]):

    assert isinstance(1, x)

    assert isinstance(True, x)

    assert isinstance('a', x)

    assert not isinstance(None, x)

    assert issubclass(int, x)

    assert issubclass(bool, x)

    assert issubclass(str, x)

    assert not issubclass(type(None), x)
for x in (int | None, typing.Union[int, None]):

    assert isinstance(None, x)

    assert issubclass(type(None), x)
for x in (int | collections.abc.Mapping, typing.Union[int, collections.abc.Mapping]):

    assert isinstance({}, x)

    assert not isinstance((), x)

    assert issubclass(dict, x)

    assert not issubclass(list, x)
print("UnionTests::test_instancecheck_and_subclasscheck: ok")
