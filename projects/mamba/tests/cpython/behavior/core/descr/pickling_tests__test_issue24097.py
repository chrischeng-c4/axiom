# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "pickling_tests__test_issue24097"
# subject = "cpython.test_descr.PicklingTests.test_issue24097"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::PicklingTests::test_issue24097
"""Auto-ported test: PicklingTests::test_issue24097 (CPython 3.12 oracle)."""


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
class S(str):
    pass

class A:
    __slotnames__ = [S('spam')]

    def __getattr__(self, attr):
        if attr == 'spam':
            A.__slotnames__[:] = [S('spam')]
            return 42
        else:
            raise AttributeError
import copyreg
expected = (copyreg.__newobj__, (A,), (None, {'spam': 42}), None, None)

assert A().__reduce_ex__(2) == expected
print("PicklingTests::test_issue24097: ok")
