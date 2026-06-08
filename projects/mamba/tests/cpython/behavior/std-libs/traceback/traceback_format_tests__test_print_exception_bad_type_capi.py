# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "traceback_format_tests__test_print_exception_bad_type_capi"
# subject = "cpython.test_traceback.TracebackFormatTests.test_print_exception_bad_type_capi"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("TracebackFormatTests.test_print_exception_bad_type_capi", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TracebackFormatTests.test_print_exception_bad_type_capi did not pass"
print("TracebackFormatTests::test_print_exception_bad_type_capi: ok")
