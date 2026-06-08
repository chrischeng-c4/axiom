# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_unsubclassable_types"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_unsubclassable_types"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_unsubclassable_types
"""Auto-ported test: ClassPropertiesAndMethods::test_unsubclassable_types (CPython 3.12 oracle)."""


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
try:

    class X(type(None)):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:

    class X(object, type(None)):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:

    class X(type(None), object):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class O(object):
    pass
try:

    class X(O, type(None)):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:

    class X(type(None), O):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class X(object):
    pass
try:
    X.__bases__ = (type(None),)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    X.__bases__ = (object, type(None))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    X.__bases__ = (type(None), object)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    X.__bases__ = (O, type(None))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    X.__bases__ = (type(None), O)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ClassPropertiesAndMethods::test_unsubclassable_types: ok")
