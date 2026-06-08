# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "threaded_tests__test_read_write_after_close_raises_valuerror"
# subject = "cpython.test_ssl.ThreadedTests.test_read_write_after_close_raises_valuerror"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadedTests.test_read_write_after_close_raises_valuerror", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadedTests.test_read_write_after_close_raises_valuerror did not pass"
print("ThreadedTests::test_read_write_after_close_raises_valuerror: ok")
