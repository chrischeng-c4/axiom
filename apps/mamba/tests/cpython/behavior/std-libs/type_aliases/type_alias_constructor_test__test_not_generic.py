# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_alias_constructor_test__test_not_generic"
# subject = "cpython.test_type_aliases.TypeAliasConstructorTest.test_not_generic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeAliasConstructorTest::test_not_generic
"""Auto-ported test: TypeAliasConstructorTest::test_not_generic (CPython 3.12 oracle)."""


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
TA = TypeAliasType('TA', list[int], type_params=())

assert TA.__name__ == 'TA'

assert TA.__value__ == list[int]

assert TA.__type_params__ == ()

assert TA.__module__ == __name__
try:
    TA[int]
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('Only generic type aliases are subscriptable', str(_aR_e))
print("TypeAliasConstructorTest::test_not_generic: ok")
