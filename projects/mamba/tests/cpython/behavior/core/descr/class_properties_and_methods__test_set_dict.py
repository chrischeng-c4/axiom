# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_set_dict"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_set_dict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_set_dict
"""Auto-ported test: ClassPropertiesAndMethods::test_set_dict (CPython 3.12 oracle)."""


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
    pass
a = C()
a.__dict__ = {'b': 1}

assert a.b == 1

def cant(x, dict):
    try:
        x.__dict__ = dict
    except (AttributeError, TypeError):
        pass
    else:
        self.fail("shouldn't allow %r.__dict__ = %r" % (x, dict))
cant(a, None)
cant(a, [])
cant(a, 1)
del a.__dict__

class Base(object):
    pass

def verify_dict_readonly(x):
    """
            x has to be an instance of a class inheriting from Base.
            """
    cant(x, {})
    try:
        del x.__dict__
    except (AttributeError, TypeError):
        pass
    else:
        self.fail("shouldn't allow del %r.__dict__" % x)
    dict_descr = Base.__dict__['__dict__']
    try:
        dict_descr.__set__(x, {})
    except (AttributeError, TypeError):
        pass
    else:
        self.fail("dict_descr allowed access to %r's dict" % x)

class Meta1(type, Base):
    pass

class Meta2(Base, type):
    pass

class D(object, metaclass=Meta1):
    pass

class E(object, metaclass=Meta2):
    pass
for cls in (C, D, E):
    verify_dict_readonly(cls)
    class_dict = cls.__dict__
    try:
        class_dict['spam'] = 'eggs'
    except TypeError:
        pass
    else:

        raise AssertionError("%r's __dict__ can be modified" % cls)

class Module1(types.ModuleType, Base):
    pass

class Module2(Base, types.ModuleType):
    pass
for ModuleType in (Module1, Module2):
    mod = ModuleType('spam')
    verify_dict_readonly(mod)
    mod.__dict__['spam'] = 'eggs'

def can_delete_dict(e):
    try:
        del e.__dict__
    except (TypeError, AttributeError):
        return False
    else:
        return True

class Exception1(Exception, Base):
    pass

class Exception2(Base, Exception):
    pass
for ExceptionType in (Exception, Exception1, Exception2):
    e = ExceptionType()
    e.__dict__ = {'a': 1}

    assert e.a == 1

    assert can_delete_dict(e) == can_delete_dict(ValueError())
print("ClassPropertiesAndMethods::test_set_dict: ok")
