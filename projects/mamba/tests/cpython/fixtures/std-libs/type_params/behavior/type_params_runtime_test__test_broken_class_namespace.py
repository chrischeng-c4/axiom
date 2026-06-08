# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_runtime_test__test_broken_class_namespace"
# subject = "cpython.test_type_params.TypeParamsRuntimeTest.test_broken_class_namespace"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsRuntimeTest::test_broken_class_namespace
"""Auto-ported test: TypeParamsRuntimeTest::test_broken_class_namespace (CPython 3.12 oracle)."""


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
code = '\n        class WeirdMapping(dict):\n            def __missing__(self, key):\n                if key == "T":\n                    raise RuntimeError\n                raise KeyError(key)\n\n        class Meta(type):\n            def __prepare__(name, bases):\n                return WeirdMapping()\n\n        class MyClass[V](metaclass=Meta):\n            class Inner[U](T):\n                pass\n        '
try:
    run_code(code)
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("TypeParamsRuntimeTest::test_broken_class_namespace: ok")
