# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userlist"
# dimension = "behavior"
# case = "user_list_test__test_radd_specials"
# subject = "cpython.test_userlist.UserListTest.test_radd_specials"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_userlist.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_userlist.py::UserListTest::test_radd_specials
"""Auto-ported test: UserListTest::test_radd_specials (CPython 3.12 oracle)."""


from collections import UserList
from test import list_tests
import unittest


# --- test body ---
type2test = UserList
u = UserList('eggs')
u2 = 'spam' + u

assert u2 == list('spameggs')
u2 = u.__radd__(UserList('spam'))

assert u2 == list('spameggs')
print("UserListTest::test_radd_specials: ok")
