# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_tests__test_union_parameter_substitution"
# subject = "cpython.test_types.UnionTests.test_union_parameter_substitution"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
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
import weakref
import typing

def eq(actual, expected, typed=True):
    assert actual == expected
    if typed:
        assert type(actual) is type(expected)
T = typing.TypeVar('T')
S = typing.TypeVar('S')
NT = typing.NewType('NT', str)
x = int | T | bytes
eq(x[str], int | str | bytes, typed=False)
eq(x[list[int]], int | list[int] | bytes, typed=False)
eq(x[typing.List], int | typing.List | bytes)
eq(x[typing.List[int]], int | typing.List[int] | bytes)
eq(x[typing.Hashable], int | typing.Hashable | bytes)
eq(x[collections.abc.Hashable], int | collections.abc.Hashable | bytes, typed=False)
eq(x[typing.Callable[[int], str]], int | typing.Callable[[int], str] | bytes)
eq(x[collections.abc.Callable[[int], str]], int | collections.abc.Callable[[int], str] | bytes, typed=False)
eq(x[typing.Tuple[int, str]], int | typing.Tuple[int, str] | bytes)
eq(x[typing.Literal['none']], int | typing.Literal['none'] | bytes)
eq(x[str | list], int | str | list | bytes, typed=False)
eq(x[typing.Union[str, list]], typing.Union[int, str, list, bytes])
eq(x[str | int], int | str | bytes, typed=False)
eq(x[typing.Union[str, int]], typing.Union[int, str, bytes])
eq(x[NT], int | NT | bytes)
eq(x[S], int | S | bytes)

print("UnionTests::test_union_parameter_substitution: ok")
