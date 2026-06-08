# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userlist"
# dimension = "behavior"
# case = "user_list_test__test_getitemoverwriteiter_uc678d29"
# subject = "cpython.test_userlist.UserListTest.test_getitemoverwriteiter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_userlist.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_userlist
_suite = unittest.defaultTestLoader.loadTestsFromName("UserListTest.test_getitemoverwriteiter", test_userlist)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UserListTest.test_getitemoverwriteiter did not pass"
print("UserListTest::test_getitemoverwriteiter: ok")
