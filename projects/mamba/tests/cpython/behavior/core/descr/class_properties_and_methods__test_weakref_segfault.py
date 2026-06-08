# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_weakref_segfault"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_weakref_segfault"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_weakref_segfault
"""Auto-ported test: ClassPropertiesAndMethods::test_weakref_segfault (CPython 3.12 oracle)."""


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
import weakref

class Provoker:

    def __init__(self, referrent):
        self.ref = weakref.ref(referrent)

    def __del__(self):
        x = self.ref()

class Oops(object):
    pass
o = Oops()
o.whatever = Provoker(o)
del o
print("ClassPropertiesAndMethods::test_weakref_segfault: ok")
