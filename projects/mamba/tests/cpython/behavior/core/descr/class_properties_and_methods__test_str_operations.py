# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_str_operations"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_str_operations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_str_operations
"""Auto-ported test: ClassPropertiesAndMethods::test_str_operations (CPython 3.12 oracle)."""


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
    'a' + 5
except TypeError:
    pass
else:

    raise AssertionError("'' + 5 doesn't raise TypeError")
try:
    ''.split('')
except ValueError:
    pass
else:

    raise AssertionError("''.split('') doesn't raise ValueError")
try:
    ''.join([0])
except TypeError:
    pass
else:

    raise AssertionError("''.join([0]) doesn't raise TypeError")
try:
    ''.rindex('5')
except ValueError:
    pass
else:

    raise AssertionError("''.rindex('5') doesn't raise ValueError")
try:
    '%(n)s' % None
except TypeError:
    pass
else:

    raise AssertionError("'%(n)s' % None doesn't raise TypeError")
try:
    '%(n' % {}
except ValueError:
    pass
else:

    raise AssertionError("'%(n' % {} '' doesn't raise ValueError")
try:
    '%*s' % 'abc'
except TypeError:
    pass
else:

    raise AssertionError("'%*s' % ('abc') doesn't raise TypeError")
try:
    '%*.*s' % ('abc', 5)
except TypeError:
    pass
else:

    raise AssertionError("'%*.*s' % ('abc', 5) doesn't raise TypeError")
try:
    '%s' % (1, 2)
except TypeError:
    pass
else:

    raise AssertionError("'%s' % (1, 2) doesn't raise TypeError")
try:
    '%' % None
except ValueError:
    pass
else:

    raise AssertionError("'%' % None doesn't raise ValueError")

assert '534253'.isdigit() == 1

assert '534253x'.isdigit() == 0

assert '%c' % 5 == '\x05'

assert '%c' % '5' == '5'
print("ClassPropertiesAndMethods::test_str_operations: ok")
