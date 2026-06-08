# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_params_alias_value_test__test_alias_value_04"
# subject = "cpython.test_type_aliases.TypeParamsAliasValueTest.test_alias_value_04"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeParamsAliasValueTest::test_alias_value_04
"""Auto-ported test: TypeParamsAliasValueTest::test_alias_value_04 (CPython 3.12 oracle)."""


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
def more_generic[T, *Ts, **P]():
    type TA[T2, *Ts2, **P2] = tuple[Callable[P, tuple[T, *Ts]], Callable[P2, tuple[T2, *Ts2]]]
    return TA
alias = more_generic()

assert isinstance(alias, TypeAliasType)
T2, Ts2, P2 = alias.__type_params__

assert alias.__parameters__ == (T2, *Ts2, P2)
T, Ts, P = more_generic.__type_params__

assert alias.__value__ == tuple[Callable[P, tuple[T, *Ts]], Callable[P2, tuple[T2, *Ts2]]]
print("TypeParamsAliasValueTest::test_alias_value_04: ok")
