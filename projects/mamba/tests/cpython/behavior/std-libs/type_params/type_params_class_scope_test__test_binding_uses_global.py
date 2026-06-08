# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_class_scope_test__test_binding_uses_global"
# subject = "cpython.test_type_params.TypeParamsClassScopeTest.test_binding_uses_global"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsClassScopeTest::test_binding_uses_global
"""Auto-ported test: TypeParamsClassScopeTest::test_binding_uses_global (CPython 3.12 oracle)."""


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
ns = run_code('\n            x = "global"\n            def outer():\n                x = "nonlocal"\n                class Cls:\n                    type Alias = x\n                    val = Alias.__value__\n                    def meth[T: x](self, arg: x): ...\n                    bound = meth.__type_params__[0].__bound__\n                    annotation = meth.__annotations__["arg"]\n                    x = "class"\n                return Cls\n        ')
cls = ns['outer']()

assert cls.val == 'global'

assert cls.bound == 'global'

assert cls.annotation == 'global'
print("TypeParamsClassScopeTest::test_binding_uses_global: ok")
