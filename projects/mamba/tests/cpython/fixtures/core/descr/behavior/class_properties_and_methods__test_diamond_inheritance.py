# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_diamond_inheritance"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_diamond_inheritance"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_diamond_inheritance
"""Auto-ported test: ClassPropertiesAndMethods::test_diamond_inheritance (CPython 3.12 oracle)."""


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
class A(object):

    def spam(self):
        return 'A'

assert A().spam() == 'A'

class B(A):

    def boo(self):
        return 'B'

    def spam(self):
        return 'B'

assert B().spam() == 'B'

assert B().boo() == 'B'

class C(A):

    def boo(self):
        return 'C'

assert C().spam() == 'A'

assert C().boo() == 'C'

class D(B, C):
    pass

assert D().spam() == 'B'

assert D().boo() == 'B'

assert D.__mro__ == (D, B, C, A, object)

class E(C, B):
    pass

assert E().spam() == 'B'

assert E().boo() == 'C'

assert E.__mro__ == (E, C, B, A, object)
try:

    class F(D, E):
        pass
except TypeError:
    pass
else:

    raise AssertionError('expected MRO order disagreement (F)')
try:

    class G(E, D):
        pass
except TypeError:
    pass
else:

    raise AssertionError('expected MRO order disagreement (G)')
print("ClassPropertiesAndMethods::test_diamond_inheritance: ok")
