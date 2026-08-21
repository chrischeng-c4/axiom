# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_slices"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_slices"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_slices
"""Auto-ported test: ClassPropertiesAndMethods::test_slices (CPython 3.12 oracle)."""


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

assert 'hello'[:4] == 'hell'

assert 'hello'[slice(4)] == 'hell'

assert str.__getitem__('hello', slice(4)) == 'hell'

class S(str):

    def __getitem__(self, x):
        return str.__getitem__(self, x)

assert S('hello')[:4] == 'hell'

assert S('hello')[slice(4)] == 'hell'

assert S('hello').__getitem__(slice(4)) == 'hell'

assert (1, 2, 3)[:2] == (1, 2)

assert (1, 2, 3)[slice(2)] == (1, 2)

assert tuple.__getitem__((1, 2, 3), slice(2)) == (1, 2)

class T(tuple):

    def __getitem__(self, x):
        return tuple.__getitem__(self, x)

assert T((1, 2, 3))[:2] == (1, 2)

assert T((1, 2, 3))[slice(2)] == (1, 2)

assert T((1, 2, 3)).__getitem__(slice(2)) == (1, 2)

assert [1, 2, 3][:2] == [1, 2]

assert [1, 2, 3][slice(2)] == [1, 2]

assert list.__getitem__([1, 2, 3], slice(2)) == [1, 2]

class L(list):

    def __getitem__(self, x):
        return list.__getitem__(self, x)

assert L([1, 2, 3])[:2] == [1, 2]

assert L([1, 2, 3])[slice(2)] == [1, 2]

assert L([1, 2, 3]).__getitem__(slice(2)) == [1, 2]
a = L([1, 2, 3])
a[slice(1, 3)] = [3, 2]

assert a == [1, 3, 2]
a[slice(0, 2, 1)] = [3, 1]

assert a == [3, 1, 2]
a.__setitem__(slice(1, 3), [2, 1])

assert a == [3, 2, 1]
a.__setitem__(slice(0, 2, 1), [2, 3])

assert a == [2, 3, 1]
print("ClassPropertiesAndMethods::test_slices: ok")
