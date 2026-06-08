# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_builtin_bases"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_builtin_bases"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_builtin_bases
"""Auto-ported test: ClassPropertiesAndMethods::test_builtin_bases (CPython 3.12 oracle)."""


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
builtin_types = [tp for tp in builtins.__dict__.values() if isinstance(tp, type)]
for tp in builtin_types:
    object.__getattribute__(tp, '__bases__')
    if tp is not object:
        if tp is ExceptionGroup:
            num_bases = 2
        else:
            num_bases = 1

        assert len(tp.__bases__) == num_bases

class L(list):
    pass

class C(object):
    pass

class D(C):
    pass
try:
    L.__bases__ = (dict,)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't turn list subclass into dict subclass")
try:
    list.__bases__ = (dict,)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't be able to assign to list.__bases__")
try:
    D.__bases__ = (C, list)
except TypeError:
    pass
else:

    raise AssertionError('best_base calculation found wanting')
print("ClassPropertiesAndMethods::test_builtin_bases: ok")
