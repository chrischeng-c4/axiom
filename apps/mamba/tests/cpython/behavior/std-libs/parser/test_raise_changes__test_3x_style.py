# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parser"
# dimension = "behavior"
# case = "test_raise_changes__test_3x_style"
# subject = "cpython.test_parser.TestRaiseChanges.test_3x_style"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRaiseChanges.test_3x_style", test_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRaiseChanges.test_3x_style did not pass"
print("TestRaiseChanges::test_3x_style: ok")
