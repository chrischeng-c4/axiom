# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_params_alias_value_test__test_subscripting"
# subject = "cpython.test_type_aliases.TypeParamsAliasValueTest.test_subscripting"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeParamsAliasValueTest::test_subscripting
"""Auto-ported test: TypeParamsAliasValueTest::test_subscripting (CPython 3.12 oracle)."""


import pickle
import textwrap
import types
import unittest
from test.support import check_syntax_error, run_code
from test.typinganndata import mod_generics_cache
from typing import Callable, TypeAliasType, TypeVar, get_args


T = TypeVar('T')

type SimpleAlias = int

type RecursiveAlias = dict[str, RecursiveAlias]

type GenericAlias[X] = list[X]

type GenericAliasMultipleTypes[X, Y] = dict[X, Y]

type RecursiveGenericAlias[X] = dict[str, RecursiveAlias[X]]

type BoundGenericAlias[X: int] = set[X]

type ConstrainedGenericAlias[LongName: (str, bytes)] = list[LongName]

type AllTypesAlias[A, *B, **C] = Callable[C, A] | tuple[*B,]


# --- test body ---
type NonGeneric = int
type Generic[A] = dict[A, A]
type VeryGeneric[T, *Ts, **P] = Callable[P, tuple[T, *Ts]]
try:
    NonGeneric[int]
    raise AssertionError('expected TypeError')
except TypeError:
    pass
specialized = Generic[int]

assert isinstance(specialized, types.GenericAlias)

assert specialized.__origin__ is Generic

assert specialized.__args__ == (int,)
specialized2 = VeryGeneric[int, str, float, [bool, range]]

assert isinstance(specialized2, types.GenericAlias)

assert specialized2.__origin__ is VeryGeneric

assert specialized2.__args__ == (int, str, float, [bool, range])
print("TypeParamsAliasValueTest::test_subscripting: ok")
