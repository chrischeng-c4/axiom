# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "with"
# dimension = "behavior"
# case = "failure_test_case__testenterattributeerror2_uc5f2d8f"
# subject = "cpython.test_with.FailureTestCase.testEnterAttributeError2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_with.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_with
_suite = unittest.defaultTestLoader.loadTestsFromName("FailureTestCase.testEnterAttributeError2", test_with)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FailureTestCase.testEnterAttributeError2 did not pass"
print("FailureTestCase::testEnterAttributeError2: ok")
