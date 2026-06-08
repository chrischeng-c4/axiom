# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httplib"
# dimension = "behavior"
# case = "basic_test__test_error_leak_uc8d77b3"
# subject = "cpython.test_httplib.BasicTest.test_error_leak"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httplib
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicTest.test_error_leak", test_httplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicTest.test_error_leak did not pass"
print("BasicTest::test_error_leak: ok")
