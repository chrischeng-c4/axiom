# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_type_function_call_only_once__test_type_function_call_only_once"
# subject = "cpython.test_argparse.TestTypeFunctionCallOnlyOnce.test_type_function_call_only_once"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTypeFunctionCallOnlyOnce.test_type_function_call_only_once", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTypeFunctionCallOnlyOnce.test_type_function_call_only_once did not pass"
print("TestTypeFunctionCallOnlyOnce::test_type_function_call_only_once: ok")
