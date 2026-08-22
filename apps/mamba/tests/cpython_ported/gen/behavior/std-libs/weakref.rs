use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/weakref/class_object_is_weakreferenceable.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_class_object_is_weakreferenceable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "class_object_is_weakreferenceable"
# subject = "weakref.ref"
# kind = "semantic"
# xfail = "mamba refcount-only: class-object ref does not expire on collection (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: a class object is weak-referenceable; the ref dies once the class is deleted and collected"""
import gc
import weakref


class Throwaway:
    pass


rc = weakref.ref(Throwaway)
assert rc() is Throwaway, "class ref alive"
Throwaway = None
gc.collect()
assert rc() is None, "class ref dead after class deleted"

print("class_object_is_weakreferenceable OK")
"###);
    assert_output(&out, r###"class_object_is_weakreferenceable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/finalize_fires_once_on_collection.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_finalize_fires_once_on_collection() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "finalize_fires_once_on_collection"
# subject = "weakref.finalize"
# kind = "semantic"
# xfail = "mamba refcount-only: finalize does not fire on referent collection (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.finalize: finalize fires exactly once when its object is collected and alive flips to False"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


fired = []
n = _Node(7)
fin = weakref.finalize(n, lambda: fired.append(1))
del n
gc.collect()
assert fired == [1], f"finalize fired once = {fired!r}"
assert not fin.alive, "finalize.alive is False after firing"

print("finalize_fires_once_on_collection OK")
"###);
    assert_output(&out, r###"finalize_fires_once_on_collection OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/finalize_manual_call_flips_alive.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_finalize_manual_call_flips_alive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "finalize_manual_call_flips_alive"
# subject = "weakref.finalize"
# kind = "semantic"
# xfail = "mamba refcount-only: finalize() manual call does not run callback or flip alive (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.finalize: calling a finalize manually runs the callback once and flips alive to False"""
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


fired = []
n = _Node(8)
fin = weakref.finalize(n, lambda: fired.append("manual"))
fin.atexit = False  # don't run at program exit
fin()  # manual call
assert fired == ["manual"], f"manual finalize = {fired!r}"
assert not fin.alive, "finalize.alive False after manual call"

print("finalize_manual_call_flips_alive OK")
"###);
    assert_output(&out, r###"finalize_manual_call_flips_alive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/getweakrefcount_tracks_live_refs.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_getweakrefcount_tracks_live_refs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "getweakrefcount_tracks_live_refs"
# subject = "weakref.getweakrefcount"
# kind = "semantic"
# xfail = "mamba getweakrefcount returns 0 (no weak-ref registry under refcount-only runtime, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.getweakrefcount: getweakrefcount counts distinct live refs (two callback refs = 2) and drops as refs are deleted"""
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


# Refs created with callbacks are always distinct objects, so two of them
# count as two live weak references to the same referent.
n = _Node(10)
r1 = weakref.ref(n, lambda _: None)
r2 = weakref.ref(n, lambda _: None)
assert weakref.getweakrefcount(n) == 2, f"refcount with 2 cbs = {weakref.getweakrefcount(n)!r}"
del r1
assert weakref.getweakrefcount(n) == 1, f"refcount after del r1 = {weakref.getweakrefcount(n)!r}"

print("getweakrefcount_tracks_live_refs OK")
"###);
    assert_output(&out, r###"getweakrefcount_tracks_live_refs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/mapping_test_case__test_remove_closure.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_mapping_test_case__test_remove_closure() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "mapping_test_case__test_remove_closure"
# subject = "cpython.test_weakref.MappingTestCase.test_remove_closure"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::MappingTestCase::test_remove_closure
"""Auto-ported test: MappingTestCase::test_remove_closure (CPython 3.12 oracle)."""


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
d = weakref.WeakValueDictionary()

assert d._remove.__closure__ is None
print("MappingTestCase::test_remove_closure: ok")
"###);
    assert_output(&out, r###"MappingTestCase::test_remove_closure: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/module_test_case__test_names.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_module_test_case__test_names() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "module_test_case__test_names"
# subject = "cpython.test_weakref.ModuleTestCase.test_names"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ModuleTestCase::test_names
"""Auto-ported test: ModuleTestCase::test_names (CPython 3.12 oracle)."""


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
for name in ('ReferenceType', 'ProxyType', 'CallableProxyType', 'WeakMethod', 'WeakSet', 'WeakKeyDictionary', 'WeakValueDictionary'):
    obj = getattr(weakref, name)
    if name != 'WeakSet':

        assert obj.__module__ == 'weakref'

    assert obj.__name__ == name

    assert obj.__qualname__ == name
print("ModuleTestCase::test_names: ok")
"###);
    assert_output(&out, r###"ModuleTestCase::test_names: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/proxy_attribute_forwarding.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_proxy_attribute_forwarding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_attribute_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: proxy forwards attribute reads transparently to the live referent"""
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


n = _Node(5)
p = weakref.proxy(n)
assert p.val == 5, f"proxy attr forwards -> {p.val!r}"

print("proxy_attribute_forwarding OK")
"###);
    assert_output(&out, r###"proxy_attribute_forwarding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/proxy_index_and_bool_forwarding.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_proxy_index_and_bool_forwarding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_index_and_bool_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not protocol-transparent: __index__/__bool__ forwarding diverges (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: operator.index(proxy) forwards __index__ and bool(proxy) mirrors referent truthiness"""
import operator
import weakref


# operator.index() forwards __index__.
class Indexable:
    def __index__(self):
        return 10


idx = Indexable()
assert operator.index(weakref.proxy(idx)) == 10, "proxy __index__"


# bool() mirrors the referent's truthiness.
class EmptyList(list):
    pass


empty = EmptyList()
assert bool(weakref.proxy(empty)) is False, "empty proxy is falsey"
empty.append(1)
assert bool(weakref.proxy(empty)) is True, "non-empty proxy is truthy"

print("proxy_index_and_bool_forwarding OK")
"###);
    assert_output(&out, r###"proxy_index_and_bool_forwarding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/proxy_matmul_operator_forwarding.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_proxy_matmul_operator_forwarding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_matmul_operator_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not operator-transparent: proxy @ diverges from CPython (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: proxy forwards @, reflected @, and @= to __matmul__/__rmatmul__/__imatmul__"""
import weakref


# Matrix-multiply forwarding (@, reflected, in-place).
class Matmul:
    def __matmul__(self, other):
        return 1729

    def __rmatmul__(self, other):
        return -163

    def __imatmul__(self, other):
        return 561


mm = Matmul()
p_mm = weakref.proxy(mm)
assert p_mm @ 5 == 1729, "proxy @"
assert 5 @ p_mm == -163, "reflected @"
p_mm @= 5
assert p_mm == 561, f"proxy @= -> {p_mm!r}"

print("proxy_matmul_operator_forwarding OK")
"###);
    assert_output(&out, r###"proxy_matmul_operator_forwarding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/proxy_numeric_operator_forwarding.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_proxy_numeric_operator_forwarding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_numeric_operator_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not operator-transparent: proxy + float diverges from CPython (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: proxy forwards +, reflected +, // and //= to the referent's numeric dunders"""
import weakref


# Numeric forwarding through a float subclass referent.
class MyFloat(float):
    pass


num = MyFloat(2.0)
p_num = weakref.proxy(num)
assert p_num + 1.0 == 3.0, "proxy + float"
assert 1.0 + p_num == 3.0, "float + proxy (reflected)"


# Floor-division, both normal and in-place.
class Divver:
    def __floordiv__(self, other):
        return 42

    def __ifloordiv__(self, other):
        return 21


div = Divver()
p_div = weakref.proxy(div)
assert p_div // 5 == 42, "proxy //"
p_div //= 5
assert p_div == 21, f"proxy //= -> {p_div!r}"

print("proxy_numeric_operator_forwarding OK")
"###);
    assert_output(&out, r###"proxy_numeric_operator_forwarding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/proxy_raises_referenceerror_after_collection.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_proxy_raises_referenceerror_after_collection() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_raises_referenceerror_after_collection"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba refcount-only: dead proxy does not raise ReferenceError (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: accessing a proxy after its referent is collected raises ReferenceError"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


n = _Node(6)
p = weakref.proxy(n)
del n
gc.collect()
_raised = False
try:
    _ = p.val
except ReferenceError:
    _raised = True
assert _raised, "dead proxy raises ReferenceError"

print("proxy_raises_referenceerror_after_collection OK")
"###);
    assert_output(&out, r###"proxy_raises_referenceerror_after_collection OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/proxy_reversed_and_iteration_forwarding.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_proxy_reversed_and_iteration_forwarding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_reversed_and_iteration_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not protocol-transparent: __reversed__/__iter__ forwarding diverges (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: reversed(proxy) forwards __reversed__ and a proxy over an iterator can drive a for-loop"""
import weakref


# reversed() forwards __reversed__.
class Reversible:
    def __len__(self):
        return 3

    def __reversed__(self):
        return iter("cba")


rev = Reversible()
assert "".join(reversed(weakref.proxy(rev))) == "cba", "reversed(proxy)"


# A proxy whose referent is itself an iterator can drive a for-loop.
def gen():
    yield from [4, 5, 6]


it = gen()


class Iterates:
    def __iter__(self):
        return weakref.proxy(it)


assert list(Iterates()) == [4, 5, 6], "iterate through a proxied iterator"


# Proxying a non-iterator and using it for iteration raises TypeError.
not_iter = lambda: 0


class BadIter:
    def __iter__(self):
        return weakref.proxy(not_iter)


_raised = False
try:
    list(BadIter())
except TypeError:
    _raised = True
assert _raised, "non-iterator proxy raises TypeError on next()"

print("proxy_reversed_and_iteration_forwarding OK")
"###);
    assert_output(&out, r###"proxy_reversed_and_iteration_forwarding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/public_type_name_metadata.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_public_type_name_metadata() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "public_type_name_metadata"
# subject = "weakref.WeakValueDictionary"
# kind = "semantic"
# xfail = "mamba weakref shim classes lack stable __name__/__qualname__/__module__ metadata (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakValueDictionary: public weakref classes expose stable __name__/__qualname__/__module__ dotted-name metadata"""
import weakref


# Public type objects expose clean dotted-name metadata.
for name in (
    "ReferenceType", "ProxyType", "CallableProxyType", "WeakMethod",
    "WeakSet", "WeakKeyDictionary", "WeakValueDictionary",
):
    obj = getattr(weakref, name)
    assert obj.__name__ == name, f"{name}.__name__ -> {obj.__name__!r}"
    assert obj.__qualname__ == name, f"{name}.__qualname__ -> {obj.__qualname__!r}"
    # WeakSet lives in the _weakrefset helper module, the rest in weakref.
    if name != "WeakSet":
        assert obj.__module__ == "weakref", f"{name}.__module__ -> {obj.__module__!r}"

print("public_type_name_metadata OK")
"###);
    assert_output(&out, r###"public_type_name_metadata OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/ref_callback_fires_on_collection.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_ref_callback_fires_on_collection() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "ref_callback_fires_on_collection"
# subject = "weakref.ref"
# kind = "semantic"
# xfail = "mamba refcount-only: weakref collection callbacks never fire (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: a ref() callback fires on referent collection and receives the now-dead ref as its only argument"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


# The callback receives the (now-dead) ref object as its only argument.
seen = []
n = _Node(11)
r = weakref.ref(n, lambda w: seen.append(w))
del n
gc.collect()
assert seen == [r], f"callback arg is the ref = {seen!r}"
assert seen[0]() is None, "ref passed to callback is already dead"

print("ref_callback_fires_on_collection OK")
"###);
    assert_output(&out, r###"ref_callback_fires_on_collection OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/ref_deref_returns_referent.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_ref_deref_returns_referent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "ref_deref_returns_referent"
# subject = "weakref.ref"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref() returns the exact referent object while it is alive; its attributes read through"""
import weakref


class _Obj:
    def __init__(self, v):
        self.v = v


o = _Obj(42)
r = weakref.ref(o)
assert isinstance(r, weakref.ref), f"ref type = {type(r)!r}"
assert r() is o, "deref returns the exact referent"
assert r().v == 42, f"deref.v = {r().v!r}"

print("ref_deref_returns_referent OK")
"###);
    assert_output(&out, r###"ref_deref_returns_referent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/ref_expires_after_referent_collected.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_ref_expires_after_referent_collected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "ref_expires_after_referent_collected"
# subject = "weakref.ref"
# kind = "semantic"
# xfail = "mamba refcount-only: ref does not expire on referent collection (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: a ref() returns None after its referent is deleted and garbage-collected"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


n = _Node(1)
r = weakref.ref(n)
assert r() is not None, "ref alive before del"
del n
gc.collect()
assert r() is None, "ref dead after del+gc"

print("ref_expires_after_referent_collected OK")
"###);
    assert_output(&out, r###"ref_expires_after_referent_collected OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_callback_attribute.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_callback_attribute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_callback_attribute"
# subject = "cpython.test_weakref.ReferencesTestCase.test_callback_attribute"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_callback_attribute
"""Auto-ported test: ReferencesTestCase::test_callback_attribute (CPython 3.12 oracle)."""


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
self_cbcalled = 0
x = Object(1)
callback = lambda ref: None
ref1 = weakref.ref(x, callback)

assert ref1.__callback__ is callback
ref2 = weakref.ref(x)

assert ref2.__callback__ is None
print("ReferencesTestCase::test_callback_attribute: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_callback_attribute: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_callback_different_classes.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_callback_different_classes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_callback_different_classes"
# subject = "cpython.test_weakref.ReferencesTestCase.test_callback_different_classes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_callback_different_classes
"""Auto-ported test: ReferencesTestCase::test_callback_different_classes (CPython 3.12 oracle)."""


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
self_cbcalled = 0
import gc

class C(object):

    def cb(self, ignore):
        self.me
        self.c1
        self.wr

class D:
    pass
c1, c2 = (D(), C())
c2.me = c2
c2.c1 = c1
c2.wr = weakref.ref(c1, c2.cb)
del c1, c2, C, D
gc.collect()
print("ReferencesTestCase::test_callback_different_classes: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_callback_different_classes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_callback_gcs.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_callback_gcs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_callback_gcs"
# subject = "cpython.test_weakref.ReferencesTestCase.test_callback_gcs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_callback_gcs
"""Auto-ported test: ReferencesTestCase::test_callback_gcs (CPython 3.12 oracle)."""


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
self_cbcalled = 0

class ObjectWithDel(Object):

    def __del__(self):
        pass
x = ObjectWithDel(1)
ref1 = weakref.ref(x, lambda ref: support.gc_collect())
del x
support.gc_collect()
print("ReferencesTestCase::test_callback_gcs: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_callback_gcs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_callback_in_cycle.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_callback_in_cycle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_callback_in_cycle"
# subject = "cpython.test_weakref.ReferencesTestCase.test_callback_in_cycle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_callback_in_cycle
"""Auto-ported test: ReferencesTestCase::test_callback_in_cycle (CPython 3.12 oracle)."""


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
self_cbcalled = 0
import gc

class J(object):
    pass

class II(object):

    def acallback(self, ignore):
        self.J
I = II()
I.J = J
I.wr = weakref.ref(J, I.acallback)
del I, J, II
gc.collect()
print("ReferencesTestCase::test_callback_in_cycle: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_callback_in_cycle: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_callback_reachable_one_way.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_callback_reachable_one_way() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_callback_reachable_one_way"
# subject = "cpython.test_weakref.ReferencesTestCase.test_callback_reachable_one_way"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_callback_reachable_one_way
"""Auto-ported test: ReferencesTestCase::test_callback_reachable_one_way (CPython 3.12 oracle)."""


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
self_cbcalled = 0
import gc

class C:

    def cb(self, ignore):
        self.me
        self.c1
        self.wr
c1, c2 = (C(), C())
c2.me = c2
c2.c1 = c1
c2.wr = weakref.ref(c1, c2.cb)
del c1, c2
gc.collect()
print("ReferencesTestCase::test_callback_reachable_one_way: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_callback_reachable_one_way: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_gc_during_proxy_creation.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_gc_during_proxy_creation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_gc_during_proxy_creation"
# subject = "cpython.test_weakref.ReferencesTestCase.test_gc_during_proxy_creation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_gc_during_proxy_creation
"""Auto-ported test: ReferencesTestCase::test_gc_during_proxy_creation (CPython 3.12 oracle)."""


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
def callback(ref):
    self_cbcalled += 1

def check_basic_callback(factory):
    self_cbcalled = 0
    o = factory()
    ref = weakref.ref(o, callback)
    del o
    gc_collect()

    assert self_cbcalled == 1

    assert ref() is None

def check_basic_ref(factory):
    o = factory()
    ref = weakref.ref(o)

    assert ref() is not None
    o2 = ref()

    assert o is o2

def check_gc_during_creation(makeref):
    thresholds = gc.get_threshold()
    gc.set_threshold(1, 1, 1)
    gc.collect()

    class A:
        pass

    def callback(*args):
        pass
    referenced = A()
    a = A()
    a.a = a
    a.wr = makeref(referenced)
    try:
        a = A()
        weakref.ref(referenced, callback)
    finally:
        gc.set_threshold(*thresholds)

def check_proxy(o, proxy):
    o.foo = 1

    assert proxy.foo == 1
    o.foo = 2

    assert proxy.foo == 2
    del o.foo

    assert not hasattr(proxy, 'foo')
    proxy.foo = 1

    assert o.foo == 1
    proxy.foo = 2

    assert o.foo == 2
    del proxy.foo

    assert not hasattr(o, 'foo')

def check_shared_without_callback(makeref):
    o = Object(1)
    p1 = makeref(o, None)
    p2 = makeref(o, None)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o)
    p2 = makeref(o, None)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o)
    p2 = makeref(o)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o, None)
    p2 = makeref(o)

    assert p1 is p2
self_cbcalled = 0
check_gc_during_creation(weakref.proxy)
print("ReferencesTestCase::test_gc_during_proxy_creation: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_gc_during_proxy_creation: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_gc_during_ref_creation.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_gc_during_ref_creation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_gc_during_ref_creation"
# subject = "cpython.test_weakref.ReferencesTestCase.test_gc_during_ref_creation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_gc_during_ref_creation
"""Auto-ported test: ReferencesTestCase::test_gc_during_ref_creation (CPython 3.12 oracle)."""


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
def callback(ref):
    self_cbcalled += 1

def check_basic_callback(factory):
    self_cbcalled = 0
    o = factory()
    ref = weakref.ref(o, callback)
    del o
    gc_collect()

    assert self_cbcalled == 1

    assert ref() is None

def check_basic_ref(factory):
    o = factory()
    ref = weakref.ref(o)

    assert ref() is not None
    o2 = ref()

    assert o is o2

def check_gc_during_creation(makeref):
    thresholds = gc.get_threshold()
    gc.set_threshold(1, 1, 1)
    gc.collect()

    class A:
        pass

    def callback(*args):
        pass
    referenced = A()
    a = A()
    a.a = a
    a.wr = makeref(referenced)
    try:
        a = A()
        weakref.ref(referenced, callback)
    finally:
        gc.set_threshold(*thresholds)

def check_proxy(o, proxy):
    o.foo = 1

    assert proxy.foo == 1
    o.foo = 2

    assert proxy.foo == 2
    del o.foo

    assert not hasattr(proxy, 'foo')
    proxy.foo = 1

    assert o.foo == 1
    proxy.foo = 2

    assert o.foo == 2
    del proxy.foo

    assert not hasattr(o, 'foo')

def check_shared_without_callback(makeref):
    o = Object(1)
    p1 = makeref(o, None)
    p2 = makeref(o, None)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o)
    p2 = makeref(o, None)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o)
    p2 = makeref(o)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o, None)
    p2 = makeref(o)

    assert p1 is p2
self_cbcalled = 0
check_gc_during_creation(weakref.ref)
print("ReferencesTestCase::test_gc_during_ref_creation: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_gc_during_ref_creation: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_proxy_bool.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_proxy_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_proxy_bool"
# subject = "cpython.test_weakref.ReferencesTestCase.test_proxy_bool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_proxy_bool
"""Auto-ported test: ReferencesTestCase::test_proxy_bool (CPython 3.12 oracle)."""


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
self_cbcalled = 0

class List(list):
    pass
lyst = List()

assert bool(weakref.proxy(lyst)) == bool(lyst)
print("ReferencesTestCase::test_proxy_bool: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_proxy_bool: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_proxy_index.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_proxy_index() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_proxy_index"
# subject = "cpython.test_weakref.ReferencesTestCase.test_proxy_index"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_proxy_index
"""Auto-ported test: ReferencesTestCase::test_proxy_index (CPython 3.12 oracle)."""


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
self_cbcalled = 0

class C:

    def __index__(self):
        return 10
o = C()
p = weakref.proxy(o)

assert operator.index(p) == 10
print("ReferencesTestCase::test_proxy_index: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_proxy_index: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_proxy_reuse.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_proxy_reuse() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_proxy_reuse"
# subject = "cpython.test_weakref.ReferencesTestCase.test_proxy_reuse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_proxy_reuse
"""Auto-ported test: ReferencesTestCase::test_proxy_reuse (CPython 3.12 oracle)."""


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
self_cbcalled = 0
o = C()
proxy1 = weakref.proxy(o)
ref = weakref.ref(o)
proxy2 = weakref.proxy(o)

assert proxy1 is proxy2
print("ReferencesTestCase::test_proxy_reuse: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_proxy_reuse: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_ref_created_during_del.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_ref_created_during_del() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_ref_created_during_del"
# subject = "cpython.test_weakref.ReferencesTestCase.test_ref_created_during_del"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_ref_created_during_del
"""Auto-ported test: ReferencesTestCase::test_ref_created_during_del (CPython 3.12 oracle)."""


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
self_cbcalled = 0

class Target(object):

    def __del__(self):
        global ref_from_del
        ref_from_del = weakref.ref(self)
w = Target()
print("ReferencesTestCase::test_ref_created_during_del: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_ref_created_during_del: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_set_callback_attribute.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_set_callback_attribute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_set_callback_attribute"
# subject = "cpython.test_weakref.ReferencesTestCase.test_set_callback_attribute"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_set_callback_attribute
"""Auto-ported test: ReferencesTestCase::test_set_callback_attribute (CPython 3.12 oracle)."""


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
self_cbcalled = 0
x = Object(1)
callback = lambda ref: None
ref1 = weakref.ref(x, callback)
try:
    ref1.__callback__ = lambda ref: None
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("ReferencesTestCase::test_set_callback_attribute: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_set_callback_attribute: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_sf_bug_840829.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_sf_bug_840829() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_sf_bug_840829"
# subject = "cpython.test_weakref.ReferencesTestCase.test_sf_bug_840829"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_sf_bug_840829
"""Auto-ported test: ReferencesTestCase::test_sf_bug_840829 (CPython 3.12 oracle)."""


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
self_cbcalled = 0
import gc

class C(object):
    pass
c = C()
wr = weakref.ref(c, lambda ignore: gc.collect())
del c
del wr
c1 = C()
c1.i = C()
wr = weakref.ref(c1.i, lambda ignore: gc.collect())
c2 = C()
c2.c1 = c1
del c1
del c2
print("ReferencesTestCase::test_sf_bug_840829: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_sf_bug_840829: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_shared_proxy_without_callback.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_shared_proxy_without_callback() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_shared_proxy_without_callback"
# subject = "cpython.test_weakref.ReferencesTestCase.test_shared_proxy_without_callback"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_shared_proxy_without_callback
"""Auto-ported test: ReferencesTestCase::test_shared_proxy_without_callback (CPython 3.12 oracle)."""


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
def callback(ref):
    self_cbcalled += 1

def check_basic_callback(factory):
    self_cbcalled = 0
    o = factory()
    ref = weakref.ref(o, callback)
    del o
    gc_collect()

    assert self_cbcalled == 1

    assert ref() is None

def check_basic_ref(factory):
    o = factory()
    ref = weakref.ref(o)

    assert ref() is not None
    o2 = ref()

    assert o is o2

def check_gc_during_creation(makeref):
    thresholds = gc.get_threshold()
    gc.set_threshold(1, 1, 1)
    gc.collect()

    class A:
        pass

    def callback(*args):
        pass
    referenced = A()
    a = A()
    a.a = a
    a.wr = makeref(referenced)
    try:
        a = A()
        weakref.ref(referenced, callback)
    finally:
        gc.set_threshold(*thresholds)

def check_proxy(o, proxy):
    o.foo = 1

    assert proxy.foo == 1
    o.foo = 2

    assert proxy.foo == 2
    del o.foo

    assert not hasattr(proxy, 'foo')
    proxy.foo = 1

    assert o.foo == 1
    proxy.foo = 2

    assert o.foo == 2
    del proxy.foo

    assert not hasattr(o, 'foo')

def check_shared_without_callback(makeref):
    o = Object(1)
    p1 = makeref(o, None)
    p2 = makeref(o, None)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o)
    p2 = makeref(o, None)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o)
    p2 = makeref(o)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o, None)
    p2 = makeref(o)

    assert p1 is p2
self_cbcalled = 0
check_shared_without_callback(weakref.proxy)
print("ReferencesTestCase::test_shared_proxy_without_callback: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_shared_proxy_without_callback: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_shared_ref_without_callback.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_shared_ref_without_callback() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_shared_ref_without_callback"
# subject = "cpython.test_weakref.ReferencesTestCase.test_shared_ref_without_callback"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_shared_ref_without_callback
"""Auto-ported test: ReferencesTestCase::test_shared_ref_without_callback (CPython 3.12 oracle)."""


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
def callback(ref):
    self_cbcalled += 1

def check_basic_callback(factory):
    self_cbcalled = 0
    o = factory()
    ref = weakref.ref(o, callback)
    del o
    gc_collect()

    assert self_cbcalled == 1

    assert ref() is None

def check_basic_ref(factory):
    o = factory()
    ref = weakref.ref(o)

    assert ref() is not None
    o2 = ref()

    assert o is o2

def check_gc_during_creation(makeref):
    thresholds = gc.get_threshold()
    gc.set_threshold(1, 1, 1)
    gc.collect()

    class A:
        pass

    def callback(*args):
        pass
    referenced = A()
    a = A()
    a.a = a
    a.wr = makeref(referenced)
    try:
        a = A()
        weakref.ref(referenced, callback)
    finally:
        gc.set_threshold(*thresholds)

def check_proxy(o, proxy):
    o.foo = 1

    assert proxy.foo == 1
    o.foo = 2

    assert proxy.foo == 2
    del o.foo

    assert not hasattr(proxy, 'foo')
    proxy.foo = 1

    assert o.foo == 1
    proxy.foo = 2

    assert o.foo == 2
    del proxy.foo

    assert not hasattr(o, 'foo')

def check_shared_without_callback(makeref):
    o = Object(1)
    p1 = makeref(o, None)
    p2 = makeref(o, None)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o)
    p2 = makeref(o, None)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o)
    p2 = makeref(o)

    assert p1 is p2
    del p1, p2
    p1 = makeref(o, None)
    p2 = makeref(o)

    assert p1 is p2
self_cbcalled = 0
check_shared_without_callback(weakref.ref)
print("ReferencesTestCase::test_shared_ref_without_callback: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_shared_ref_without_callback: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/references_test_case__test_trashcan_16602.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_references_test_case__test_trashcan_16602() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_trashcan_16602"
# subject = "cpython.test_weakref.ReferencesTestCase.test_trashcan_16602"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::ReferencesTestCase::test_trashcan_16602
"""Auto-ported test: ReferencesTestCase::test_trashcan_16602 (CPython 3.12 oracle)."""


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
self_cbcalled = 0

class C:

    def __init__(self, parent):
        if not parent:
            return
        wself = weakref.ref(self)

        def cb(wparent):
            o = wself()
        self.wparent = weakref.ref(parent, cb)
d = weakref.WeakKeyDictionary()
root = c = C(None)
for n in range(100):
    d[c] = c = C(c)
del root
gc.collect()
print("ReferencesTestCase::test_trashcan_16602: ok")
"###);
    assert_output(&out, r###"ReferencesTestCase::test_trashcan_16602: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/subclassable_weakref_test_case__test_subclass_refs_with_cycle.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_subclassable_weakref_test_case__test_subclass_refs_with_cycle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "subclassable_weakref_test_case__test_subclass_refs_with_cycle"
# subject = "cpython.test_weakref.SubclassableWeakrefTestCase.test_subclass_refs_with_cycle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakref.py::SubclassableWeakrefTestCase::test_subclass_refs_with_cycle
"""Auto-ported test: SubclassableWeakrefTestCase::test_subclass_refs_with_cycle (CPython 3.12 oracle)."""


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
self_cbcalled = 0
'Confirm https://bugs.python.org/issue3100 is fixed.'

class MyRef(weakref.ref):
    pass

def callback(w):
    self.cbcalled += 1
o = C()
r1 = MyRef(o, callback)
r1.o = o
del o
del r1

assert self_cbcalled == 0
o = C()
r1 = MyRef(o, callback)
r2 = MyRef(o, callback)
r1.r = r2
r2.o = o
del o
del r2
del r1

assert self_cbcalled == 0
print("SubclassableWeakrefTestCase::test_subclass_refs_with_cycle: ok")
"###);
    assert_output(&out, r###"SubclassableWeakrefTestCase::test_subclass_refs_with_cycle: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/weakkeydictionary_autoremoves_on_collection.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_weakkeydictionary_autoremoves_on_collection() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "weakkeydictionary_autoremoves_on_collection"
# subject = "weakref.WeakKeyDictionary"
# kind = "semantic"
# xfail = "mamba refcount-only: WeakKeyDictionary does not auto-remove collected keys (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakKeyDictionary: WeakKeyDictionary drops an entry once its key is collected"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


wkd = weakref.WeakKeyDictionary()
n = _Node(4)
wkd[n] = "data"
assert n in wkd, "key present"
del n
gc.collect()
assert len(wkd) == 0, f"WeakKeyDictionary auto-removed = {len(wkd)!r}"

print("weakkeydictionary_autoremoves_on_collection OK")
"###);
    assert_output(&out, r###"weakkeydictionary_autoremoves_on_collection OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/weakset_autoremoves_on_collection.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_weakset_autoremoves_on_collection() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "weakset_autoremoves_on_collection"
# subject = "weakref.WeakSet"
# kind = "semantic"
# xfail = "mamba refcount-only: WeakSet does not auto-remove collected members (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakSet: WeakSet drops a member once it is collected"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


ws = weakref.WeakSet()
n = _Node(9)
ws.add(n)
assert n in ws, "WeakSet contains node"
del n
gc.collect()
assert len(ws) == 0, f"WeakSet cleared after GC = {len(ws)!r}"

print("weakset_autoremoves_on_collection OK")
"###);
    assert_output(&out, r###"weakset_autoremoves_on_collection OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakref/weakvaluedictionary_autoremoves_on_collection.py`.
#[test]
fn test_gen_behavior_std_libs_weakref_weakvaluedictionary_autoremoves_on_collection() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "weakvaluedictionary_autoremoves_on_collection"
# subject = "weakref.WeakValueDictionary"
# kind = "semantic"
# xfail = "mamba refcount-only: WeakValueDictionary does not auto-remove collected values (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakValueDictionary: WeakValueDictionary drops an entry once its value is collected"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


wvd = weakref.WeakValueDictionary()
n = _Node(3)
wvd["x"] = n
assert "x" in wvd, "entry present"
del n
gc.collect()
assert "x" not in wvd, "entry auto-removed after GC"

print("weakvaluedictionary_autoremoves_on_collection OK")
"###);
    assert_output(&out, r###"weakvaluedictionary_autoremoves_on_collection OK
"###);
}
