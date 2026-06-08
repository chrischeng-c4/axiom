# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pulldom"
# dimension = "behavior"
# case = "thorough_test_case__test_sax2dom_fail_uc8b1d0c"
# subject = "cpython.test_pulldom.ThoroughTestCase.test_sax2dom_fail"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pulldom.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pulldom
_suite = unittest.defaultTestLoader.loadTestsFromName("ThoroughTestCase.test_sax2dom_fail", test_pulldom)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThoroughTestCase.test_sax2dom_fail did not pass"
print("ThoroughTestCase::test_sax2dom_fail: ok")
