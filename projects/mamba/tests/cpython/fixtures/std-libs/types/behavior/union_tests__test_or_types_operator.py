# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_or_types_operator"
# subject = "cpython.test_types.UnionTests.test_or_types_operator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::UnionTests::test_or_types_operator
"""Auto-ported test: UnionTests::test_or_types_operator (CPython 3.12 oracle)."""


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

assert int | str == typing.Union[int, str]

assert int | list != typing.Union[int, str]

assert str | int == typing.Union[int, str]

assert int | None == typing.Union[int, None]

assert None | int == typing.Union[int, None]

assert int | type(None) == int | None

assert type(None) | int == None | int

assert int | str | list == typing.Union[int, str, list]

assert int | (str | list) == typing.Union[int, str, list]

assert str | (int | list) == typing.Union[int, str, list]

assert typing.List | typing.Tuple == typing.Union[typing.List, typing.Tuple]

assert typing.List[int] | typing.Tuple[int] == typing.Union[typing.List[int], typing.Tuple[int]]

assert typing.List[int] | None == typing.Union[typing.List[int], None]

assert None | typing.List[int] == typing.Union[None, typing.List[int]]

assert str | float | int | complex | int == int | str | (float | complex)

assert typing.Union[str, int, typing.List[int]] == str | int | typing.List[int]

assert int | int is int

assert BaseException | bool | bytes | complex | float | int | list | map | set == typing.Union[BaseException, bool, bytes, complex, float, int, list, map, set]
try:
    int | 3
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    3 | int
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    Example() | int
    raise AssertionError('expected TypeError')
except TypeError:
    pass
x = int | str

assert x == int | str

assert x == str | int

assert x != {}
try:
    x < x
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    x <= x
    raise AssertionError('expected TypeError')
except TypeError:
    pass
y = typing.Union[str, int]
try:
    x < y
    raise AssertionError('expected TypeError')
except TypeError:
    pass
y = int | bool
try:
    x < y
    raise AssertionError('expected TypeError')
except TypeError:
    pass
y = typing.Union[str, int]
y.__args__ = [str, int]

assert x == y
print("UnionTests::test_or_types_operator: ok")
