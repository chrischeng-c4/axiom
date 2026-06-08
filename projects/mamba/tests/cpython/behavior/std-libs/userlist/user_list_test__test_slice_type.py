# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userlist"
# dimension = "behavior"
# case = "user_list_test__test_slice_type"
# subject = "cpython.test_userlist.UserListTest.test_slice_type"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_userlist.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_userlist.py::UserListTest::test_slice_type
"""Auto-ported test: UserListTest::test_slice_type (CPython 3.12 oracle)."""


from collections import UserList
from test import list_tests
import unittest


# --- test body ---
type2test = UserList
l = [0, 1, 2, 3, 4]
u = UserList(l)

assert isinstance(u[:], u.__class__)

assert u[:] == u
print("UserListTest::test_slice_type: ok")
