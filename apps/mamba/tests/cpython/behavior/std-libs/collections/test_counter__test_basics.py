# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "test_counter__test_basics"
# subject = "cpython.test_collections.TestCounter.test_basics"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_collections.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_collections.py::TestCounter::test_basics
"""Auto-ported test: TestCounter::test_basics (CPython 3.12 oracle)."""


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
c = Counter('abcaba')

assert c == Counter({'a': 3, 'b': 2, 'c': 1})

assert c == Counter(a=3, b=2, c=1)

assert isinstance(c, dict)

assert isinstance(c, Mapping)

assert issubclass(Counter, dict)

assert issubclass(Counter, Mapping)

assert len(c) == 3

assert sum(c.values()) == 6

assert list(c.values()) == [3, 2, 1]

assert list(c.keys()) == ['a', 'b', 'c']

assert list(c) == ['a', 'b', 'c']

assert list(c.items()) == [('a', 3), ('b', 2), ('c', 1)]

assert c['b'] == 2

assert c['z'] == 0

assert c.__contains__('c') == True

assert c.__contains__('z') == False

assert c.get('b', 10) == 2

assert c.get('z', 10) == 10

assert c == dict(a=3, b=2, c=1)

assert repr(c) == "Counter({'a': 3, 'b': 2, 'c': 1})"

assert c.most_common() == [('a', 3), ('b', 2), ('c', 1)]
for i in range(5):

    assert c.most_common(i) == [('a', 3), ('b', 2), ('c', 1)][:i]

assert ''.join(c.elements()) == 'aaabbc'
c['a'] += 1
c['b'] -= 2
del c['c']
del c['c']
c['d'] -= 2
c['e'] = -5
c['f'] += 4

assert c == dict(a=4, b=0, d=-2, e=-5, f=4)

assert ''.join(c.elements()) == 'aaaaffff'

assert c.pop('f') == 4

assert 'f' not in c
for i in range(3):
    elem, cnt = c.popitem()

    assert elem not in c
c.clear()

assert c == {}

assert repr(c) == 'Counter()'

try:
    Counter.fromkeys('abc')
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    hash(c)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
c.update(dict(a=5, b=3))
c.update(c=1)
c.update(Counter('a' * 50 + 'b' * 30))
c.update()
c.__init__('a' * 500 + 'b' * 300)
c.__init__('cdc')
c.__init__()

assert c == dict(a=555, b=333, c=3, d=1)

assert c.setdefault('d', 5) == 1

assert c['d'] == 1

assert c.setdefault('e', 5) == 5

assert c['e'] == 5
print("TestCounter::test_basics: ok")
