# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_object_new"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_object_new"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_object_new
"""Auto-ported test: ClassPropertiesAndMethods::test_object_new (CPython 3.12 oracle)."""


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
    pass
object.__new__(A)

try:
    object.__new__(A, 5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
object.__init__(A())

try:
    object.__init__(A(), 5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class A(object):

    def __init__(self, foo):
        self.foo = foo
object.__new__(A)
object.__new__(A, 5)
object.__init__(A(3))

try:
    object.__init__(A(3), 5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class A(object):

    def __new__(cls, foo):
        return object.__new__(cls)
object.__new__(A)

try:
    object.__new__(A, 5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
object.__init__(A(3))
object.__init__(A(3), 5)

class A(object):

    def __new__(cls, foo):
        return object.__new__(cls)

    def __init__(self, foo):
        self.foo = foo
object.__new__(A)

try:
    object.__new__(A, 5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
object.__init__(A(3))

try:
    object.__init__(A(3), 5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ClassPropertiesAndMethods::test_object_new: ok")
