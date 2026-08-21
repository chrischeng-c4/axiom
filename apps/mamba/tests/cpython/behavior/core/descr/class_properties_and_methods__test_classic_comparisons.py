# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_classic_comparisons"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_classic_comparisons"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_classic_comparisons
"""Auto-ported test: ClassPropertiesAndMethods::test_classic_comparisons (CPython 3.12 oracle)."""


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
class classic:
    pass
for base in (classic, int, object):

    class C(base):

        def __init__(self, value):
            self.value = int(value)

        def __eq__(self, other):
            if isinstance(other, C):
                return self.value == other.value
            if isinstance(other, int) or isinstance(other, int):
                return self.value == other
            return NotImplemented

        def __ne__(self, other):
            if isinstance(other, C):
                return self.value != other.value
            if isinstance(other, int) or isinstance(other, int):
                return self.value != other
            return NotImplemented

        def __lt__(self, other):
            if isinstance(other, C):
                return self.value < other.value
            if isinstance(other, int) or isinstance(other, int):
                return self.value < other
            return NotImplemented

        def __le__(self, other):
            if isinstance(other, C):
                return self.value <= other.value
            if isinstance(other, int) or isinstance(other, int):
                return self.value <= other
            return NotImplemented

        def __gt__(self, other):
            if isinstance(other, C):
                return self.value > other.value
            if isinstance(other, int) or isinstance(other, int):
                return self.value > other
            return NotImplemented

        def __ge__(self, other):
            if isinstance(other, C):
                return self.value >= other.value
            if isinstance(other, int) or isinstance(other, int):
                return self.value >= other
            return NotImplemented
    c1 = C(1)
    c2 = C(2)
    c3 = C(3)

    assert c1 == 1
    c = {1: c1, 2: c2, 3: c3}
    for x in (1, 2, 3):
        for y in (1, 2, 3):
            for op in ('<', '<=', '==', '!=', '>', '>='):

                assert eval('c[x] %s c[y]' % op) == eval('x %s y' % op)

                assert eval('c[x] %s y' % op) == eval('x %s y' % op)

                assert eval('x %s c[y]' % op) == eval('x %s y' % op)
print("ClassPropertiesAndMethods::test_classic_comparisons: ok")
