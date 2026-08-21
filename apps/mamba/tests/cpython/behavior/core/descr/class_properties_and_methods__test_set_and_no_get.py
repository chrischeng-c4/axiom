# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_set_and_no_get"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_set_and_no_get"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_set_and_no_get
"""Auto-ported test: ClassPropertiesAndMethods::test_set_and_no_get (CPython 3.12 oracle)."""


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
class Descr(object):

    def __init__(self, name):
        self.name = name

    def __set__(self, obj, value):
        obj.__dict__[self.name] = value
descr = Descr('a')

class X(object):
    a = descr
x = X()

assert x.a is descr
x.a = 42

assert x.a == 42

class Meta(type):
    pass

class X(metaclass=Meta):
    pass
X.a = 42
Meta.a = Descr('a')

assert X.a == 42
print("ClassPropertiesAndMethods::test_set_and_no_get: ok")
