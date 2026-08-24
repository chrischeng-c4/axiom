use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/list_methods/list_test__test_basic.py`.
#[test]
fn test_gen_behavior_builtin_libs_list_methods_list_test__test_basic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_basic"
# subject = "cpython.test_list.ListTest.test_basic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_basic
"""Auto-ported test: ListTest::test_basic (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list

assert list([]) == []
l0_3 = [0, 1, 2, 3]
l0_3_bis = list(l0_3)

assert l0_3 == l0_3_bis

assert l0_3 is not l0_3_bis

assert list(()) == []

assert list((0, 1, 2, 3)) == [0, 1, 2, 3]

assert list('') == []

assert list('spam') == ['s', 'p', 'a', 'm']

assert list((x for x in range(10) if x % 2)) == [1, 3, 5, 7, 9]
if sys.maxsize == 2147483647:

    try:
        list(range(sys.maxsize // 2))
        raise AssertionError('expected MemoryError')
    except MemoryError:
        pass
x = []
x.extend((-y for y in x))

assert x == []
print("ListTest::test_basic: ok")
"###);
    assert_output(&out, r###"ListTest::test_basic: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/list_methods/list_test__test_identity.py`.
#[test]
fn test_gen_behavior_builtin_libs_list_methods_list_test__test_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_identity"
# subject = "cpython.test_list.ListTest.test_identity"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_identity
"""Auto-ported test: ListTest::test_identity (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list

assert [] is not []
print("ListTest::test_identity: ok")
"###);
    assert_output(&out, r###"ListTest::test_identity: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/list_methods/list_test__test_len.py`.
#[test]
fn test_gen_behavior_builtin_libs_list_methods_list_test__test_len() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_len"
# subject = "cpython.test_list.ListTest.test_len"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_len
"""Auto-ported test: ListTest::test_len (CPython 3.12 oracle)."""


type2test = list

assert len(type2test()) == 0
assert len(type2test([0])) == 1
assert len(type2test([0, 1, 2])) == 3
assert len([]) == 0
assert len([0]) == 1
assert len([0, 1, 2]) == 3

print("ListTest::test_len: ok")
"###);
    assert_output(&out, r###"ListTest::test_len: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/list_methods/list_test__test_preallocation.py`.
#[test]
fn test_gen_behavior_builtin_libs_list_methods_list_test__test_preallocation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_preallocation"
# subject = "cpython.test_list.ListTest.test_preallocation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_preallocation
"""Auto-ported test: ListTest::test_preallocation (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list
iterable = [0] * 10
iter_size = sys.getsizeof(iterable)

assert iter_size == sys.getsizeof(list([0] * 10))

assert iter_size == sys.getsizeof(list(range(10)))
print("ListTest::test_preallocation: ok")
"###);
    assert_output(&out, r###"ListTest::test_preallocation: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/list_methods/list_test__test_repr_large.py`.
#[test]
fn test_gen_behavior_builtin_libs_list_methods_list_test__test_repr_large() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_repr_large"
# subject = "cpython.test_list.ListTest.test_repr_large"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_repr_large
"""Auto-ported test: ListTest::test_repr_large (CPython 3.12 oracle)."""


def check(n):
    values = [0] * n
    rendered = repr(values)
    assert rendered == "[" + ", ".join(["0"] * n) + "]"


check(10)
check(1_000_000)

print("ListTest::test_repr_large: ok")
"###);
    assert_output(&out, r###"ListTest::test_repr_large: ok
"###);
}
