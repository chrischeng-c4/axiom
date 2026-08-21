# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_params_alias_value_test__test_recursive_repr"
# subject = "cpython.test_type_aliases.TypeParamsAliasValueTest.test_recursive_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeParamsAliasValueTest::test_recursive_repr
"""Auto-ported test: TypeParamsAliasValueTest::test_recursive_repr (CPython 3.12 oracle)."""


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
type Recursive = Recursive

assert repr(Recursive) == 'Recursive'
type X = list[Y]
type Y = list[X]

assert repr(X) == 'X'

assert repr(Y) == 'Y'
type GenericRecursive[X] = list[X | GenericRecursive[X]]

assert repr(GenericRecursive) == 'GenericRecursive'

assert repr(GenericRecursive[int]) == 'GenericRecursive[int]'

assert repr(GenericRecursive[GenericRecursive[int]]) == 'GenericRecursive[GenericRecursive[int]]'
print("TypeParamsAliasValueTest::test_recursive_repr: ok")
