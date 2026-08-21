# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "test_single_dispatch__test_method_wrapping_attributes"
# subject = "cpython.test_functools.TestSingleDispatch.test_method_wrapping_attributes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_functools.py::TestSingleDispatch::test_method_wrapping_attributes
"""Auto-ported test: TestSingleDispatch::test_method_wrapping_attributes (CPython 3.12 oracle)."""


import abc
import builtins
import collections
import collections.abc
import copy
from itertools import permutations
import pickle
from random import choice
import sys
from test import support
import threading
import time
import typing
import unittest
import unittest.mock
import weakref
import gc
from weakref import proxy
import contextlib
from inspect import Signature
from test.support import import_helper
from test.support import threading_helper
import functools


py_functools = import_helper.import_fresh_module('functools', blocked=['_functools'])

c_functools = import_helper.import_fresh_module('functools', fresh=['_functools'])

decimal = import_helper.import_fresh_module('decimal', fresh=['_decimal'])

_partial_types = [py_functools.partial]

if c_functools:
    _partial_types.append(c_functools.partial)

@contextlib.contextmanager
def replaced_module(name, replacement):
    original_module = sys.modules[name]
    sys.modules[name] = replacement
    try:
        yield
    finally:
        sys.modules[name] = original_module

def capture(*args, **kw):
    """capture all positional and keyword arguments"""
    return (args, kw)

def signature(part):
    """ return the signature of a partial object """
    return (part.func, part.args, part.keywords, part.__dict__)

class MyTuple(tuple):
    pass

class BadTuple(tuple):

    def __add__(self, other):
        return list(self) + list(other)

class MyDict(dict):
    pass

if c_functools:

    class CPartialSubclass(c_functools.partial):
        pass

class PyPartialSubclass(py_functools.partial):
    pass

@functools.total_ordering
class Orderable_LT:

    def __init__(self, value):
        self.value = value

    def __lt__(self, other):
        return self.value < other.value

    def __eq__(self, other):
        return self.value == other.value

@py_functools.lru_cache()
def py_cached_func(x, y):
    return 3 * x + y

if c_functools:

    @c_functools.lru_cache()
    def c_cached_func(x, y):
        return 3 * x + y

class CachedCostItem:
    _cost = 1

    def __init__(self):
        self.lock = py_functools.RLock()

    @py_functools.cached_property
    def cost(self):
        """The cost of the item."""
        with self.lock:
            self._cost += 1
        return self._cost

class OptionallyCachedCostItem:
    _cost = 1

    def get_cost(self):
        """The cost of the item."""
        self._cost += 1
        return self._cost
    cached_cost = py_functools.cached_property(get_cost)

class CachedCostItemWithSlots:
    __slots__ = '_cost'

    def __init__(self):
        self._cost = 1

    @py_functools.cached_property
    def cost(self):
        raise RuntimeError('never called, slots not supported')


# --- test body ---
class A:

    @functools.singledispatchmethod
    def func(self, arg: int) -> str:
        """My function docstring"""
        return str(arg)

    @functools.singledispatchmethod
    @classmethod
    def cls_func(cls, arg: int) -> str:
        """My function docstring"""
        return str(arg)

    @functools.singledispatchmethod
    @staticmethod
    def static_func(arg: int) -> str:
        """My function docstring"""
        return str(arg)
prefix = A.__qualname__ + '.'
for meth in (A.func, A().func, A.cls_func, A().cls_func, A.static_func, A().static_func):

    assert meth.__qualname__ == prefix + meth.__name__

    assert meth.__doc__ == ('My function docstring' if support.HAVE_DOCSTRINGS else None)

    assert meth.__annotations__['arg'] == int

assert A.func.__name__ == 'func'

assert A().func.__name__ == 'func'

assert A.cls_func.__name__ == 'cls_func'

assert A().cls_func.__name__ == 'cls_func'

assert A.static_func.__name__ == 'static_func'

assert A().static_func.__name__ == 'static_func'
print("TestSingleDispatch::test_method_wrapping_attributes: ok")
