# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_params_exotic_globals_test__test_exec_with_unusual_globals"
# subject = "cpython.test_type_aliases.TypeParamsExoticGlobalsTest.test_exec_with_unusual_globals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_aliases.py::TypeParamsExoticGlobalsTest::test_exec_with_unusual_globals
"""Auto-ported test: TypeParamsExoticGlobalsTest::test_exec_with_unusual_globals (CPython 3.12 oracle)."""


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
class customdict(dict):

    def __missing__(self, key):
        return key
code = compile('type Alias = undefined', 'test', 'exec')
ns = customdict()
exec(code, ns)
Alias = ns['Alias']

assert Alias.__value__ == 'undefined'
code = compile('class A: type Alias = undefined', 'test', 'exec')
ns = customdict()
exec(code, ns)
Alias = ns['A'].Alias

assert Alias.__value__ == 'undefined'
print("TypeParamsExoticGlobalsTest::test_exec_with_unusual_globals: ok")
