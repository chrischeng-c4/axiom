# /// script
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
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_userlist.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
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
