# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "callbacks"
# dimension = "behavior"
# case = "sample_callbacks_test_case__test_callback_too_many_args_uc666002"
# subject = "cpython.test_callbacks.SampleCallbacksTestCase.test_callback_too_many_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_callbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_callbacks
_suite = unittest.defaultTestLoader.loadTestsFromName("SampleCallbacksTestCase.test_callback_too_many_args", test_callbacks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SampleCallbacksTestCase.test_callback_too_many_args did not pass"
print("SampleCallbacksTestCase::test_callback_too_many_args: ok")
