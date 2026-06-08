# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_errors"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_errors
"""Auto-ported test: ClassPropertiesAndMethods::test_errors (CPython 3.12 oracle)."""


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

    class C(list, dict):
        pass
except TypeError:
    pass
else:

    raise AssertionError('inheritance from both list and dict should be illegal')
try:

    class C(object, None):
        pass
except TypeError:
    pass
else:

    raise AssertionError('inheritance from non-type should be illegal')

class Classic:
    pass
try:

    class C(type(len)):
        pass
except TypeError:
    pass
else:

    raise AssertionError('inheritance from CFunction should be illegal')
try:

    class C(object):
        __slots__ = 1
except TypeError:
    pass
else:

    raise AssertionError('__slots__ = 1 should be illegal')
try:

    class C(object):
        __slots__ = [1]
except TypeError:
    pass
else:

    raise AssertionError('__slots__ = [1] should be illegal')

class M1(type):
    pass

class M2(type):
    pass

class A1(object, metaclass=M1):
    pass

class A2(object, metaclass=M2):
    pass
try:

    class B(A1, A2):
        pass
except TypeError:
    pass
else:

    raise AssertionError('finding the most derived metaclass should have failed')
print("ClassPropertiesAndMethods::test_errors: ok")
