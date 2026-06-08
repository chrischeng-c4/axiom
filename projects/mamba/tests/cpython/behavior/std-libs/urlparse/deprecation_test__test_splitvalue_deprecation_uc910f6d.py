# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urlparse"
# dimension = "behavior"
# case = "deprecation_test__test_splitvalue_deprecation_uc910f6d"
# subject = "cpython.test_urlparse.DeprecationTest.test_splitvalue_deprecation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urlparse
_suite = unittest.defaultTestLoader.loadTestsFromName("DeprecationTest.test_splitvalue_deprecation", test_urlparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DeprecationTest.test_splitvalue_deprecation did not pass"
print("DeprecationTest::test_splitvalue_deprecation: ok")
