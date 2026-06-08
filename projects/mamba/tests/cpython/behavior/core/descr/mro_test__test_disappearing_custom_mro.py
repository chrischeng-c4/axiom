# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "mro_test__test_disappearing_custom_mro"
# subject = "cpython.test_descr.MroTest.test_disappearing_custom_mro"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::MroTest::test_disappearing_custom_mro
"""Auto-ported test: MroTest::test_disappearing_custom_mro (CPython 3.12 oracle)."""


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
self_step = 0
self_ready = False
'\n        gh-92112: A custom mro() returning a result conflicting with\n        __bases__ and deleting itself caused a double free.\n        '

class B:
    pass

class M(DebugHelperMeta):

    def mro(cls):
        del M.mro
        return (B,)
try:

    class A(metaclass=M):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("MroTest::test_disappearing_custom_mro: ok")
