# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_slot_shadows_class_variable"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_slot_shadows_class_variable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_slot_shadows_class_variable
"""Auto-ported test: ClassPropertiesAndMethods::test_slot_shadows_class_variable (CPython 3.12 oracle)."""


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
try:

    class X:
        __slots__ = ['foo']
        foo = None
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
m = str(cm.exception)

assert "'foo' in __slots__ conflicts with class variable" == m
print("ClassPropertiesAndMethods::test_slot_shadows_class_variable: ok")
