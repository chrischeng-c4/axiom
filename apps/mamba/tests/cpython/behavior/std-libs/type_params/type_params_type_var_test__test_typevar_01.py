# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_type_var_test__test_typevar_01"
# subject = "cpython.test_type_params.TypeParamsTypeVarTest.test_typevar_01"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsTypeVarTest::test_typevar_01
"""Auto-ported test: TypeParamsTypeVarTest::test_typevar_01 (CPython 3.12 oracle)."""


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
def func1[A: str, B: str | int, C: (int, str)]():
    return (A, B, C)
a, b, c = func1()

assert isinstance(a, TypeVar)

assert a.__bound__ == str

assert a.__infer_variance__

assert not a.__covariant__

assert not a.__contravariant__

assert isinstance(b, TypeVar)

assert b.__bound__ == str | int

assert b.__infer_variance__

assert not b.__covariant__

assert not b.__contravariant__

assert isinstance(c, TypeVar)

assert c.__bound__ == None

assert c.__constraints__ == (int, str)

assert c.__infer_variance__

assert not c.__covariant__

assert not c.__contravariant__
print("TypeParamsTypeVarTest::test_typevar_01: ok")
