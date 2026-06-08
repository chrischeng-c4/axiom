# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_builtin_function_or_method"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_builtin_function_or_method"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_builtin_function_or_method
"""Auto-ported test: ClassPropertiesAndMethods::test_builtin_function_or_method (CPython 3.12 oracle)."""


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
def assertNotOrderable(a, b):
    try:
        a < b
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        a > b
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        a <= b
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        a >= b
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
l = []

assert l.append == l.append

assert not l.append != l.append

assert not l.append == [].append

assert l.append != [].append

assert not l.append == l.pop

assert l.append != l.pop
assertNotOrderable(l.append, l.append)

assert l.append.__name__ == 'append'

assert l.append.__self__ is l

assert l.append.__doc__ == list.append.__doc__
hash(l.append)
print("ClassPropertiesAndMethods::test_builtin_function_or_method: ok")
