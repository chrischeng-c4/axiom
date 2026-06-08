# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_set_doc"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_set_doc"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_set_doc
"""Auto-ported test: ClassPropertiesAndMethods::test_set_doc (CPython 3.12 oracle)."""


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
class X:
    """elephant"""
X.__doc__ = 'banana'

assert X.__doc__ == 'banana'
try:
    type(list).__dict__['__doc__'].__set__(list, 'blah')
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert "cannot set '__doc__' attribute of immutable type 'list'" in str(cm.exception)
try:
    type(X).__dict__['__doc__'].__delete__(X)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert "cannot delete '__doc__' attribute of immutable type 'X'" in str(cm.exception)

assert X.__doc__ == 'banana'
print("ClassPropertiesAndMethods::test_set_doc: ok")
