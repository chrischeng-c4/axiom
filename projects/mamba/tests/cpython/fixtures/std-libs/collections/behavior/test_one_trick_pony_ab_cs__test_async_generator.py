# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "test_one_trick_pony_ab_cs__test_async_generator"
# subject = "cpython.test_collections.TestOneTrickPonyABCs.test_AsyncGenerator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_collections.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_collections.py::TestOneTrickPonyABCs::test_AsyncGenerator
"""Auto-ported test: TestOneTrickPonyABCs::test_AsyncGenerator (CPython 3.12 oracle)."""


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

class NonAGen1:

    def __aiter__(self):
        return self

    def __anext__(self):
        return None

    def aclose(self):
        pass

    def athrow(self, typ, val=None, tb=None):
        pass

class NonAGen2:

    def __aiter__(self):
        return self

    def __anext__(self):
        return None

    def aclose(self):
        pass

    def asend(self, value):
        return value

class NonAGen3:

    def aclose(self):
        pass

    def asend(self, value):
        return value

    def athrow(self, typ, val=None, tb=None):
        pass
non_samples = [None, 42, 3.14, 1j, b'', '', (), [], {}, set(), iter(()), iter([]), NonAGen1(), NonAGen2(), NonAGen3()]
for x in non_samples:

    assert not isinstance(x, AsyncGenerator)

    assert not issubclass(type(x), AsyncGenerator)

class Gen:

    def __aiter__(self):
        return self

    async def __anext__(self):
        return None

    async def aclose(self):
        pass

    async def asend(self, value):
        return value

    async def athrow(self, typ, val=None, tb=None):
        pass

class MinimalAGen(AsyncGenerator):

    async def asend(self, value):
        return value

    async def athrow(self, typ, val=None, tb=None):
        await super().athrow(typ, val, tb)

async def gen():
    yield 1
samples = [gen(), Gen(), MinimalAGen()]
for x in samples:

    assert isinstance(x, AsyncIterator)

    assert isinstance(x, AsyncGenerator)

    assert issubclass(type(x), AsyncGenerator)
validate_abstract_methods(AsyncGenerator, 'asend', 'athrow')

def run_async(coro):
    result = None
    while True:
        try:
            coro.send(None)
        except StopIteration as ex:
            result = ex.args[0] if ex.args else None
            break
    return result
mgen = MinimalAGen()

assert mgen is mgen.__aiter__()

assert run_async(mgen.asend(None)) is run_async(mgen.__anext__())

assert 2 == run_async(mgen.asend(2))

assert run_async(mgen.aclose()) is None
try:
    run_async(mgen.athrow(ValueError))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

class FailOnClose(AsyncGenerator):

    async def asend(self, value):
        return value

    async def athrow(self, *args):
        raise ValueError
try:
    run_async(FailOnClose().aclose())
    raise AssertionError('expected ValueError')
except ValueError:
    pass

class IgnoreGeneratorExit(AsyncGenerator):

    async def asend(self, value):
        return value

    async def athrow(self, *args):
        pass
try:
    run_async(IgnoreGeneratorExit().aclose())
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("TestOneTrickPonyABCs::test_AsyncGenerator: ok")
