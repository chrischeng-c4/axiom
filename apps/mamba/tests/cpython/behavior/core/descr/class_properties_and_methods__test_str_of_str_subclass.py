# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_str_of_str_subclass"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_str_of_str_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_str_of_str_subclass
"""Auto-ported test: ClassPropertiesAndMethods::test_str_of_str_subclass (CPython 3.12 oracle)."""


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
import binascii

class octetstring(str):

    def __str__(self):
        return binascii.b2a_hex(self.encode('ascii')).decode('ascii')

    def __repr__(self):
        return self + ' repr'
o = octetstring('A')

assert type(o) == octetstring

assert type(str(o)) == str

assert type(repr(o)) == str

assert ord(o) == 65

assert str(o) == '41'

assert repr(o) == 'A repr'

assert o.__str__() == '41'

assert o.__repr__() == 'A repr'
print("ClassPropertiesAndMethods::test_str_of_str_subclass: ok")
