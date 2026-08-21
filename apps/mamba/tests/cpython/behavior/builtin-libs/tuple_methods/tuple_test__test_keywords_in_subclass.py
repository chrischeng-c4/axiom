# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "tuple_methods"
# dimension = "behavior"
# case = "tuple_test__test_keywords_in_subclass"
# subject = "cpython.test_tuple.TupleTest.test_keywords_in_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tuple.py::TupleTest::test_keywords_in_subclass
"""Auto-ported test: TupleTest::test_keywords_in_subclass (CPython 3.12 oracle)."""


from test import support, seq_tests
import unittest
import gc
import pickle


RUN_ALL_HASH_TESTS = False

JUST_SHOW_HASH_RESULTS = False


# --- test body ---
type2test = tuple

def _not_tracked(t):
    gc.collect()
    gc.collect()

    assert not gc.is_tracked(t)

def _tracked(t):

    assert gc.is_tracked(t)
    gc.collect()
    gc.collect()

    assert gc.is_tracked(t)

def check_track_dynamic(tp, always_track):
    x, y, z = (1.5, 'a', [])
    check = _tracked if always_track else _not_tracked
    check(tp())
    check(tp([]))
    check(tp(set()))
    check(tp([1, x, y]))
    check(tp((obj for obj in [1, x, y])))
    check(tp(set([1, x, y])))
    check(tp((tuple([obj]) for obj in [1, x, y])))
    check(tuple((tp([obj]) for obj in [1, x, y])))
    _tracked(tp([z]))
    _tracked(tp([[x, y]]))
    _tracked(tp([{x: y}]))
    _tracked(tp((obj for obj in [x, y, z])))
    _tracked(tp((tuple([obj]) for obj in [x, y, z])))
    _tracked(tuple((tp([obj]) for obj in [x, y, z])))

class subclass(tuple):
    pass
u = subclass([1, 2])

assert type(u) is subclass

assert list(u) == [1, 2]
try:
    subclass(sequence=())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class subclass_with_init(tuple):

    def __init__(self, arg, newarg=None):
        self.newarg = newarg
u = subclass_with_init([1, 2], newarg=3)

assert type(u) is subclass_with_init

assert list(u) == [1, 2]

assert u.newarg == 3

class subclass_with_new(tuple):

    def __new__(cls, arg, newarg=None):
        self = super().__new__(cls, arg)
        self.newarg = newarg
        return self
u = subclass_with_new([1, 2], newarg=3)

assert type(u) is subclass_with_new

assert list(u) == [1, 2]

assert u.newarg == 3
print("TupleTest::test_keywords_in_subclass: ok")
