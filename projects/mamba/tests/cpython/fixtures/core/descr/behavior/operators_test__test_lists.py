# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "operators_test__test_lists"
# subject = "cpython.test_descr.OperatorsTest.test_lists"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::OperatorsTest::test_lists
"""Auto-ported test: OperatorsTest::test_lists (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
def binop_test(a, b, res, expr='a+b', meth='__add__'):
    d = {'a': a, 'b': b}

    assert eval(expr, d) == res
    t = type(a)
    m = getattr(t, meth)
    while meth not in t.__dict__:
        t = t.__bases__[0]

    assert getattr(m, 'im_func', m) == t.__dict__[meth]

    assert m(a, b) == res
    bm = getattr(a, meth)

    assert bm(b) == res

def number_operators(a, b, skip=[]):
    dict = {'a': a, 'b': b}
    for name, expr in self_binops.items():
        if name not in skip:
            name = '__%s__' % name
            if hasattr(a, name):
                res = eval(expr, dict)
                binop_test(a, b, res, expr, name)
    for name, expr in list(self_unops.items()):
        if name not in skip:
            name = '__%s__' % name
            if hasattr(a, name):
                res = eval(expr, dict)
                unop_test(a, res, expr, name)

def set2op_test(a, b, c, res, stmt='a[b]=c', meth='__setitem__'):
    d = {'a': deepcopy(a), 'b': b, 'c': c}
    exec(stmt, d)

    assert d['a'] == res
    t = type(a)
    m = getattr(t, meth)
    while meth not in t.__dict__:
        t = t.__bases__[0]

    assert getattr(m, 'im_func', m) == t.__dict__[meth]
    d['a'] = deepcopy(a)
    m(d['a'], b, c)

    assert d['a'] == res
    d['a'] = deepcopy(a)
    bm = getattr(d['a'], meth)
    bm(b, c)

    assert d['a'] == res

def setop_test(a, b, res, stmt='a+=b', meth='__iadd__'):
    d = {'a': deepcopy(a), 'b': b}
    exec(stmt, d)

    assert d['a'] == res
    t = type(a)
    m = getattr(t, meth)
    while meth not in t.__dict__:
        t = t.__bases__[0]

    assert getattr(m, 'im_func', m) == t.__dict__[meth]
    d['a'] = deepcopy(a)
    m(d['a'], b)

    assert d['a'] == res
    d['a'] = deepcopy(a)
    bm = getattr(d['a'], meth)
    bm(b)

    assert d['a'] == res

def setsliceop_test(a, b, c, d, res, stmt='a[b:c]=d', meth='__setitem__'):
    dictionary = {'a': deepcopy(a), 'b': b, 'c': c, 'd': d}
    exec(stmt, dictionary)

    assert dictionary['a'] == res
    t = type(a)
    while meth not in t.__dict__:
        t = t.__bases__[0]
    m = getattr(t, meth)

    assert getattr(m, 'im_func', m) == t.__dict__[meth]
    dictionary['a'] = deepcopy(a)
    m(dictionary['a'], slice(b, c), d)

    assert dictionary['a'] == res
    dictionary['a'] = deepcopy(a)
    bm = getattr(dictionary['a'], meth)
    bm(slice(b, c), d)

    assert dictionary['a'] == res

def sliceop_test(a, b, c, res, expr='a[b:c]', meth='__getitem__'):
    d = {'a': a, 'b': b, 'c': c}

    assert eval(expr, d) == res
    t = type(a)
    m = getattr(t, meth)
    while meth not in t.__dict__:
        t = t.__bases__[0]

    assert getattr(m, 'im_func', m) == t.__dict__[meth]

    assert m(a, slice(b, c)) == res
    bm = getattr(a, meth)

    assert bm(slice(b, c)) == res

def unop_test(a, res, expr='len(a)', meth='__len__'):
    d = {'a': a}

    assert eval(expr, d) == res
    t = type(a)
    m = getattr(t, meth)
    while meth not in t.__dict__:
        t = t.__bases__[0]

    assert getattr(m, 'im_func', m) == t.__dict__[meth]

    assert m(a) == res
    bm = getattr(a, meth)

    assert bm() == res
binop_test([1], [2], [1, 2], 'a+b', '__add__')
binop_test([1, 2, 3], 2, 1, 'b in a', '__contains__')
binop_test([1, 2, 3], 4, 0, 'b in a', '__contains__')
binop_test([1, 2, 3], 1, 2, 'a[b]', '__getitem__')
sliceop_test([1, 2, 3], 0, 2, [1, 2], 'a[b:c]', '__getitem__')
setop_test([1], [2], [1, 2], 'a+=b', '__iadd__')
setop_test([1, 2], 3, [1, 2, 1, 2, 1, 2], 'a*=b', '__imul__')
unop_test([1, 2, 3], 3, 'len(a)', '__len__')
binop_test([1, 2], 3, [1, 2, 1, 2, 1, 2], 'a*b', '__mul__')
binop_test([1, 2], 3, [1, 2, 1, 2, 1, 2], 'b*a', '__rmul__')
set2op_test([1, 2], 1, 3, [1, 3], 'a[b]=c', '__setitem__')
setsliceop_test([1, 2, 3, 4], 1, 3, [5, 6], [1, 5, 6, 4], 'a[b:c]=d', '__setitem__')
print("OperatorsTest::test_lists: ok")
