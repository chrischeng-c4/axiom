# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "pickling_tests__test_object_reduce"
# subject = "cpython.test_descr.PicklingTests.test_object_reduce"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::PicklingTests::test_object_reduce
"""Auto-ported test: PicklingTests::test_object_reduce (CPython 3.12 oracle)."""


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
object().__reduce__()
try:
    object().__reduce__(0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
object().__reduce_ex__(0)
try:
    object().__reduce_ex__()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    object().__reduce_ex__(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PicklingTests::test_object_reduce: ok")
