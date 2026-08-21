# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_type_function_called_on_default__test_type_function_call_with_non_string_default"
# subject = "cpython.test_argparse.TestTypeFunctionCalledOnDefault.test_type_function_call_with_non_string_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTypeFunctionCalledOnDefault.test_type_function_call_with_non_string_default", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTypeFunctionCalledOnDefault.test_type_function_call_with_non_string_default did not pass"
print("TestTypeFunctionCalledOnDefault::test_type_function_call_with_non_string_default: ok")
