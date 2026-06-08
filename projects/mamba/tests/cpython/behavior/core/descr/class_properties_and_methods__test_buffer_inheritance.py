# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_buffer_inheritance"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_buffer_inheritance"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_buffer_inheritance
"""Auto-ported test: ClassPropertiesAndMethods::test_buffer_inheritance (CPython 3.12 oracle)."""


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
import binascii

class MyBytes(bytes):
    pass
base = b'abc'
m = MyBytes(base)

assert binascii.b2a_hex(m) == binascii.b2a_hex(base)

class MyInt(int):
    pass
m = MyInt(42)
try:
    binascii.b2a_hex(m)

    raise AssertionError('subclass of int should not have a buffer interface')
except TypeError:
    pass
print("ClassPropertiesAndMethods::test_buffer_inheritance: ok")
