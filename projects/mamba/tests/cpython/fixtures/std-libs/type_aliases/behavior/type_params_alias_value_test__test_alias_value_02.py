# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_params_alias_value_test__test_alias_value_02"
# subject = "cpython.test_type_aliases.TypeParamsAliasValueTest.test_alias_value_02"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeParamsAliasValueTest::test_alias_value_02
"""Auto-ported test: TypeParamsAliasValueTest::test_alias_value_02 (CPython 3.12 oracle)."""


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
class Parent[A]:
    type TA1[B] = dict[A, B]

assert isinstance(Parent.TA1, TypeAliasType)

assert len(Parent.TA1.__parameters__) == 1

assert len(Parent.__parameters__) == 1
a, = Parent.__parameters__
b, = Parent.TA1.__parameters__

assert Parent.__type_params__ == (a,)

assert Parent.TA1.__type_params__ == (b,)

assert Parent.TA1.__value__ == dict[a, b]
print("TypeParamsAliasValueTest::test_alias_value_02: ok")
