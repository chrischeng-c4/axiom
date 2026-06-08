# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_mangling_test__test_mangling"
# subject = "cpython.test_type_params.TypeParamsManglingTest.test_mangling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsManglingTest::test_mangling
"""Auto-ported test: TypeParamsManglingTest::test_mangling (CPython 3.12 oracle)."""


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
class Foo[__T]:
    param = __T

    def meth[__U](self, arg: __T, arg2: __U):
        return (__T, __U)
    type Alias[__V] = (__T, __V)
T = Foo.__type_params__[0]

assert T.__name__ == '__T'
U = Foo.meth.__type_params__[0]

assert U.__name__ == '__U'
V = Foo.Alias.__type_params__[0]

assert V.__name__ == '__V'
anno = Foo.meth.__annotations__

assert anno['arg'] is T

assert anno['arg2'] is U

assert Foo().meth(1, 2) == (T, U)

assert Foo.Alias.__value__ == (T, V)
print("TypeParamsManglingTest::test_mangling: ok")
