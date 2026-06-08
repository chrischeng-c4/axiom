# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_params_access_test__test_alias_access_03"
# subject = "cpython.test_type_aliases.TypeParamsAccessTest.test_alias_access_03"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeParamsAccessTest::test_alias_access_03
"""Auto-ported test: TypeParamsAccessTest::test_alias_access_03 (CPython 3.12 oracle)."""


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
ns = run_code('\n            class Outer[A]:\n                def inner[B](self):\n                    type TA1[C] = TA1[A, B] | int\n                    return TA1\n            ')
cls = ns['Outer']
A, = cls.__type_params__
B, = cls.inner.__type_params__
alias = cls.inner(None)

assert isinstance(alias, TypeAliasType)
alias2 = cls.inner(None)

assert alias is not alias2

assert len(alias.__type_params__) == 1

assert alias.__value__ == alias[A, B] | int
print("TypeParamsAccessTest::test_alias_access_03: ok")
