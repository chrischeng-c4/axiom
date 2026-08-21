# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "tuple_methods"
# dimension = "behavior"
# case = "tuple_test__test_track_literals"
# subject = "cpython.test_tuple.TupleTest.test_track_literals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_tuple.py::TupleTest::test_track_literals
"""Auto-ported test: TupleTest::test_track_literals (CPython 3.12 oracle)."""


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
x, y, z = (1.5, 'a', [])
_not_tracked(())
_not_tracked((1,))
_not_tracked((1, 2))
_not_tracked((1, 2, 'a'))
_not_tracked((1, 2, (None, True, False, ()), int))
_not_tracked((object(),))
_not_tracked(((1, x), y, (2, 3)))
_tracked(([],))
_tracked(([1],))
_tracked(({},))
_tracked((set(),))
_tracked((x, y, z))
print("TupleTest::test_track_literals: ok")
