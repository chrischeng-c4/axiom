# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "shared_key_tests__test_subclasses"
# subject = "cpython.test_descr.SharedKeyTests.test_subclasses"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::SharedKeyTests::test_subclasses
"""Auto-ported test: SharedKeyTests::test_subclasses (CPython 3.12 oracle)."""


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
class A:
    pass

class B(A):
    pass
[(A(), B()) for _ in range(30)]
a, b = (A(), B())

assert sys.getsizeof(vars(a)) == sys.getsizeof(vars(b))

assert sys.getsizeof(vars(a)) < sys.getsizeof({'a': 1})
a.x, a.y, a.z, a.w, a.v, a.u = range(6)

assert sys.getsizeof(vars(a)) != sys.getsizeof(vars(b))
a2 = A()

assert sys.getsizeof(vars(a)) > sys.getsizeof(vars(a2))

assert sys.getsizeof(vars(a2)) < sys.getsizeof({'a': 1})

assert sys.getsizeof(vars(b)) < sys.getsizeof({'a': 1})
print("SharedKeyTests::test_subclasses: ok")
