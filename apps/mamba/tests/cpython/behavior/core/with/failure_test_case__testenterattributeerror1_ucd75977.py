# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "with"
# dimension = "behavior"
# case = "failure_test_case__testenterattributeerror1_ucd75977"
# subject = "cpython.test_with.FailureTestCase.testEnterAttributeError1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_with.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_with
_suite = unittest.defaultTestLoader.loadTestsFromName("FailureTestCase.testEnterAttributeError1", test_with)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FailureTestCase.testEnterAttributeError1 did not pass"
print("FailureTestCase::testEnterAttributeError1: ok")
