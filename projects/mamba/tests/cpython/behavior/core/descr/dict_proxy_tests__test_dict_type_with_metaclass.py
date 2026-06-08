# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "dict_proxy_tests__test_dict_type_with_metaclass"
# subject = "cpython.test_descr.DictProxyTests.test_dict_type_with_metaclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::DictProxyTests::test_dict_type_with_metaclass
"""Auto-ported test: DictProxyTests::test_dict_type_with_metaclass (CPython 3.12 oracle)."""


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
class C(object):

    def meth(self):
        pass
self_C = C

class B(object):
    pass

class M(type):
    pass

class C(metaclass=M):
    pass

assert type(C.__dict__) == type(B.__dict__)
print("DictProxyTests::test_dict_type_with_metaclass: ok")
