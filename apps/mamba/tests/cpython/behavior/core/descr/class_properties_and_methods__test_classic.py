# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_classic"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_classic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_classic
"""Auto-ported test: ClassPropertiesAndMethods::test_classic (CPython 3.12 oracle)."""


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
class C:

    def foo(*a):
        return a
    goo = classmethod(foo)
c = C()

assert C.goo(1) == (C, 1)

assert c.goo(1) == (C, 1)

assert c.foo(1) == (c, 1)

class D(C):
    pass
d = D()

assert D.goo(1) == (D, 1)

assert d.goo(1) == (D, 1)

assert d.foo(1) == (d, 1)

assert D.foo(d, 1) == (d, 1)

class E:
    foo = C.foo

assert E().foo.__func__ == C.foo

assert repr(C.foo.__get__(C())).startswith('<bound method ')
print("ClassPropertiesAndMethods::test_classic: ok")
