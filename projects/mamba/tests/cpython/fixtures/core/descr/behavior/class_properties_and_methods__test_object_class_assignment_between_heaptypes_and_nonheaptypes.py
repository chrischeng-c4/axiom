# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_object_class_assignment_between_heaptypes_and_nonheaptypes"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_object_class_assignment_between_heaptypes_and_nonheaptypes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_object_class_assignment_between_heaptypes_and_nonheaptypes
"""Auto-ported test: ClassPropertiesAndMethods::test_object_class_assignment_between_heaptypes_and_nonheaptypes (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class SubType(types.ModuleType):
    a = 1
m = types.ModuleType('m')

assert m.__class__ is types.ModuleType

assert not hasattr(m, 'a')
m.__class__ = SubType

assert m.__class__ is SubType

assert hasattr(m, 'a')
m.__class__ = types.ModuleType

assert m.__class__ is types.ModuleType

assert not hasattr(m, 'a')

class MyInt(int):
    __slots__ = ()
try:
    1 .__class__ = MyInt
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class MyFloat(float):
    __slots__ = ()
try:
    1.0.__class__ = MyFloat
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class MyComplex(complex):
    __slots__ = ()
try:
    (1 + 2j).__class__ = MyComplex
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class MyStr(str):
    __slots__ = ()
try:
    'a'.__class__ = MyStr
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class MyBytes(bytes):
    __slots__ = ()
try:
    b'a'.__class__ = MyBytes
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class MyTuple(tuple):
    __slots__ = ()
try:
    ().__class__ = MyTuple
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class MyFrozenSet(frozenset):
    __slots__ = ()
try:
    frozenset().__class__ = MyFrozenSet
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ClassPropertiesAndMethods::test_object_class_assignment_between_heaptypes_and_nonheaptypes: ok")
