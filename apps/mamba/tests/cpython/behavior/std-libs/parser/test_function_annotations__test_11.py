# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parser"
# dimension = "behavior"
# case = "test_function_annotations__test_11"
# subject = "cpython.test_parser.TestFunctionAnnotations.test_11"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFunctionAnnotations.test_11", test_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFunctionAnnotations.test_11 did not pass"
print("TestFunctionAnnotations::test_11: ok")
