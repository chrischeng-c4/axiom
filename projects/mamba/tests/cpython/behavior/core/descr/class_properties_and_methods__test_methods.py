# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_methods"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_methods
"""Auto-ported test: ClassPropertiesAndMethods::test_methods (CPython 3.12 oracle)."""


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

    def __init__(self, x):
        self.x = x

    def foo(self):
        return self.x
c1 = C(1)

assert c1.foo() == 1

class D(C):
    boo = C.foo
    goo = c1.foo
d2 = D(2)

assert d2.foo() == 2

assert d2.boo() == 2

assert d2.goo() == 1

class E(object):
    foo = C.foo

assert E().foo.__func__ == C.foo

assert repr(C.foo.__get__(C(1))).startswith('<bound method ')
print("ClassPropertiesAndMethods::test_methods: ok")
