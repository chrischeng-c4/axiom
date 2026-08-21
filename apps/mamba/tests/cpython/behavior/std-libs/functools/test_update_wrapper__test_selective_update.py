# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "test_update_wrapper__test_selective_update"
# subject = "cpython.test_functools.TestUpdateWrapper.test_selective_update"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_functools.py::TestUpdateWrapper::test_selective_update
"""Auto-ported test: TestUpdateWrapper::test_selective_update (CPython 3.12 oracle)."""


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
def _default_update():

    def f[T](a: 'This is a new annotation'):
        """This is a test"""
        pass
    f.attr = 'This is also a test'
    f.__wrapped__ = 'This is a bald faced lie'

    def wrapper(b: 'This is the prior annotation'):
        pass
    functools.update_wrapper(wrapper, f)
    return (wrapper, f)

def check_wrapper(wrapper, wrapped, assigned=functools.WRAPPER_ASSIGNMENTS, updated=functools.WRAPPER_UPDATES):
    for name in assigned:

        assert getattr(wrapper, name) is getattr(wrapped, name)
    for name in updated:
        wrapper_attr = getattr(wrapper, name)
        wrapped_attr = getattr(wrapped, name)
        for key in wrapped_attr:
            if name == '__dict__' and key == '__wrapped__':
                continue

            assert wrapped_attr[key] is wrapper_attr[key]

    assert wrapper.__wrapped__ is wrapped

def f():
    pass
f.attr = 'This is a different test'
f.dict_attr = dict(a=1, b=2, c=3)

def wrapper():
    pass
wrapper.dict_attr = {}
assign = ('attr',)
update = ('dict_attr',)
functools.update_wrapper(wrapper, f, assign, update)
check_wrapper(wrapper, f, assign, update)

assert wrapper.__name__ == 'wrapper'

assert wrapper.__qualname__ != f.__qualname__

assert wrapper.__doc__ == None

assert wrapper.attr == 'This is a different test'

assert wrapper.dict_attr == f.dict_attr
print("TestUpdateWrapper::test_selective_update: ok")
