# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_module_subclasses"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_module_subclasses"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_module_subclasses
"""Auto-ported test: ClassPropertiesAndMethods::test_module_subclasses (CPython 3.12 oracle)."""


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
log = []
MT = type(sys)

class MM(MT):

    def __init__(self, name):
        MT.__init__(self, name)

    def __getattribute__(self, name):
        log.append(('getattr', name))
        return MT.__getattribute__(self, name)

    def __setattr__(self, name, value):
        log.append(('setattr', name, value))
        MT.__setattr__(self, name, value)

    def __delattr__(self, name):
        log.append(('delattr', name))
        MT.__delattr__(self, name)
a = MM('a')
a.foo = 12
x = a.foo
del a.foo

assert log == [('setattr', 'foo', 12), ('getattr', 'foo'), ('delattr', 'foo')]
try:

    class Module(types.ModuleType, str):
        pass
except TypeError:
    pass
else:

    raise AssertionError('inheriting from ModuleType and str at the same time should fail')

def random_name():
    return ''.join(random.choices(string.ascii_letters, k=10))

class A:
    pass
subclasses = [type(random_name(), (A,), {}) for i in range(100)]

assert A.__subclasses__() == subclasses
print("ClassPropertiesAndMethods::test_module_subclasses: ok")
