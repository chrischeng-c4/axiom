# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "unix_getpass_test__test_falls_back_to_stdin_ucff0035"
# subject = "cpython.test_getpass.UnixGetpassTest.test_falls_back_to_stdin"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_getpass
_suite = unittest.defaultTestLoader.loadTestsFromName("UnixGetpassTest.test_falls_back_to_stdin", test_getpass)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnixGetpassTest.test_falls_back_to_stdin did not pass"
print("UnixGetpassTest::test_falls_back_to_stdin: ok")
