# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "tuple_methods"
# dimension = "behavior"
# case = "tuple_test__test_repr"
# subject = "cpython.test_tuple.TupleTest.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tuple.py::TupleTest::test_repr
"""Auto-ported test: TupleTest::test_repr (CPython 3.12 oracle)."""


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
l0 = tuple()
l2 = (0, 1, 2)
a0 = type2test(l0)
a2 = type2test(l2)

assert str(a0) == repr(l0)

assert str(a2) == repr(l2)

assert repr(a0) == '()'

assert repr(a2) == '(0, 1, 2)'
print("TupleTest::test_repr: ok")
