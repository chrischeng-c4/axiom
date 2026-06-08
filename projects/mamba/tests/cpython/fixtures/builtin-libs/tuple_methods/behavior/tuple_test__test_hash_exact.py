# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "tuple_methods"
# dimension = "behavior"
# case = "tuple_test__test_hash_exact"
# subject = "cpython.test_tuple.TupleTest.test_hash_exact"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tuple.py::TupleTest::test_hash_exact
"""Auto-ported test: TupleTest::test_hash_exact (CPython 3.12 oracle)."""


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

def check_one_exact(t, e32, e64):
    got = hash(t)
    expected = e32 if support.NHASHBITS == 32 else e64
    if got != expected:
        msg = f'FAIL hash({t!r}) == {got} != {expected}'
        self.fail(msg)
check_one_exact((), 750394483, 5740354900026072187)
check_one_exact((0,), 1214856301, -8753497827991233192)
check_one_exact((0, 0), -168982784, -8458139203682520985)
check_one_exact((0.5,), 2077348973, -408149959306781352)
check_one_exact((0.5, (), (-2, 3, (4, 6))), 714642271, -1845940830829704396)
print("TupleTest::test_hash_exact: ok")
