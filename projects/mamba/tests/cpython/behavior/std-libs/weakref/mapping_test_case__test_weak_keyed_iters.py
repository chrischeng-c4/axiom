# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "mapping_test_case__test_weak_keyed_iters"
# subject = "cpython.test_weakref.MappingTestCase.test_weak_keyed_iters"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_weakref.py::MappingTestCase::test_weak_keyed_iters
"""Auto-ported test: MappingTestCase::test_weak_keyed_iters (CPython 3.12 oracle)."""


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

def callback(ref):
    self_cbcalled += 1

def check_iters(dict):
    items = list(dict.items())
    for item in dict.items():
        items.remove(item)

    assert not items
    keys = list(dict.keys())
    for k in dict:
        keys.remove(k)

    assert not keys
    keys = list(dict.keys())
    for k in dict.keys():
        keys.remove(k)

    assert not keys
    values = list(dict.values())
    for v in dict.values():
        values.remove(v)

    assert not values

def check_len_cycles(dict_type, cons):
    N = 20
    items = [RefCycle() for i in range(N)]
    dct = dict_type((cons(o) for o in items))
    it = dct.items()
    try:
        next(it)
    except StopIteration:
        pass
    del items
    gc.collect()
    n1 = len(dct)
    del it
    gc.collect()
    n2 = len(dct)

    assert n1 in (0, 1)

    assert n2 == 0

def check_len_race(dict_type, cons):
    pass
    for th in range(1, 100):
        N = 20
        gc.collect(0)
        gc.set_threshold(th, th, th)
        items = [RefCycle() for i in range(N)]
        dct = dict_type((cons(o) for o in items))
        del items
        it = dct.items()
        try:
            next(it)
        except StopIteration:
            pass
        n1 = len(dct)
        del it
        n2 = len(dct)

        assert n1 >= 0

        assert n1 <= N

        assert n2 >= 0

        assert n2 <= n1

def check_popitem(klass, key1, value1, key2, value2):
    weakdict = klass()
    weakdict[key1] = value1
    weakdict[key2] = value2

    assert len(weakdict) == 2
    k, v = weakdict.popitem()

    assert len(weakdict) == 1
    if k is key1:

        assert v is value1
    else:

        assert v is value2
    k, v = weakdict.popitem()

    assert len(weakdict) == 0
    if k is key1:

        assert v is value1
    else:

        assert v is value2

def check_setdefault(klass, key, value1, value2):

    assert value1 is not value2
    weakdict = klass()
    o = weakdict.setdefault(key, value1)

    assert o is value1

    assert key in weakdict

    assert weakdict.get(key) is value1

    assert weakdict[key] is value1
    o = weakdict.setdefault(key, value2)

    assert o is value1

    assert key in weakdict

    assert weakdict.get(key) is value1

    assert weakdict[key] is value1

def check_threaded_weak_dict_copy(type_, deepcopy):
    exc = []

    class DummyKey:

        def __init__(self, ctr):
            self.ctr = ctr

    class DummyValue:

        def __init__(self, ctr):
            self.ctr = ctr

    def dict_copy(d, exc):
        try:
            if deepcopy is True:
                _ = copy.deepcopy(d)
            else:
                _ = d.copy()
        except Exception as ex:
            exc.append(ex)

    def pop_and_collect(lst):
        gc_ctr = 0
        while lst:
            i = random.randint(0, len(lst) - 1)
            gc_ctr += 1
            lst.pop(i)
            if gc_ctr % 10000 == 0:
                gc.collect()

    assert type_ in (weakref.WeakKeyDictionary, weakref.WeakValueDictionary)
    d = type_()
    keys = []
    values = []
    for i in range(70000):
        k, v = (DummyKey(i), DummyValue(i))
        keys.append(k)
        values.append(v)
        d[k] = v
        del k
        del v
    t_copy = threading.Thread(target=dict_copy, args=(d, exc))
    if type_ is weakref.WeakKeyDictionary:
        t_collect = threading.Thread(target=pop_and_collect, args=(keys,))
    else:
        t_collect = threading.Thread(target=pop_and_collect, args=(values,))
    t_copy.start()
    t_collect.start()
    t_copy.join()
    t_collect.join()
    if exc:
        raise exc[0]

def check_update(klass, dict):
    weakdict = klass()
    weakdict.update(dict)

    assert len(weakdict) == len(dict)
    for k in weakdict.keys():

        assert k in dict
        v = dict.get(k)

        assert v is weakdict[k]

        assert v is weakdict.get(k)
    for k in dict.keys():

        assert k in weakdict
        v = dict[k]

        assert v is weakdict[k]

        assert v is weakdict.get(k)

def check_weak_del_and_len_while_iterating(dict, testcontext):
    o = Object(123456)
    with testcontext():
        n = len(dict)
        dict.pop(next(dict.keys()))

        assert len(dict) == n - 1
        dict[o] = o

        assert len(dict) == n
    with testcontext():

        assert len(dict) == n - 1
        dict.popitem()

        assert len(dict) == n - 2
    with testcontext():

        assert len(dict) == n - 3
        del dict[next(dict.keys())]

        assert len(dict) == n - 4
    with testcontext():

        assert len(dict) == n - 5
        dict.popitem()

        assert len(dict) == n - 6
    with testcontext():
        dict.clear()

        assert len(dict) == 0

    assert len(dict) == 0

def check_weak_destroy_and_mutate_while_iterating(dict, testcontext):
    with testcontext() as (k, v):

        assert k not in dict
    with testcontext() as (k, v):

        try:
            dict.__delitem__(k)
            raise AssertionError('expected KeyError')
        except KeyError:
            pass

    assert k not in dict
    with testcontext() as (k, v):

        try:
            dict.pop(k)
            raise AssertionError('expected KeyError')
        except KeyError:
            pass

    assert k not in dict
    with testcontext() as (k, v):
        dict[k] = v

    assert dict[k] == v
    ddict = copy.copy(dict)
    with testcontext() as (k, v):
        dict.update(ddict)

    assert dict == ddict
    with testcontext() as (k, v):
        dict.clear()

    assert len(dict) == 0

def check_weak_destroy_while_iterating(dict, objects, iter_name):
    n = len(dict)
    it = iter(getattr(dict, iter_name)())
    next(it)
    del objects[-1]
    gc.collect()

    assert len(list(it)) in [len(objects), len(objects) - 1]
    del it

    assert len(dict) == n - 1

def make_weak_keyed_dict():
    dict = weakref.WeakKeyDictionary()
    objects = list(map(Object, range(COUNT)))
    for o in objects:
        dict[o] = o.arg
    return (dict, objects)

def make_weak_valued_dict():
    dict = weakref.WeakValueDictionary()
    objects = list(map(Object, range(COUNT)))
    for o in objects:
        dict[o.arg] = o
    return (dict, objects)
self_cbcalled = 0
dict, objects = make_weak_keyed_dict()
check_iters(dict)
refs = dict.keyrefs()

assert len(refs) == len(objects)
objects2 = list(objects)
for wr in refs:
    ob = wr()

    assert ob in dict

    assert ob in dict

    assert ob.arg == dict[ob]
    objects2.remove(ob)

assert len(objects2) == 0
objects2 = list(objects)

assert len(list(dict.keyrefs())) == len(objects)
for wr in dict.keyrefs():
    ob = wr()

    assert ob in dict

    assert ob in dict

    assert ob.arg == dict[ob]
    objects2.remove(ob)

assert len(objects2) == 0
print("MappingTestCase::test_weak_keyed_iters: ok")
