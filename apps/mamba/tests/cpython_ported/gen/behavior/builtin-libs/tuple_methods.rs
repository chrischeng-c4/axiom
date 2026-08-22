use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/tuple_methods/tuple_test__test_bug7466.py`.
#[test]
fn test_gen_behavior_builtin_libs_tuple_methods_tuple_test__test_bug7466() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "tuple_methods"
# dimension = "behavior"
# case = "tuple_test__test_bug7466"
# subject = "cpython.test_tuple.TupleTest.test_bug7466"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_tuple.py::TupleTest::test_bug7466
"""Auto-ported test: TupleTest::test_bug7466 (CPython 3.12 oracle)."""


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
_not_tracked(tuple((gc.collect() for i in range(101))))
print("TupleTest::test_bug7466: ok")
"###);
    assert_output(&out, r###"TupleTest::test_bug7466: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/tuple_methods/tuple_test__test_lexicographic_ordering.py`.
#[test]
fn test_gen_behavior_builtin_libs_tuple_methods_tuple_test__test_lexicographic_ordering() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "tuple_methods"
# dimension = "behavior"
# case = "tuple_test__test_lexicographic_ordering"
# subject = "cpython.test_tuple.TupleTest.test_lexicographic_ordering"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_tuple.py::TupleTest::test_lexicographic_ordering
"""Auto-ported test: TupleTest::test_lexicographic_ordering (CPython 3.12 oracle)."""


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
a = type2test([1, 2])
b = type2test([1, 2, 0])
c = type2test([1, 3])

assert a < b

assert b < c
print("TupleTest::test_lexicographic_ordering: ok")
"###);
    assert_output(&out, r###"TupleTest::test_lexicographic_ordering: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/tuple_methods/tuple_test__test_repr.py`.
#[test]
fn test_gen_behavior_builtin_libs_tuple_methods_tuple_test__test_repr() {
    let out = jit_capture(r###"# /// script
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
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
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
"###);
    assert_output(&out, r###"TupleTest::test_repr: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/tuple_methods/tuple_test__test_track_literals.py`.
#[test]
fn test_gen_behavior_builtin_libs_tuple_methods_tuple_test__test_track_literals() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"TupleTest::test_track_literals: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/tuple_methods/tuple_test__test_track_subtypes.py`.
#[test]
fn test_gen_behavior_builtin_libs_tuple_methods_tuple_test__test_track_subtypes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "tuple_methods"
# dimension = "behavior"
# case = "tuple_test__test_track_subtypes"
# subject = "cpython.test_tuple.TupleTest.test_track_subtypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_tuple.py::TupleTest::test_track_subtypes
"""Auto-ported test: TupleTest::test_track_subtypes (CPython 3.12 oracle)."""


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

class MyTuple(tuple):
    pass
check_track_dynamic(MyTuple, True)
print("TupleTest::test_track_subtypes: ok")
"###);
    assert_output(&out, r###"TupleTest::test_track_subtypes: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/tuple_methods/tuple_test__test_tupleresizebug.py`.
#[test]
fn test_gen_behavior_builtin_libs_tuple_methods_tuple_test__test_tupleresizebug() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "tuple_methods"
# dimension = "behavior"
# case = "tuple_test__test_tupleresizebug"
# subject = "cpython.test_tuple.TupleTest.test_tupleresizebug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_tuple.py::TupleTest::test_tupleresizebug
"""Auto-ported test: TupleTest::test_tupleresizebug (CPython 3.12 oracle)."""


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

def f():
    for i in range(1000):
        yield i

assert list(tuple(f())) == list(range(1000))
print("TupleTest::test_tupleresizebug: ok")
"###);
    assert_output(&out, r###"TupleTest::test_tupleresizebug: ok
"###);
}
