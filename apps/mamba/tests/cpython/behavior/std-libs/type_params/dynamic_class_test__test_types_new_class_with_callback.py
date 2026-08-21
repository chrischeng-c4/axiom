# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "dynamic_class_test__test_types_new_class_with_callback"
# subject = "cpython.test_type_params.DynamicClassTest.test_types_new_class_with_callback"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::DynamicClassTest::test_types_new_class_with_callback
"""Auto-ported test: DynamicClassTest::test_types_new_class_with_callback (CPython 3.12 oracle)."""


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
def _set_type_params(ns, params):
    ns['__type_params__'] = params
T = TypeVar('T', infer_variance=True)
Klass = types.new_class('Klass', (Generic[T],), {}, lambda ns: _set_type_params(ns, (T,)))

assert Klass.__bases__ == (Generic,)

assert Klass.__orig_bases__ == (Generic[T],)

assert Klass.__type_params__ == (T,)

assert Klass.__parameters__ == (T,)
print("DynamicClassTest::test_types_new_class_with_callback: ok")
