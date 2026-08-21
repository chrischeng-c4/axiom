# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "test_single_dispatch__test_cache_invalidation"
# subject = "cpython.test_functools.TestSingleDispatch.test_cache_invalidation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_functools.py::TestSingleDispatch::test_cache_invalidation
"""Auto-ported test: TestSingleDispatch::test_cache_invalidation (CPython 3.12 oracle)."""


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
from collections import UserDict
import weakref

class TracingDict(UserDict):

    def __init__(self, *args, **kwargs):
        super(TracingDict, self).__init__(*args, **kwargs)
        self.set_ops = []
        self.get_ops = []

    def __getitem__(self, key):
        result = self.data[key]
        self.get_ops.append(key)
        return result

    def __setitem__(self, key, value):
        self.set_ops.append(key)
        self.data[key] = value

    def clear(self):
        self.data.clear()
td = TracingDict()
with support.swap_attr(weakref, 'WeakKeyDictionary', lambda: td):
    c = collections.abc

    @functools.singledispatch
    def g(arg):
        return 'base'
    d = {}
    l = []

    assert len(td) == 0

    assert g(d) == 'base'

    assert len(td) == 1

    assert td.get_ops == []

    assert td.set_ops == [dict]

    assert td.data[dict] == g.registry[object]

    assert g(l) == 'base'

    assert len(td) == 2

    assert td.get_ops == []

    assert td.set_ops == [dict, list]

    assert td.data[dict] == g.registry[object]

    assert td.data[list] == g.registry[object]

    assert td.data[dict] == td.data[list]

    assert g(l) == 'base'

    assert g(d) == 'base'

    assert td.get_ops == [list, dict]

    assert td.set_ops == [dict, list]
    g.register(list, lambda arg: 'list')

    assert td.get_ops == [list, dict]

    assert len(td) == 0

    assert g(d) == 'base'

    assert len(td) == 1

    assert td.get_ops == [list, dict]

    assert td.set_ops == [dict, list, dict]

    assert td.data[dict] == functools._find_impl(dict, g.registry)

    assert g(l) == 'list'

    assert len(td) == 2

    assert td.get_ops == [list, dict]

    assert td.set_ops == [dict, list, dict, list]

    assert td.data[list] == functools._find_impl(list, g.registry)

    class X:
        pass
    c.MutableMapping.register(X)

    assert g(d) == 'base'

    assert g(l) == 'list'

    assert td.get_ops == [list, dict, dict, list]

    assert td.set_ops == [dict, list, dict, list]
    g.register(c.Sized, lambda arg: 'sized')

    assert len(td) == 0

    assert g(d) == 'sized'

    assert len(td) == 1

    assert td.get_ops == [list, dict, dict, list]

    assert td.set_ops == [dict, list, dict, list, dict]

    assert g(l) == 'list'

    assert len(td) == 2

    assert td.get_ops == [list, dict, dict, list]

    assert td.set_ops == [dict, list, dict, list, dict, list]

    assert g(l) == 'list'

    assert g(d) == 'sized'

    assert td.get_ops == [list, dict, dict, list, list, dict]

    assert td.set_ops == [dict, list, dict, list, dict, list]
    g.dispatch(list)
    g.dispatch(dict)

    assert td.get_ops == [list, dict, dict, list, list, dict, list, dict]

    assert td.set_ops == [dict, list, dict, list, dict, list]
    c.MutableSet.register(X)

    assert len(td) == 2

    assert g(l) == 'list'

    assert len(td) == 1
    g.register(c.MutableMapping, lambda arg: 'mutablemapping')

    assert len(td) == 0

    assert g(d) == 'mutablemapping'

    assert len(td) == 1

    assert g(l) == 'list'

    assert len(td) == 2
    g.register(dict, lambda arg: 'dict')

    assert g(d) == 'dict'

    assert g(l) == 'list'
    g._clear_cache()

    assert len(td) == 0
print("TestSingleDispatch::test_cache_invalidation: ok")
