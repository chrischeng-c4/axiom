# /// script
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
