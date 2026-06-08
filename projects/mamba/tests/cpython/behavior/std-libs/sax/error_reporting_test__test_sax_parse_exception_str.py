# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sax"
# dimension = "behavior"
# case = "error_reporting_test__test_sax_parse_exception_str"
# subject = "cpython.test_sax.ErrorReportingTest.test_sax_parse_exception_str"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sax.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sax
_suite = unittest.defaultTestLoader.loadTestsFromName("ErrorReportingTest.test_sax_parse_exception_str", test_sax)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ErrorReportingTest.test_sax_parse_exception_str did not pass"
print("ErrorReportingTest::test_sax_parse_exception_str: ok")
