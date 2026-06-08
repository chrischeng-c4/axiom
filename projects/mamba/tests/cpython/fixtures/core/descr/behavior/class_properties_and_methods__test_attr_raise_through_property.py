# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_attr_raise_through_property"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_attr_raise_through_property"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_attr_raise_through_property
"""Auto-ported test: ClassPropertiesAndMethods::test_attr_raise_through_property (CPython 3.12 oracle)."""


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

    def __getattr__(self, name):
        raise ValueError('FOO')

    @property
    def foo(self):
        return self.__getattr__('asdf')
try:
    A().foo
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('FOO', str(_aR_e))

class B:

    @property
    def __getattr__(self, name):
        raise ValueError('FOO')

    @property
    def foo(self):
        raise NotImplementedError('BAR')
try:
    B().foo
    raise AssertionError('expected NotImplementedError')
except NotImplementedError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('BAR', str(_aR_e))
print("ClassPropertiesAndMethods::test_attr_raise_through_property: ok")
