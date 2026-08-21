# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_subclass_right_op"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_subclass_right_op"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_subclass_right_op
"""Auto-ported test: ClassPropertiesAndMethods::test_subclass_right_op (CPython 3.12 oracle)."""


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
class B(int):

    def __floordiv__(self, other):
        return 'B.__floordiv__'

    def __rfloordiv__(self, other):
        return 'B.__rfloordiv__'

assert B(1) // 1 == 'B.__floordiv__'

assert 1 // B(1) == 'B.__rfloordiv__'

class C(object):

    def __floordiv__(self, other):
        return 'C.__floordiv__'

    def __rfloordiv__(self, other):
        return 'C.__rfloordiv__'

assert C() // 1 == 'C.__floordiv__'

assert 1 // C() == 'C.__rfloordiv__'

class D(C):

    def __floordiv__(self, other):
        return 'D.__floordiv__'

    def __rfloordiv__(self, other):
        return 'D.__rfloordiv__'

assert D() // C() == 'D.__floordiv__'

assert C() // D() == 'D.__rfloordiv__'

class E(C):
    pass

assert E.__rfloordiv__ == C.__rfloordiv__

assert E() // 1 == 'C.__floordiv__'

assert 1 // E() == 'C.__rfloordiv__'

assert E() // C() == 'C.__floordiv__'

assert C() // E() == 'C.__floordiv__'
print("ClassPropertiesAndMethods::test_subclass_right_op: ok")
