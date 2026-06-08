# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "call"
# dimension = "behavior"
# case = "c_function_calls_error_messages__test_object_not_callable"
# subject = "cpython.test_call.CFunctionCallsErrorMessages.test_object_not_callable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_call.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_call
_suite = unittest.defaultTestLoader.loadTestsFromName("CFunctionCallsErrorMessages.test_object_not_callable", test_call)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CFunctionCallsErrorMessages.test_object_not_callable did not pass"
print("CFunctionCallsErrorMessages::test_object_not_callable: ok")
