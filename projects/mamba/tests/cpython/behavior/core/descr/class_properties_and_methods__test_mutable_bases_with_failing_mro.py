# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_mutable_bases_with_failing_mro"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_mutable_bases_with_failing_mro"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_mutable_bases_with_failing_mro
"""Auto-ported test: ClassPropertiesAndMethods::test_mutable_bases_with_failing_mro (CPython 3.12 oracle)."""


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
class WorkOnce(type):

    def __new__(self, name, bases, ns):
        self.flag = 0
        return super(WorkOnce, self).__new__(WorkOnce, name, bases, ns)

    def mro(self):
        if self.flag > 0:
            raise RuntimeError('bozo')
        else:
            self.flag += 1
            return type.mro(self)

class WorkAlways(type):

    def mro(self):
        return type.mro(self)

class C(object):
    pass

class C2(object):
    pass

class D(C):
    pass

class E(D):
    pass

class F(D, metaclass=WorkOnce):
    pass

class G(D, metaclass=WorkAlways):
    pass
E_mro_before = E.__mro__
D_mro_before = D.__mro__
try:
    D.__bases__ = (C2,)
except RuntimeError:

    assert E.__mro__ == E_mro_before

    assert D.__mro__ == D_mro_before
else:

    raise AssertionError('exception not propagated')
print("ClassPropertiesAndMethods::test_mutable_bases_with_failing_mro: ok")
