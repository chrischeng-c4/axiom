# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_deepcopy_recursive"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_deepcopy_recursive"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_deepcopy_recursive
"""Auto-ported test: ClassPropertiesAndMethods::test_deepcopy_recursive (CPython 3.12 oracle)."""


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
class Node:
    pass
a = Node()
b = Node()
a.b = b
b.a = a
z = deepcopy(a)
print("ClassPropertiesAndMethods::test_deepcopy_recursive: ok")
