# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_class_scope_test__test_bound"
# subject = "cpython.test_type_params.TypeParamsClassScopeTest.test_bound"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsClassScopeTest::test_bound
"""Auto-ported test: TypeParamsClassScopeTest::test_bound (CPython 3.12 oracle)."""


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
class X:
    T = int

    def foo[U: T](self):
        ...

assert X.foo.__type_params__[0].__bound__ is int
ns = run_code('\n            glb = "global"\n            class X:\n                cls = "class"\n                def foo[T: glb, U: cls](self): ...\n        ')
cls = ns['X']
T, U = cls.foo.__type_params__

assert T.__bound__ == 'global'

assert U.__bound__ == 'class'
print("TypeParamsClassScopeTest::test_bound: ok")
