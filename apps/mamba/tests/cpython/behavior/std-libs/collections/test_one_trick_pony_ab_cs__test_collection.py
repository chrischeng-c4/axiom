# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "test_one_trick_pony_ab_cs__test_collection"
# subject = "cpython.test_collections.TestOneTrickPonyABCs.test_Collection"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_collections.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_collections.py::TestOneTrickPonyABCs::test_Collection
"""Auto-ported test: TestOneTrickPonyABCs::test_Collection (CPython 3.12 oracle)."""


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
def validate_abstract_methods(abc, *names):
    methodstubs = dict.fromkeys(names, lambda s, *args: 0)
    C = type('C', (abc,), methodstubs)
    C()
    for name in names:
        stubs = methodstubs.copy()
        del stubs[name]
        C = type('C', (abc,), stubs)

        try:
            C(name)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

def validate_comparison(instance):
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

        assert other.right_side

def validate_isinstance(abc, name):
    stub = lambda s, *args: 0
    C = type('C', (object,), {'__hash__': None})
    setattr(C, name, stub)

    assert isinstance(C(), abc)

    assert issubclass(C, abc)
    C = type('C', (object,), {'__hash__': None})

    assert not isinstance(C(), abc)

    assert not issubclass(C, abc)
non_collections = [None, 42, 3.14, 1j, lambda x: 2 * x]
for x in non_collections:

    assert not isinstance(x, Collection)

    assert not issubclass(type(x), Collection)
non_col_iterables = [_test_gen(), iter(b''), iter(bytearray()), (x for x in [])]
for x in non_col_iterables:

    assert not isinstance(x, Collection)

    assert not issubclass(type(x), Collection)
samples = [set(), frozenset(), dict(), bytes(), str(), tuple(), list(), dict().keys(), dict().items(), dict().values()]
for x in samples:

    assert isinstance(x, Collection)

    assert issubclass(type(x), Collection)

assert issubclass(Sequence, Collection)

assert issubclass(Mapping, Collection)

assert issubclass(MutableMapping, Collection)

assert issubclass(Set, Collection)

assert issubclass(MutableSet, Collection)

assert issubclass(Sequence, Collection)

class Col(Collection):

    def __iter__(self):
        return iter(list())

    def __len__(self):
        return 0

    def __contains__(self, item):
        return False

class DerCol(Col):
    pass

assert list(iter(Col())) == []

assert not issubclass(list, Col)

assert not issubclass(set, Col)

assert not issubclass(float, Col)

assert list(iter(DerCol())) == []

assert not issubclass(list, DerCol)

assert not issubclass(set, DerCol)

assert not issubclass(float, DerCol)
validate_abstract_methods(Collection, '__len__', '__iter__', '__contains__')

class ColNoIter:

    def __len__(self):
        return 0

    def __contains__(self, item):
        return False

class ColNoSize:

    def __iter__(self):
        return iter([])

    def __contains__(self, item):
        return False

class ColNoCont:

    def __iter__(self):
        return iter([])

    def __len__(self):
        return 0

assert not issubclass(ColNoIter, Collection)

assert not isinstance(ColNoIter(), Collection)

assert not issubclass(ColNoSize, Collection)

assert not isinstance(ColNoSize(), Collection)

assert not issubclass(ColNoCont, Collection)

assert not isinstance(ColNoCont(), Collection)

class SizeBlock:

    def __iter__(self):
        return iter([])

    def __contains__(self):
        return False
    __len__ = None

class IterBlock:

    def __len__(self):
        return 0

    def __contains__(self):
        return True
    __iter__ = None

assert not issubclass(SizeBlock, Collection)

assert not isinstance(SizeBlock(), Collection)

assert not issubclass(IterBlock, Collection)

assert not isinstance(IterBlock(), Collection)

class ColImpl:

    def __iter__(self):
        return iter(list())

    def __len__(self):
        return 0

    def __contains__(self, item):
        return False

class NonCol(ColImpl):
    __contains__ = None

assert not issubclass(NonCol, Collection)

assert not isinstance(NonCol(), Collection)
print("TestOneTrickPonyABCs::test_Collection: ok")
