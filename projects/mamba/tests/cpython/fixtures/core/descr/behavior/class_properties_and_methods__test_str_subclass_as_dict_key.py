# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_str_subclass_as_dict_key"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_str_subclass_as_dict_key"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_str_subclass_as_dict_key
"""Auto-ported test: ClassPropertiesAndMethods::test_str_subclass_as_dict_key (CPython 3.12 oracle)."""


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
class cistr(str):
    """Subclass of str that computes __eq__ case-insensitively.

            Also computes a hash code of the string in canonical form.
            """

    def __init__(self, value):
        self.canonical = value.lower()
        self.hashcode = hash(self.canonical)

    def __eq__(self, other):
        if not isinstance(other, cistr):
            other = cistr(other)
        return self.canonical == other.canonical

    def __hash__(self):
        return self.hashcode

assert cistr('ABC') == 'abc'

assert 'aBc' == cistr('ABC')

assert str(cistr('ABC')) == 'ABC'
d = {cistr('one'): 1, cistr('two'): 2, cistr('tHree'): 3}

assert d[cistr('one')] == 1

assert d[cistr('tWo')] == 2

assert d[cistr('THrEE')] == 3

assert cistr('ONe') in d

assert d.get(cistr('thrEE')) == 3
print("ClassPropertiesAndMethods::test_str_subclass_as_dict_key: ok")
