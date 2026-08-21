# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_params_alias_value_test__test_repr"
# subject = "cpython.test_type_aliases.TypeParamsAliasValueTest.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeParamsAliasValueTest::test_repr
"""Auto-ported test: TypeParamsAliasValueTest::test_repr (CPython 3.12 oracle)."""


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
type Simple = int
type VeryGeneric[T, *Ts, **P] = Callable[P, tuple[T, *Ts]]

assert repr(Simple) == 'Simple'

assert repr(VeryGeneric) == 'VeryGeneric'

assert repr(VeryGeneric[int, bytes, str, [float, object]]) == 'VeryGeneric[int, bytes, str, [float, object]]'

assert repr(VeryGeneric[int, []]) == 'VeryGeneric[int, []]'

assert repr(VeryGeneric[int, [VeryGeneric[int], list[str]]]) == 'VeryGeneric[int, [VeryGeneric[int], list[str]]]'
print("TypeParamsAliasValueTest::test_repr: ok")
