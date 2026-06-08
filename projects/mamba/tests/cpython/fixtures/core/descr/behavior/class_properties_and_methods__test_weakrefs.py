# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_weakrefs"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_weakrefs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_weakrefs
"""Auto-ported test: ClassPropertiesAndMethods::test_weakrefs (CPython 3.12 oracle)."""


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
import weakref

class C(object):
    pass
c = C()
r = weakref.ref(c)

assert r() == c
del c
support.gc_collect()

assert r() == None
del r

class NoWeak(object):
    __slots__ = ['foo']
no = NoWeak()
try:
    weakref.ref(no)
except TypeError as msg:

    assert 'weak reference' in str(msg)
else:

    raise AssertionError('weakref.ref(no) should be illegal')

class Weak(object):
    __slots__ = ['foo', '__weakref__']
yes = Weak()
r = weakref.ref(yes)

assert r() == yes
del yes
support.gc_collect()

assert r() == None
del r
print("ClassPropertiesAndMethods::test_weakrefs: ok")
