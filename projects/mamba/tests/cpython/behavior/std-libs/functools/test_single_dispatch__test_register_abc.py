# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "test_single_dispatch__test_register_abc"
# subject = "cpython.test_functools.TestSingleDispatch.test_register_abc"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_functools.py::TestSingleDispatch::test_register_abc
"""Auto-ported test: TestSingleDispatch::test_register_abc (CPython 3.12 oracle)."""


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
c = collections.abc
d = {'a': 'b'}
l = [1, 2, 3]
s = {object(), None}
f = frozenset(s)
t = (1, 2, 3)

@functools.singledispatch
def g(obj):
    return 'base'

assert g(d) == 'base'

assert g(l) == 'base'

assert g(s) == 'base'

assert g(f) == 'base'

assert g(t) == 'base'
g.register(c.Sized, lambda obj: 'sized')

assert g(d) == 'sized'

assert g(l) == 'sized'

assert g(s) == 'sized'

assert g(f) == 'sized'

assert g(t) == 'sized'
g.register(c.MutableMapping, lambda obj: 'mutablemapping')

assert g(d) == 'mutablemapping'

assert g(l) == 'sized'

assert g(s) == 'sized'

assert g(f) == 'sized'

assert g(t) == 'sized'
g.register(collections.ChainMap, lambda obj: 'chainmap')

assert g(d) == 'mutablemapping'

assert g(l) == 'sized'

assert g(s) == 'sized'

assert g(f) == 'sized'

assert g(t) == 'sized'
g.register(c.MutableSequence, lambda obj: 'mutablesequence')

assert g(d) == 'mutablemapping'

assert g(l) == 'mutablesequence'

assert g(s) == 'sized'

assert g(f) == 'sized'

assert g(t) == 'sized'
g.register(c.MutableSet, lambda obj: 'mutableset')

assert g(d) == 'mutablemapping'

assert g(l) == 'mutablesequence'

assert g(s) == 'mutableset'

assert g(f) == 'sized'

assert g(t) == 'sized'
g.register(c.Mapping, lambda obj: 'mapping')

assert g(d) == 'mutablemapping'

assert g(l) == 'mutablesequence'

assert g(s) == 'mutableset'

assert g(f) == 'sized'

assert g(t) == 'sized'
g.register(c.Sequence, lambda obj: 'sequence')

assert g(d) == 'mutablemapping'

assert g(l) == 'mutablesequence'

assert g(s) == 'mutableset'

assert g(f) == 'sized'

assert g(t) == 'sequence'
g.register(c.Set, lambda obj: 'set')

assert g(d) == 'mutablemapping'

assert g(l) == 'mutablesequence'

assert g(s) == 'mutableset'

assert g(f) == 'set'

assert g(t) == 'sequence'
g.register(dict, lambda obj: 'dict')

assert g(d) == 'dict'

assert g(l) == 'mutablesequence'

assert g(s) == 'mutableset'

assert g(f) == 'set'

assert g(t) == 'sequence'
g.register(list, lambda obj: 'list')

assert g(d) == 'dict'

assert g(l) == 'list'

assert g(s) == 'mutableset'

assert g(f) == 'set'

assert g(t) == 'sequence'
g.register(set, lambda obj: 'concrete-set')

assert g(d) == 'dict'

assert g(l) == 'list'

assert g(s) == 'concrete-set'

assert g(f) == 'set'

assert g(t) == 'sequence'
g.register(frozenset, lambda obj: 'frozen-set')

assert g(d) == 'dict'

assert g(l) == 'list'

assert g(s) == 'concrete-set'

assert g(f) == 'frozen-set'

assert g(t) == 'sequence'
g.register(tuple, lambda obj: 'tuple')

assert g(d) == 'dict'

assert g(l) == 'list'

assert g(s) == 'concrete-set'

assert g(f) == 'frozen-set'

assert g(t) == 'tuple'
print("TestSingleDispatch::test_register_abc: ok")
