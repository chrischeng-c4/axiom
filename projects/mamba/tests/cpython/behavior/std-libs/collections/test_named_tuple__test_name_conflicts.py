# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "test_named_tuple__test_name_conflicts"
# subject = "cpython.test_collections.TestNamedTuple.test_name_conflicts"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_collections.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_collections.py::TestNamedTuple::test_name_conflicts
"""Auto-ported test: TestNamedTuple::test_name_conflicts (CPython 3.12 oracle)."""


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
T = namedtuple('T', 'itemgetter property self cls tuple')
t = T(1, 2, 3, 4, 5)

assert t == (1, 2, 3, 4, 5)
newt = t._replace(itemgetter=10, property=20, self=30, cls=40, tuple=50)

assert newt == (10, 20, 30, 40, 50)
words = {'Alias', 'At', 'AttributeError', 'Build', 'Bypass', 'Create', 'Encountered', 'Expected', 'Field', 'For', 'Got', 'Helper', 'IronPython', 'Jython', 'KeyError', 'Make', 'Modify', 'Note', 'OrderedDict', 'Point', 'Return', 'Returns', 'Type', 'TypeError', 'Used', 'Validate', 'ValueError', 'Variables', 'a', 'accessible', 'add', 'added', 'all', 'also', 'an', 'arg_list', 'args', 'arguments', 'automatically', 'be', 'build', 'builtins', 'but', 'by', 'cannot', 'class_namespace', 'classmethod', 'cls', 'collections', 'convert', 'copy', 'created', 'creation', 'd', 'debugging', 'defined', 'dict', 'dictionary', 'doc', 'docstring', 'docstrings', 'duplicate', 'effect', 'either', 'enumerate', 'environments', 'error', 'example', 'exec', 'f', 'f_globals', 'field', 'field_names', 'fields', 'formatted', 'frame', 'function', 'functions', 'generate', 'get', 'getter', 'got', 'greater', 'has', 'help', 'identifiers', 'index', 'indexable', 'instance', 'instantiate', 'interning', 'introspection', 'isidentifier', 'isinstance', 'itemgetter', 'iterable', 'join', 'keyword', 'keywords', 'kwds', 'len', 'like', 'list', 'map', 'maps', 'message', 'metadata', 'method', 'methods', 'module', 'module_name', 'must', 'name', 'named', 'namedtuple', 'namedtuple_', 'names', 'namespace', 'needs', 'new', 'nicely', 'num_fields', 'number', 'object', 'of', 'operator', 'option', 'p', 'particular', 'pickle', 'pickling', 'plain', 'pop', 'positional', 'property', 'r', 'regular', 'rename', 'replace', 'replacing', 'repr', 'repr_fmt', 'representation', 'result', 'reuse_itemgetter', 's', 'seen', 'self', 'sequence', 'set', 'side', 'specified', 'split', 'start', 'startswith', 'step', 'str', 'string', 'strings', 'subclass', 'sys', 'targets', 'than', 'the', 'their', 'this', 'to', 'tuple', 'tuple_new', 'type', 'typename', 'underscore', 'unexpected', 'unpack', 'up', 'use', 'used', 'user', 'valid', 'values', 'variable', 'verbose', 'where', 'which', 'work', 'x', 'y', 'z', 'zip'}
T = namedtuple('T', words)
values = tuple(range(len(words)))
t = T(*values)

assert t == values
t = T(**dict(zip(T._fields, values)))

assert t == values
t = T._make(values)

assert t == values
repr(t)

assert t._asdict() == dict(zip(T._fields, values))
t = T._make(values)
newvalues = tuple((v * 10 for v in values))
newt = t._replace(**dict(zip(T._fields, newvalues)))

assert newt == newvalues

assert T._fields == tuple(words)

assert t.__getnewargs__() == values
print("TestNamedTuple::test_name_conflicts: ok")
