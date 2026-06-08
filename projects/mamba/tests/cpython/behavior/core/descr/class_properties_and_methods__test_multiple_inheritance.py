# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_multiple_inheritance"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_multiple_inheritance"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_multiple_inheritance
"""Auto-ported test: ClassPropertiesAndMethods::test_multiple_inheritance (CPython 3.12 oracle)."""


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
class C(object):

    def __init__(self):
        self.__state = 0

    def getstate(self):
        return self.__state

    def setstate(self, state):
        self.__state = state
a = C()

assert a.getstate() == 0
a.setstate(10)

assert a.getstate() == 10

class D(dict, C):

    def __init__(self):
        dict.__init__(self)
        C.__init__(self)
d = D()

assert list(d.keys()) == []
d['hello'] = 'world'

assert list(d.items()) == [('hello', 'world')]

assert d['hello'] == 'world'

assert d.getstate() == 0
d.setstate(10)

assert d.getstate() == 10

assert D.__mro__ == (D, dict, C, object)

class Node(object):

    def __int__(self):
        return int(self.foo())

    def foo(self):
        return '23'

class Frag(Node, list):

    def foo(self):
        return '42'

assert Node().__int__() == 23

assert int(Node()) == 23

assert Frag().__int__() == 42

assert int(Frag()) == 42
print("ClassPropertiesAndMethods::test_multiple_inheritance: ok")
