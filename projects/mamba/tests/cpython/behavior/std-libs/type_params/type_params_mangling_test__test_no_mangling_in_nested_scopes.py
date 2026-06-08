# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_mangling_test__test_no_mangling_in_nested_scopes"
# subject = "cpython.test_type_params.TypeParamsManglingTest.test_no_mangling_in_nested_scopes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsManglingTest::test_no_mangling_in_nested_scopes
"""Auto-ported test: TypeParamsManglingTest::test_no_mangling_in_nested_scopes (CPython 3.12 oracle)."""


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
ns = run_code('\n            from test.test_type_params import make_base\n\n            class __X:\n                pass\n\n            class Y[T: __X](\n                make_base(lambda: __X),\n                # doubly nested scope\n                make_base(lambda: (lambda: __X)),\n                # list comprehension\n                make_base([__X for _ in (1,)]),\n                # genexp\n                make_base(__X for _ in (1,)),\n            ):\n                pass\n        ')
Y = ns['Y']
T, = Y.__type_params__

assert T.__bound__ is ns['__X']
base0 = Y.__bases__[0]

assert base0.__arg__() is ns['__X']
base1 = Y.__bases__[1]

assert base1.__arg__()() is ns['__X']
base2 = Y.__bases__[2]

assert base2.__arg__ == [ns['__X']]
base3 = Y.__bases__[3]

assert list(base3.__arg__) == [ns['__X']]
print("TypeParamsManglingTest::test_no_mangling_in_nested_scopes: ok")
