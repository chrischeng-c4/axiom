# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_alias_constructor_test__test_errors"
# subject = "cpython.test_type_aliases.TypeAliasConstructorTest.test_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeAliasConstructorTest::test_errors
"""Auto-ported test: TypeAliasConstructorTest::test_errors (CPython 3.12 oracle)."""


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
try:
    TypeAliasType()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    TypeAliasType('TA')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    TypeAliasType('TA', list, ())
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    TypeAliasType('TA', list, type_params=42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TypeAliasConstructorTest::test_errors: ok")
