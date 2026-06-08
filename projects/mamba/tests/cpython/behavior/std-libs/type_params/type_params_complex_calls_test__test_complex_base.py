# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_complex_calls_test__test_complex_base"
# subject = "cpython.test_type_params.TypeParamsComplexCallsTest.test_complex_base"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsComplexCallsTest::test_complex_base
"""Auto-ported test: TypeParamsComplexCallsTest::test_complex_base (CPython 3.12 oracle)."""


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
class Base:

    def __init_subclass__(cls, **kwargs) -> None:
        cls.kwargs = kwargs
kwargs = {'c': 3}

class C[T](Base, a=1, b=2, **kwargs):
    pass
T, = C.__type_params__

assert T.__name__ == 'T'

assert C.kwargs == {'a': 1, 'b': 2, 'c': 3}

assert C.__bases__ == (Base, Generic)
bases = (Base,)

class C2[T](*bases, **kwargs):
    pass
T, = C2.__type_params__

assert T.__name__ == 'T'

assert C2.kwargs == {'c': 3}

assert C2.__bases__ == (Base, Generic)
print("TypeParamsComplexCallsTest::test_complex_base: ok")
