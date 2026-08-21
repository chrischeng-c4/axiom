# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "mapping_test_case__test_weak_keyed_union_operators"
# subject = "cpython.test_weakref.MappingTestCase.test_weak_keyed_union_operators"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_weakref.py::MappingTestCase::test_weak_keyed_union_operators
"""Auto-ported test: MappingTestCase::test_weak_keyed_union_operators (CPython 3.12 oracle)."""


import gc
import sys
import doctest
import unittest
import collections
import weakref
import operator
import contextlib
import copy
import threading
import time
import random
from test import support
from test.support import script_helper, ALWAYS_EQ
from test.support import gc_collect
from test.support import threading_helper
from test import mapping_tests


ref_from_del = None

_global_var = 'foobar'

class C:

    def method(self):
        pass

class Callable:
    bar = None

    def __call__(self, x):
        self.bar = x

def create_function():

    def f():
        pass
    return f

def create_bound_method():
    return C().method

class Object:

    def __init__(self, arg):
        self.arg = arg

    def __repr__(self):
        return '<Object %r>' % self.arg

    def __eq__(self, other):
        if isinstance(other, Object):
            return self.arg == other.arg
        return NotImplemented

    def __lt__(self, other):
        if isinstance(other, Object):
            return self.arg < other.arg
        return NotImplemented

    def __hash__(self):
        return hash(self.arg)

    def some_method(self):
        return 4

    def other_method(self):
        return 5

class RefCycle:

    def __init__(self):
        self.cycle = self

@contextlib.contextmanager
def collect_in_thread(period=0.0001):
    """
    Ensure GC collections happen in a different thread, at a high frequency.
    """
    please_stop = False

    def collect():
        while not please_stop:
            time.sleep(period)
            gc.collect()
    with support.disable_gc():
        t = threading.Thread(target=collect)
        t.start()
        try:
            yield
        finally:
            please_stop = True
            t.join()

class WeakValueDictionaryTestCase(mapping_tests.BasicTestMappingProtocol):
    """Check that WeakValueDictionary conforms to the mapping protocol"""
    __ref = {'key1': Object(1), 'key2': Object(2), 'key3': Object(3)}
    type2test = weakref.WeakValueDictionary

    def _reference(self):
        return self.__ref.copy()

class WeakKeyDictionaryTestCase(mapping_tests.BasicTestMappingProtocol):
    """Check that WeakKeyDictionary conforms to the mapping protocol"""
    __ref = {Object('key1'): 1, Object('key2'): 2, Object('key3'): 3}
    type2test = weakref.WeakKeyDictionary

    def _reference(self):
        return self.__ref.copy()

libreftest = ' Doctest for examples in the library reference: weakref.rst\n\n>>> from test.support import gc_collect\n>>> import weakref\n>>> class Dict(dict):\n...     pass\n...\n>>> obj = Dict(red=1, green=2, blue=3)   # this object is weak referencable\n>>> r = weakref.ref(obj)\n>>> print(r() is obj)\nTrue\n\n>>> import weakref\n>>> class Object:\n...     pass\n...\n>>> o = Object()\n>>> r = weakref.ref(o)\n>>> o2 = r()\n>>> o is o2\nTrue\n>>> del o, o2\n>>> gc_collect()  # For PyPy or other GCs.\n>>> print(r())\nNone\n\n>>> import weakref\n>>> class ExtendedRef(weakref.ref):\n...     def __init__(self, ob, callback=None, **annotations):\n...         super().__init__(ob, callback)\n...         self.__counter = 0\n...         for k, v in annotations.items():\n...             setattr(self, k, v)\n...     def __call__(self):\n...         \'\'\'Return a pair containing the referent and the number of\n...         times the reference has been called.\n...         \'\'\'\n...         ob = super().__call__()\n...         if ob is not None:\n...             self.__counter += 1\n...             ob = (ob, self.__counter)\n...         return ob\n...\n>>> class A:   # not in docs from here, just testing the ExtendedRef\n...     pass\n...\n>>> a = A()\n>>> r = ExtendedRef(a, foo=1, bar="baz")\n>>> r.foo\n1\n>>> r.bar\n\'baz\'\n>>> r()[1]\n1\n>>> r()[1]\n2\n>>> r()[0] is a\nTrue\n\n\n>>> import weakref\n>>> _id2obj_dict = weakref.WeakValueDictionary()\n>>> def remember(obj):\n...     oid = id(obj)\n...     _id2obj_dict[oid] = obj\n...     return oid\n...\n>>> def id2obj(oid):\n...     return _id2obj_dict[oid]\n...\n>>> a = A()             # from here, just testing\n>>> a_id = remember(a)\n>>> id2obj(a_id) is a\nTrue\n>>> del a\n>>> gc_collect()  # For PyPy or other GCs.\n>>> try:\n...     id2obj(a_id)\n... except KeyError:\n...     print(\'OK\')\n... else:\n...     print(\'WeakValueDictionary error\')\nOK\n\n'

__test__ = {'libreftest': libreftest}

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite())
    return tests


# --- test body ---
COUNT = 10
self_cbcalled = 0
o1 = C()
o2 = C()
o3 = C()
wkd1 = weakref.WeakKeyDictionary({o1: 1, o2: 2})
wkd2 = weakref.WeakKeyDictionary({o3: 3, o1: 4})
wkd3 = wkd1.copy()
d1 = {o2: '5', o3: '6'}
pairs = [(o2, 7), (o3, 8)]
tmp1 = wkd1 | wkd2

assert dict(tmp1) == dict(wkd1) | dict(wkd2)

assert type(tmp1) is weakref.WeakKeyDictionary
wkd1 |= wkd2

assert wkd1 == tmp1
tmp2 = wkd2 | d1

assert dict(tmp2) == dict(wkd2) | d1

assert type(tmp2) is weakref.WeakKeyDictionary
wkd2 |= d1

assert wkd2 == tmp2
tmp3 = wkd3.copy()
tmp3 |= pairs

assert dict(tmp3) == dict(wkd3) | dict(pairs)

assert type(tmp3) is weakref.WeakKeyDictionary
tmp4 = d1 | wkd3

assert dict(tmp4) == d1 | dict(wkd3)

assert type(tmp4) is weakref.WeakKeyDictionary
del o1

assert 4 not in tmp1.values()

assert 4 not in tmp2.values()

assert 1 not in tmp3.values()

assert 1 not in tmp4.values()
print("MappingTestCase::test_weak_keyed_union_operators: ok")
