# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_meth_class_get"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_meth_class_get"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_meth_class_get
"""Auto-ported test: ClassPropertiesAndMethods::test_meth_class_get (CPython 3.12 oracle)."""


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
arg = [1, 2, 3]
res = {1: None, 2: None, 3: None}

assert dict.fromkeys(arg) == res

assert {}.fromkeys(arg) == res
descr = dict.__dict__['fromkeys']

assert descr.__get__(None, dict)(arg) == res

assert descr.__get__({})(arg) == res
try:
    descr.__get__(None, None)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't have allowed descr.__get__(None, None)")
try:
    descr.__get__(42)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't have allowed descr.__get__(42)")
try:
    descr.__get__(None, 42)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't have allowed descr.__get__(None, 42)")
try:
    descr.__get__(None, int)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't have allowed descr.__get__(None, int)")
print("ClassPropertiesAndMethods::test_meth_class_get: ok")
