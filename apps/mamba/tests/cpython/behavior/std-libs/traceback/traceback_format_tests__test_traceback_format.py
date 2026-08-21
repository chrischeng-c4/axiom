# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "traceback_format_tests__test_traceback_format"
# subject = "cpython.test_traceback.TracebackFormatTests.test_traceback_format"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("TracebackFormatTests.test_traceback_format", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TracebackFormatTests.test_traceback_format did not pass"
print("TracebackFormatTests::test_traceback_format: ok")
