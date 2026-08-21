# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "with"
# dimension = "behavior"
# case = "exceptional_test_case__testexceptionnormalized_ucfced72"
# subject = "cpython.test_with.ExceptionalTestCase.testExceptionNormalized"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_with.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_with
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionalTestCase.testExceptionNormalized", test_with)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionalTestCase.testExceptionNormalized did not pass"
print("ExceptionalTestCase::testExceptionNormalized: ok")
