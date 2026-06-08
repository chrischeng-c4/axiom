# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "behavior"
# case = "test_bytecode_test_case__test_assert_not_in_with_arg_in_bytecode_uc277142"
# subject = "cpython.test_dis.TestBytecodeTestCase.test_assert_not_in_with_arg_in_bytecode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dis.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dis
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBytecodeTestCase.test_assert_not_in_with_arg_in_bytecode", test_dis)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBytecodeTestCase.test_assert_not_in_with_arg_in_bytecode did not pass"
print("TestBytecodeTestCase::test_assert_not_in_with_arg_in_bytecode: ok")
