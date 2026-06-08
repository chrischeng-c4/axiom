# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_mutable_bases_catch_mro_conflict"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_mutable_bases_catch_mro_conflict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_mutable_bases_catch_mro_conflict
"""Auto-ported test: ClassPropertiesAndMethods::test_mutable_bases_catch_mro_conflict (CPython 3.12 oracle)."""


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
class A(object):
    pass

class B(object):
    pass

class C(A, B):
    pass

class D(A, B):
    pass

class E(C, D):
    pass
try:
    C.__bases__ = (B, A)
except TypeError:
    pass
else:

    raise AssertionError("didn't catch MRO conflict")
print("ClassPropertiesAndMethods::test_mutable_bases_catch_mro_conflict: ok")
