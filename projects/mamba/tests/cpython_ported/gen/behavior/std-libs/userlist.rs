use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/userlist/user_list_test__test_mixedadd.py`.
#[test]
fn test_gen_behavior_std_libs_userlist_user_list_test__test_mixedadd() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userlist"
# dimension = "behavior"
# case = "user_list_test__test_mixedadd"
# subject = "cpython.test_userlist.UserListTest.test_mixedadd"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_userlist.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_userlist.py::UserListTest::test_mixedadd
"""Auto-ported test: UserListTest::test_mixedadd (CPython 3.12 oracle)."""


from collections import UserList
from test import list_tests
import unittest


# --- test body ---
type2test = UserList
u = type2test([0, 1])

assert u + [] == u

assert u + [2] == [0, 1, 2]
print("UserListTest::test_mixedadd: ok")
"###);
    assert_output(&out, r###"UserListTest::test_mixedadd: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/userlist/user_list_test__test_mixedcmp.py`.
#[test]
fn test_gen_behavior_std_libs_userlist_user_list_test__test_mixedcmp() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userlist"
# dimension = "behavior"
# case = "user_list_test__test_mixedcmp"
# subject = "cpython.test_userlist.UserListTest.test_mixedcmp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_userlist.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_userlist.py::UserListTest::test_mixedcmp
"""Auto-ported test: UserListTest::test_mixedcmp (CPython 3.12 oracle)."""


from collections import UserList
from test import list_tests
import unittest


# --- test body ---
type2test = UserList
u = type2test([0, 1])

assert u == [0, 1]

assert u != [0]

assert u != [0, 2]
print("UserListTest::test_mixedcmp: ok")
"###);
    assert_output(&out, r###"UserListTest::test_mixedcmp: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/userlist/user_list_test__test_userlist_copy.py`.
#[test]
fn test_gen_behavior_std_libs_userlist_user_list_test__test_userlist_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userlist"
# dimension = "behavior"
# case = "user_list_test__test_userlist_copy"
# subject = "cpython.test_userlist.UserListTest.test_userlist_copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_userlist.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_userlist.py::UserListTest::test_userlist_copy
"""Auto-ported test: UserListTest::test_userlist_copy (CPython 3.12 oracle)."""


from collections import UserList
from test import list_tests
import unittest


# --- test body ---
type2test = UserList
u = type2test([6, 8, 1, 9, 1])
v = u.copy()

assert u == v

assert type(u) == type(v)
print("UserListTest::test_userlist_copy: ok")
"###);
    assert_output(&out, r###"UserListTest::test_userlist_copy: ok
"###);
}
