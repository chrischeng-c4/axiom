# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random_things"
# dimension = "behavior"
# case = "callback_tracback_test_case__test_typeerrordivisionerror_uc803681"
# subject = "cpython.test_random_things.CallbackTracbackTestCase.test_TypeErrorDivisionError"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_random_things.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_random_things
_suite = unittest.defaultTestLoader.loadTestsFromName("CallbackTracbackTestCase.test_TypeErrorDivisionError", test_random_things)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CallbackTracbackTestCase.test_TypeErrorDivisionError did not pass"
print("CallbackTracbackTestCase::test_TypeErrorDivisionError: ok")
