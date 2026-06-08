# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_ipow_returns_not_implemented"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_ipow_returns_not_implemented"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_ipow_returns_not_implemented
"""Auto-ported test: ClassPropertiesAndMethods::test_ipow_returns_not_implemented (CPython 3.12 oracle)."""


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
class A:

    def __ipow__(self, other):
        return NotImplemented

class B(A):

    def __rpow__(self, other):
        return 1

class C(A):

    def __pow__(self, other):
        return 2
a = A()
b = B()
c = C()
a **= b

assert a == 1
c **= b

assert c == 2
print("ClassPropertiesAndMethods::test_ipow_returns_not_implemented: ok")
