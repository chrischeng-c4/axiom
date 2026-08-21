# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_params"
# dimension = "behavior"
# case = "type_params_pickle_test__test_pickling_classes"
# subject = "cpython.test_type_params.TypeParamsPickleTest.test_pickling_classes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_params.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_params.py::TypeParamsPickleTest::test_pickling_classes
"""Auto-ported test: TypeParamsPickleTest::test_pickling_classes (CPython 3.12 oracle)."""


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
things_to_test = [Class1, Class1[int], Class1[T], Class2, Class2[int, T], Class2[T, int], Class2[int, str], Class3, Class3[int, T, str, bytes, [float, object, T]], Class4, Class4[int, bytes], Class4[T, bytes], Class4[int, T], Class4[T, T]]
for thing in things_to_test:
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        pickled = pickle.dumps(thing, protocol=proto)

        assert pickle.loads(pickled) == thing
for klass in things_to_test:
    real_class = getattr(klass, '__origin__', klass)
    thing = klass()
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        pickled = pickle.dumps(thing, protocol=proto)

        assert isinstance(pickle.loads(pickled), real_class)
print("TypeParamsPickleTest::test_pickling_classes: ok")
