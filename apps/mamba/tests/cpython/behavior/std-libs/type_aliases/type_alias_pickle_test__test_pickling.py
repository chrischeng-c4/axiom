# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_alias_pickle_test__test_pickling"
# subject = "cpython.test_type_aliases.TypeAliasPickleTest.test_pickling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeAliasPickleTest::test_pickling
"""Auto-ported test: TypeAliasPickleTest::test_pickling (CPython 3.12 oracle)."""


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
things_to_test = [SimpleAlias, RecursiveAlias, GenericAlias, GenericAlias[T], GenericAlias[int], GenericAliasMultipleTypes, GenericAliasMultipleTypes[str, T], GenericAliasMultipleTypes[T, str], GenericAliasMultipleTypes[int, str], RecursiveGenericAlias, RecursiveGenericAlias[T], RecursiveGenericAlias[int], BoundGenericAlias, BoundGenericAlias[int], BoundGenericAlias[T], ConstrainedGenericAlias, ConstrainedGenericAlias[str], ConstrainedGenericAlias[T], AllTypesAlias, AllTypesAlias[int, str, T, [T, object]], mod_generics_cache.Alias, mod_generics_cache.OldStyle]
for thing in things_to_test:
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        pickled = pickle.dumps(thing, protocol=proto)

        assert pickle.loads(pickled) == thing
print("TypeAliasPickleTest::test_pickling: ok")
