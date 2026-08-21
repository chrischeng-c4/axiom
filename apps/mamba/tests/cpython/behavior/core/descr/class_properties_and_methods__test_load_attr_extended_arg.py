# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_load_attr_extended_arg"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_load_attr_extended_arg"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_load_attr_extended_arg
"""Auto-ported test: ClassPropertiesAndMethods::test_load_attr_extended_arg (CPython 3.12 oracle)."""


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
class Numbers:

    def __getattr__(self, attr):
        return int(attr.lstrip('_'))
attrs = ', '.join((f'Z._{n:03d}' for n in range(280)))
code = f'def number_attrs(Z):\n    return [ {attrs} ]'
ns = {}
exec(code, ns)
number_attrs = ns['number_attrs']
for _ in range(30):

    assert number_attrs(Numbers()) == list(range(280))
print("ClassPropertiesAndMethods::test_load_attr_extended_arg: ok")
