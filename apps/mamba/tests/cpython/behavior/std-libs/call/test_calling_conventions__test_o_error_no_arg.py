# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "call"
# dimension = "behavior"
# case = "test_calling_conventions__test_o_error_no_arg"
# subject = "cpython.test_call.TestCallingConventions.test_o_error_no_arg"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_call.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_call
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCallingConventions.test_o_error_no_arg", test_call)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCallingConventions.test_o_error_no_arg did not pass"
print("TestCallingConventions::test_o_error_no_arg: ok")
