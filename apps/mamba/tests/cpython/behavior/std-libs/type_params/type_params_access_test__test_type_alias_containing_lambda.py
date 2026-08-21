# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_access_test__test_type_alias_containing_lambda"
# subject = "cpython.test_type_params.TypeParamsAccessTest.test_type_alias_containing_lambda"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsAccessTest::test_type_alias_containing_lambda
"""Auto-ported test: TypeParamsAccessTest::test_type_alias_containing_lambda (CPython 3.12 oracle)."""


import asyncio
import textwrap
import types
import unittest
import pickle
import weakref
from test.support import requires_working_socket, check_syntax_error, run_code
from typing import Generic, Sequence, TypeVar, TypeVarTuple, ParamSpec, get_args


def make_base(arg):

    class Base:
        __arg__ = arg
    return Base

def global_generic_func[T]():
    pass

class GlobalGenericClass[T]:
    pass

T = TypeVar('T')

def func1[X](x: X) -> X:
    ...

def func2[X, Y](x: X | Y) -> X | Y:
    ...

def func3[X, *Y, **Z](x: X, y: tuple[*Y,], z: Z) -> X:
    ...

def func4[X: int, Y: (bytes, str)](x: X, y: Y) -> X | Y:
    ...

class Class1[X]:
    ...

class Class2[X, Y]:
    ...

class Class3[X, *Y, **Z]:
    ...

class Class4[X: int, Y: (bytes, str)]:
    ...


# --- test body ---
type Alias[T] = lambda: T
T, = Alias.__type_params__

assert Alias.__value__() is T
print("TypeParamsAccessTest::test_type_alias_containing_lambda: ok")
