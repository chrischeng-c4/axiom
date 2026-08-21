# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_lazy_evaluation_test__test_recursive_class"
# subject = "cpython.test_type_params.TypeParamsLazyEvaluationTest.test_recursive_class"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsLazyEvaluationTest::test_recursive_class
"""Auto-ported test: TypeParamsLazyEvaluationTest::test_recursive_class (CPython 3.12 oracle)."""


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
class Foo[T: Foo, U: (Foo, Foo)]:
    pass
type_params = Foo.__type_params__

assert len(type_params) == 2

assert type_params[0].__name__ == 'T'

assert type_params[0].__bound__ is Foo

assert type_params[0].__constraints__ == ()

assert type_params[1].__name__ == 'U'

assert type_params[1].__bound__ is None

assert type_params[1].__constraints__ == (Foo, Foo)
print("TypeParamsLazyEvaluationTest::test_recursive_class: ok")
