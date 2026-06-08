# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "test_single_dispatch__test_double_wrapped_methods"
# subject = "cpython.test_functools.TestSingleDispatch.test_double_wrapped_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_functools.py::TestSingleDispatch::test_double_wrapped_methods
"""Auto-ported test: TestSingleDispatch::test_double_wrapped_methods (CPython 3.12 oracle)."""


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
def classmethod_friendly_decorator(func):
    wrapped = func.__func__

    @classmethod
    @functools.wraps(wrapped)
    def wrapper(*args, **kwargs):
        return wrapped(*args, **kwargs)
    return wrapper

class WithoutSingleDispatch:

    @classmethod
    @contextlib.contextmanager
    def cls_context_manager(cls, arg: int) -> str:
        try:
            yield str(arg)
        finally:
            return 'Done'

    @classmethod_friendly_decorator
    @classmethod
    def decorated_classmethod(cls, arg: int) -> str:
        return str(arg)

class WithSingleDispatch:

    @functools.singledispatchmethod
    @classmethod
    @contextlib.contextmanager
    def cls_context_manager(cls, arg: int) -> str:
        """My function docstring"""
        try:
            yield str(arg)
        finally:
            return 'Done'

    @functools.singledispatchmethod
    @classmethod_friendly_decorator
    @classmethod
    def decorated_classmethod(cls, arg: int) -> str:
        """My function docstring"""
        return str(arg)
with WithoutSingleDispatch.cls_context_manager(5) as foo:
    without_single_dispatch_foo = foo
with WithSingleDispatch.cls_context_manager(5) as foo:
    single_dispatch_foo = foo

assert without_single_dispatch_foo == single_dispatch_foo

assert single_dispatch_foo == '5'

assert WithoutSingleDispatch.decorated_classmethod(5) == WithSingleDispatch.decorated_classmethod(5)

assert WithSingleDispatch.decorated_classmethod(5) == '5'
for method_name in ('cls_context_manager', 'decorated_classmethod'):

    assert getattr(WithSingleDispatch, method_name).__name__ == getattr(WithoutSingleDispatch, method_name).__name__

    assert getattr(WithSingleDispatch(), method_name).__name__ == getattr(WithoutSingleDispatch(), method_name).__name__
for meth in (WithSingleDispatch.cls_context_manager, WithSingleDispatch().cls_context_manager, WithSingleDispatch.decorated_classmethod, WithSingleDispatch().decorated_classmethod):

    assert meth.__doc__ == ('My function docstring' if support.HAVE_DOCSTRINGS else None)

    assert meth.__annotations__['arg'] == int

assert WithSingleDispatch.cls_context_manager.__name__ == 'cls_context_manager'

assert WithSingleDispatch().cls_context_manager.__name__ == 'cls_context_manager'

assert WithSingleDispatch.decorated_classmethod.__name__ == 'decorated_classmethod'

assert WithSingleDispatch().decorated_classmethod.__name__ == 'decorated_classmethod'
print("TestSingleDispatch::test_double_wrapped_methods: ok")
