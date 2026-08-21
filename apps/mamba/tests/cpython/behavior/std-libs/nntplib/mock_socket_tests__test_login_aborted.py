# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "behavior"
# case = "mock_socket_tests__test_login_aborted"
# subject = "cpython.test_nntplib.MockSocketTests.test_login_aborted"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_nntplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_nntplib
_suite = unittest.defaultTestLoader.loadTestsFromName("MockSocketTests.test_login_aborted", test_nntplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MockSocketTests.test_login_aborted did not pass"
print("MockSocketTests::test_login_aborted: ok")
