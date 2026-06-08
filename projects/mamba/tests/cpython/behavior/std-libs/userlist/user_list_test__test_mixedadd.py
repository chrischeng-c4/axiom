# /// script
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
