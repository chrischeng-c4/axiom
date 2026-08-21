# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "with"
# dimension = "behavior"
# case = "exceptional_test_case__testsingleresource_uca108d2"
# subject = "cpython.test_with.ExceptionalTestCase.testSingleResource"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_with.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_with
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionalTestCase.testSingleResource", test_with)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionalTestCase.testSingleResource did not pass"
print("ExceptionalTestCase::testSingleResource: ok")
