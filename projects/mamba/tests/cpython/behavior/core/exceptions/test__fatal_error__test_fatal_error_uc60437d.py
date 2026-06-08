# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "test__fatal_error__test_fatal_error_uc60437d"
# subject = "cpython.test_exceptions.Test_FatalError.test_fatal_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_exceptions
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_FatalError.test_fatal_error", test_exceptions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_FatalError.test_fatal_error did not pass"
print("Test_FatalError::test_fatal_error: ok")
