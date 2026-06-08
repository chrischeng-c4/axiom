# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_alias_type_test__test_union"
# subject = "cpython.test_type_aliases.TypeAliasTypeTest.test_union"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeAliasTypeTest::test_union
"""Auto-ported test: TypeAliasTypeTest::test_union (CPython 3.12 oracle)."""


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
type Alias1 = int
type Alias2 = str
union = Alias1 | Alias2

assert isinstance(union, types.UnionType)

assert get_args(union) == (Alias1, Alias2)
union2 = Alias1 | list[float]

assert isinstance(union2, types.UnionType)

assert get_args(union2) == (Alias1, list[float])
union3 = list[range] | Alias1

assert isinstance(union3, types.UnionType)

assert get_args(union3) == (list[range], Alias1)
print("TypeAliasTypeTest::test_union: ok")
