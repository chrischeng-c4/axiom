# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "mapping_test_case__test_make_weak_valued_dict_from_weak_valued_dict"
# subject = "cpython.test_weakref.MappingTestCase.test_make_weak_valued_dict_from_weak_valued_dict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_weakref.py::MappingTestCase::test_make_weak_valued_dict_from_weak_valued_dict
"""Auto-ported test: MappingTestCase::test_make_weak_valued_dict_from_weak_valued_dict (CPython 3.12 oracle)."""


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
o = Object(3)
dict = weakref.WeakValueDictionary({364: o})
dict2 = weakref.WeakValueDictionary(dict)

assert dict[364] == o
print("MappingTestCase::test_make_weak_valued_dict_from_weak_valued_dict: ok")
