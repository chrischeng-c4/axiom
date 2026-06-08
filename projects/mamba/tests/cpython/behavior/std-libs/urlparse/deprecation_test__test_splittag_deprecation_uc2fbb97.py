# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urlparse"
# dimension = "behavior"
# case = "deprecation_test__test_splittag_deprecation_uc2fbb97"
# subject = "cpython.test_urlparse.DeprecationTest.test_splittag_deprecation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urlparse
_suite = unittest.defaultTestLoader.loadTestsFromName("DeprecationTest.test_splittag_deprecation", test_urlparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DeprecationTest.test_splittag_deprecation did not pass"
print("DeprecationTest::test_splittag_deprecation: ok")
