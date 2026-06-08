# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "test_named_tuple__test_factory"
# subject = "cpython.test_collections.TestNamedTuple.test_factory"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_collections.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_collections.py::TestNamedTuple::test_factory
"""Auto-ported test: TestNamedTuple::test_factory (CPython 3.12 oracle)."""


import collections
import copy
import doctest
import inspect
import operator
import pickle
from random import choice, randrange
from itertools import product, chain, combinations
import string
import sys
from test import support
import types
import unittest
from collections import namedtuple, Counter, OrderedDict, _count_elements
from collections import UserDict, UserString, UserList
from collections import ChainMap
from collections import deque
from collections.abc import Awaitable, Coroutine
from collections.abc import AsyncIterator, AsyncIterable, AsyncGenerator
from collections.abc import Hashable, Iterable, Iterator, Generator, Reversible
from collections.abc import Sized, Container, Callable, Collection
from collections.abc import Set, MutableSet
from collections.abc import Mapping, MutableMapping, KeysView, ItemsView, ValuesView
from collections.abc import Sequence, MutableSequence
from collections.abc import ByteString, Buffer


'Unit tests for collections.py.'

TestNT = namedtuple('TestNT', 'x y z')

class ABCTestCase(unittest.TestCase):

    def validate_abstract_methods(self, abc, *names):
        methodstubs = dict.fromkeys(names, lambda s, *args: 0)
        C = type('C', (abc,), methodstubs)
        C()
        for name in names:
            stubs = methodstubs.copy()
            del stubs[name]
            C = type('C', (abc,), stubs)
            self.assertRaises(TypeError, C, name)

    def validate_isinstance(self, abc, name):
        stub = lambda s, *args: 0
        C = type('C', (object,), {'__hash__': None})
        setattr(C, name, stub)
        self.assertIsInstance(C(), abc)
        self.assertTrue(issubclass(C, abc))
        C = type('C', (object,), {'__hash__': None})
        self.assertNotIsInstance(C(), abc)
        self.assertFalse(issubclass(C, abc))

    def validate_comparison(self, instance):
        ops = ['lt', 'gt', 'le', 'ge', 'ne', 'or', 'and', 'xor', 'sub']
        operators = {}
        for op in ops:
            name = '__' + op + '__'
            operators[name] = getattr(operator, name)

        class Other:

            def __init__(self):
                self.right_side = False

            def __eq__(self, other):
                self.right_side = True
                return True
            __lt__ = __eq__
            __gt__ = __eq__
            __le__ = __eq__
            __ge__ = __eq__
            __ne__ = __eq__
            __ror__ = __eq__
            __rand__ = __eq__
            __rxor__ = __eq__
            __rsub__ = __eq__
        for name, op in operators.items():
            if not hasattr(instance, name):
                continue
            other = Other()
            op(instance, other)
            self.assertTrue(other.right_side, 'Right side not called for %s.%s' % (type(instance), name))

def _test_gen():
    yield

class WithSet(MutableSet):

    def __init__(self, it=()):
        self.data = set(it)

    def __len__(self):
        return len(self.data)

    def __iter__(self):
        return iter(self.data)

    def __contains__(self, item):
        return item in self.data

    def add(self, item):
        self.data.add(item)

    def discard(self, item):
        self.data.discard(item)

class CounterSubclassWithSetItem(Counter):

    def __init__(self, *args, **kwds):
        self.called = False
        Counter.__init__(self, *args, **kwds)

    def __setitem__(self, key, value):
        self.called = True
        Counter.__setitem__(self, key, value)

class CounterSubclassWithGet(Counter):

    def __init__(self, *args, **kwds):
        self.called = False
        Counter.__init__(self, *args, **kwds)

    def get(self, key, default):
        self.called = True
        return Counter.get(self, key, default)

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(collections))
    return tests


# --- test body ---
Point = namedtuple('Point', 'x y')

assert Point.__name__ == 'Point'

assert Point.__slots__ == ()

assert Point.__module__ == __name__

assert Point.__getitem__ == tuple.__getitem__

assert Point._fields == ('x', 'y')

try:
    namedtuple('abc%', 'efg ghi')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    namedtuple('class', 'efg ghi')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    namedtuple('9abc', 'efg ghi')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    namedtuple('abc', 'efg g%hi')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    namedtuple('abc', 'abc class')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    namedtuple('abc', '8efg 9ghi')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    namedtuple('abc', '_efg ghi')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    namedtuple('abc', 'efg efg ghi')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
namedtuple('Point0', 'x1 y2')
namedtuple('_', 'a b c')
nt = namedtuple('nt', 'the quick brown fox')

assert "u'" not in repr(nt._fields)
nt = namedtuple('nt', ('the', 'quick'))

assert "u'" not in repr(nt._fields)

try:
    Point._make([11])
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    Point._make([11, 22, 33])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestNamedTuple::test_factory: ok")
