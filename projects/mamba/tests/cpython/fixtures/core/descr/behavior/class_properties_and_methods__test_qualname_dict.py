# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_qualname_dict"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_qualname_dict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_qualname_dict
"""Auto-ported test: ClassPropertiesAndMethods::test_qualname_dict (CPython 3.12 oracle)."""


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
ns = {'__qualname__': 'some.name'}
tp = type('Foo', (), ns)

assert tp.__qualname__ == 'some.name'

assert '__qualname__' not in tp.__dict__

assert ns == {'__qualname__': 'some.name'}
ns = {'__qualname__': 1}

try:
    type('Foo', (), ns)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ClassPropertiesAndMethods::test_qualname_dict: ok")
