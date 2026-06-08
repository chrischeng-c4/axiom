# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_mro_disagreement"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_mro_disagreement"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_mro_disagreement
"""Auto-ported test: ClassPropertiesAndMethods::test_mro_disagreement (CPython 3.12 oracle)."""


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
mro_err_msg = 'Cannot create a consistent method resolution\norder (MRO) for bases '

def raises(exc, expected, callable, *args):
    try:
        callable(*args)
    except exc as msg:
        if support.check_impl_detail():
            if not str(msg).startswith(expected):
                self.fail('Message %r, expected %r' % (str(msg), expected))
    else:
        self.fail('Expected %s' % exc)

class A(object):
    pass

class B(A):
    pass

class C(object):
    pass
raises(TypeError, 'duplicate base class A', type, 'X', (A, A), {})
raises(TypeError, mro_err_msg, type, 'X', (A, B), {})
raises(TypeError, mro_err_msg, type, 'X', (A, C, B), {})

class GridLayout(object):
    pass

class HorizontalGrid(GridLayout):
    pass

class VerticalGrid(GridLayout):
    pass

class HVGrid(HorizontalGrid, VerticalGrid):
    pass

class VHGrid(VerticalGrid, HorizontalGrid):
    pass
raises(TypeError, mro_err_msg, type, 'ConfusedGrid', (HVGrid, VHGrid), {})
print("ClassPropertiesAndMethods::test_mro_disagreement: ok")
