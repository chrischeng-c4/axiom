# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "misc_tests__test_type_lookup_mro_reference"
# subject = "cpython.test_descr.MiscTests.test_type_lookup_mro_reference"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::MiscTests::test_type_lookup_mro_reference
"""Auto-ported test: MiscTests::test_type_lookup_mro_reference (CPython 3.12 oracle)."""


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
class MyKey(object):

    def __hash__(self):
        return hash('mykey')

    def __eq__(self, other):
        X.__bases__ = (Base2,)

class Base(object):
    mykey = 'from Base'
    mykey2 = 'from Base'

class Base2(object):
    mykey = 'from Base2'
    mykey2 = 'from Base2'
X = type('X', (Base,), {MyKey(): 5})

assert X.mykey == 'from Base'

assert X.mykey2 == 'from Base2'
print("MiscTests::test_type_lookup_mro_reference: ok")
