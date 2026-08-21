# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_init"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_init"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_init
"""Auto-ported test: ClassPropertiesAndMethods::test_init (CPython 3.12 oracle)."""


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
class Foo(object):

    def __init__(self):
        return 10
try:
    Foo()
except TypeError:
    pass
else:

    raise AssertionError('did not test __init__() for None return')
print("ClassPropertiesAndMethods::test_init: ok")
