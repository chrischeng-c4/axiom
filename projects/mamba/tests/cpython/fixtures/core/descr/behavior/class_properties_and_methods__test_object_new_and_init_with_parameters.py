# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_object_new_and_init_with_parameters"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_object_new_and_init_with_parameters"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_object_new_and_init_with_parameters
"""Auto-ported test: ClassPropertiesAndMethods::test_object_new_and_init_with_parameters (CPython 3.12 oracle)."""


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
class OverrideNeither:
    pass

try:
    OverrideNeither(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    OverrideNeither(kw=1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class OverrideNew:

    def __new__(cls, foo, kw=0, *args, **kwds):
        return object.__new__(cls, *args, **kwds)

class OverrideInit:

    def __init__(self, foo, kw=0, *args, **kwargs):
        return object.__init__(self, *args, **kwargs)

class OverrideBoth(OverrideNew, OverrideInit):
    pass
for case in (OverrideNew, OverrideInit, OverrideBoth):
    case(1)
    case(1, kw=2)

    try:
        case(1, 2, 3)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        case(1, 2, foo=3)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("ClassPropertiesAndMethods::test_object_new_and_init_with_parameters: ok")
