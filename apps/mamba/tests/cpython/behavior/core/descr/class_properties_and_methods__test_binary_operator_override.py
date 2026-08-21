# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_binary_operator_override"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_binary_operator_override"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_binary_operator_override
"""Auto-ported test: ClassPropertiesAndMethods::test_binary_operator_override (CPython 3.12 oracle)."""


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
class I(int):

    def __repr__(self):
        return 'I(%r)' % int(self)

    def __add__(self, other):
        return I(int(self) + int(other))
    __radd__ = __add__

    def __pow__(self, other, mod=None):
        if mod is None:
            return I(pow(int(self), int(other)))
        else:
            return I(pow(int(self), int(other), int(mod)))

    def __rpow__(self, other, mod=None):
        if mod is None:
            return I(pow(int(other), int(self), mod))
        else:
            return I(pow(int(other), int(self), int(mod)))

assert repr(I(1) + I(2)) == 'I(3)'

assert repr(I(1) + 2) == 'I(3)'

assert repr(1 + I(2)) == 'I(3)'

assert repr(I(2) ** I(3)) == 'I(8)'

assert repr(2 ** I(3)) == 'I(8)'

assert repr(I(2) ** 3) == 'I(8)'

assert repr(pow(I(2), I(3), I(5))) == 'I(3)'

class S(str):

    def __eq__(self, other):
        return self.lower() == other.lower()
print("ClassPropertiesAndMethods::test_binary_operator_override: ok")
