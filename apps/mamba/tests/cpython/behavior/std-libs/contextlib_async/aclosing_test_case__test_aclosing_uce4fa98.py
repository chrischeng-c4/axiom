# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib_async"
# dimension = "behavior"
# case = "aclosing_test_case__test_aclosing_uce4fa98"
# subject = "cpython.test_contextlib_async.AclosingTestCase.test_aclosing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib_async.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_contextlib_async
_suite = unittest.defaultTestLoader.loadTestsFromName("AclosingTestCase.test_aclosing", test_contextlib_async)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AclosingTestCase.test_aclosing did not pass"
print("AclosingTestCase::test_aclosing: ok")
