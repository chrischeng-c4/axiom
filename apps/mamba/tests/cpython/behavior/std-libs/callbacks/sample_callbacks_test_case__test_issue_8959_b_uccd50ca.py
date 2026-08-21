# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "callbacks"
# dimension = "behavior"
# case = "sample_callbacks_test_case__test_issue_8959_b_uccd50ca"
# subject = "cpython.test_callbacks.SampleCallbacksTestCase.test_issue_8959_b"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_callbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_callbacks
_suite = unittest.defaultTestLoader.loadTestsFromName("SampleCallbacksTestCase.test_issue_8959_b", test_callbacks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SampleCallbacksTestCase.test_issue_8959_b did not pass"
print("SampleCallbacksTestCase::test_issue_8959_b: ok")
