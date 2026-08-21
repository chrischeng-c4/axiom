# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "callbacks"
# dimension = "behavior"
# case = "sample_callbacks_test_case__test_convert_result_error_uc1a598e"
# subject = "cpython.test_callbacks.SampleCallbacksTestCase.test_convert_result_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_callbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_callbacks
_suite = unittest.defaultTestLoader.loadTestsFromName("SampleCallbacksTestCase.test_convert_result_error", test_callbacks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SampleCallbacksTestCase.test_convert_result_error did not pass"
print("SampleCallbacksTestCase::test_convert_result_error: ok")
