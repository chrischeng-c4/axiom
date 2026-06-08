# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "traceback_format_tests__test_recursive_traceback_cpython_internal"
# subject = "cpython.test_traceback.TracebackFormatTests.test_recursive_traceback_cpython_internal"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("TracebackFormatTests.test_recursive_traceback_cpython_internal", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TracebackFormatTests.test_recursive_traceback_cpython_internal did not pass"
print("TracebackFormatTests::test_recursive_traceback_cpython_internal: ok")
